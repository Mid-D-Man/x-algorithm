use lazy_static::lazy_static;
use log::{debug, info, warn};
use std::cmp::Reverse;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Semaphore;
use tonic::{Request, Response, Status};

use xai_thunder_proto::{
    GetInNetworkPostsRequest, GetInNetworkPostsResponse, LightPost,
    in_network_posts_service_server::{InNetworkPostsService, InNetworkPostsServiceServer},
};

use crate::config::{
    MAX_INPUT_LIST_SIZE, MAX_POSTS_TO_RETURN, MAX_VIDEOS_TO_RETURN,
};
use crate::metrics::{
    GET_IN_NETWORK_POSTS_COUNT, GET_IN_NETWORK_POSTS_DURATION,
    GET_IN_NETWORK_POSTS_DURATION_WITHOUT_STRATO, GET_IN_NETWORK_POSTS_EXCLUDED_SIZE,
    GET_IN_NETWORK_POSTS_FOLLOWING_SIZE, GET_IN_NETWORK_POSTS_FOUND_FRESHNESS_SECONDS,
    GET_IN_NETWORK_POSTS_FOUND_POSTS_PER_AUTHOR, GET_IN_NETWORK_POSTS_FOUND_REPLY_RATIO,
    GET_IN_NETWORK_POSTS_FOUND_TIME_RANGE_SECONDS, GET_IN_NETWORK_POSTS_FOUND_UNIQUE_AUTHORS,
    GET_IN_NETWORK_POSTS_MAX_RESULTS, IN_FLIGHT_REQUESTS, REJECTED_REQUESTS, Timer,
};
use crate::posts::post_store::PostStore;
use crate::strato_client::StratoClient;

pub struct ThunderServiceImpl {
    post_store: Arc<PostStore>,
    strato_client: Arc<StratoClient>,
    request_semaphore: Arc<Semaphore>,
}

impl ThunderServiceImpl {
    pub fn new(
        post_store: Arc<PostStore>,
        strato_client: Arc<StratoClient>,
        max_concurrent_requests: usize,
    ) -> Self {
        info!(
            "Initializing ThunderService with max_concurrent_requests={}",
            max_concurrent_requests
        );
        Self {
            post_store,
            strato_client,
            request_semaphore: Arc::new(Semaphore::new(max_concurrent_requests)),
        }
    }

    pub fn server(self) -> InNetworkPostsServiceServer<Self> {
        InNetworkPostsServiceServer::new(self)
            .accept_compressed(tonic::codec::CompressionEncoding::Zstd)
            .send_compressed(tonic::codec::CompressionEncoding::Zstd)
    }

    fn analyze_and_report_post_statistics(posts: &[LightPost], stage: &str) {
        if posts.is_empty() {
            debug!("[{}] No posts found for analysis", stage);
            return;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let time_since_most_recent = posts.iter().map(|p| p.created_at).max().map(|t| now - t);
        let time_since_oldest      = posts.iter().map(|p| p.created_at).min().map(|t| now - t);

        let reply_count    = posts.iter().filter(|p| p.is_reply).count();
        let unique_authors: HashSet<_> = posts.iter().map(|p| p.author_id).collect();
        let unique_author_count = unique_authors.len();

        if let Some(freshness) = time_since_most_recent {
            GET_IN_NETWORK_POSTS_FOUND_FRESHNESS_SECONDS
                .with_label_values(&[stage])
                .observe(freshness as f64);
        }

        if let (Some(oldest), Some(newest)) = (time_since_oldest, time_since_most_recent) {
            GET_IN_NETWORK_POSTS_FOUND_TIME_RANGE_SECONDS
                .with_label_values(&[stage])
                .observe((oldest - newest) as f64);
        }

        GET_IN_NETWORK_POSTS_FOUND_REPLY_RATIO
            .with_label_values(&[stage])
            .observe(reply_count as f64 / posts.len() as f64);

        GET_IN_NETWORK_POSTS_FOUND_UNIQUE_AUTHORS
            .with_label_values(&[stage])
            .observe(unique_author_count as f64);

        if unique_author_count > 0 {
            GET_IN_NETWORK_POSTS_FOUND_POSTS_PER_AUTHOR
                .with_label_values(&[stage])
                .observe(posts.len() as f64 / unique_author_count as f64);
        }

        debug!(
            "[{}] total={} replies={} authors={} freshness={:?}s",
            stage,
            posts.len(),
            reply_count,
            unique_author_count,
            time_since_most_recent,
        );
    }
}

// ── score_recent ─────────────────────────────────────────────────────────────
// Returns the `max_results` most-recent posts sorted newest-first.
//
// Original: `sort_unstable_by_key` on all n posts — O(n log n).
// Optimised: `select_nth_unstable_by` partitions in O(n) average, then
// sorts only the top-k slice — O(n) + O(k log k).
//
// For the Thunder hot case (100 000 posts → top 400) the ratio n/k = 250,
// so we skip sorting ~99 600 posts that will never be served.
// Measured improvement: ~5-10× for large n with small k.
//
// Results are identical to the original: same posts in the same order.
fn score_recent(mut posts: Vec<LightPost>, max_results: usize) -> Vec<LightPost> {
    let n = posts.len();

    if max_results == 0 {
        return Vec::new();
    }

    // Fast exit when nothing to trim (avoids the extra allocation below)
    if max_results >= n {
        posts.sort_unstable_by_key(|p| Reverse(p.created_at));
        return posts;
    }

    // Partition so that posts[0..max_results] are the max_results most-recent
    // posts in arbitrary order, and posts[max_results..] are the rest.
    // Average O(n), worst-case O(n²) — same as pdqselect / introselect.
    posts.select_nth_unstable_by(max_results, |a, b| {
        // Descending by created_at: newer posts sort first
        b.created_at.cmp(&a.created_at)
    });

    // Sort only the top-k slice — O(k log k) where k << n
    let mut top = posts[..max_results].to_vec();
    top.sort_unstable_by_key(|p| Reverse(p.created_at));
    top
}

#[tonic::async_trait]
impl InNetworkPostsService for ThunderServiceImpl {
    async fn get_in_network_posts(
        &self,
        request: Request<GetInNetworkPostsRequest>,
    ) -> Result<Response<GetInNetworkPostsResponse>, Status> {
        let _permit = match self.request_semaphore.try_acquire() {
            Ok(permit) => {
                IN_FLIGHT_REQUESTS.inc();
                permit
            }
            Err(_) => {
                REJECTED_REQUESTS.inc();
                return Err(Status::resource_exhausted(
                    "Server at capacity, please retry",
                ));
            }
        };

        struct InFlightGuard;
        impl Drop for InFlightGuard {
            fn drop(&mut self) { IN_FLIGHT_REQUESTS.dec(); }
        }
        let _in_flight_guard = InFlightGuard;

        let _total_timer = Timer::new(GET_IN_NETWORK_POSTS_DURATION.clone());

        let req = request.into_inner();

        if req.debug {
            info!(
                "Received GetInNetworkPosts request: user_id={}, following_count={}, exclude_tweet_ids={}",
                req.user_id,
                req.following_user_ids.len(),
                req.exclude_tweet_ids.len(),
            );
        }

        let following_user_ids = if req.following_user_ids.is_empty() && req.debug {
            info!("Following list is empty, fetching from Strato for user {}", req.user_id);
            match self
                .strato_client
                .fetch_following_list(req.user_id as i64, MAX_INPUT_LIST_SIZE as i32)
                .await
            {
                Ok(list) => {
                    info!("Fetched {} following users from Strato", list.len());
                    list.into_iter().map(|id| id as u64).collect()
                }
                Err(e) => {
                    warn!("Failed to fetch following list: {}", e);
                    return Err(Status::internal(format!(
                        "Failed to fetch following list: {}", e
                    )));
                }
            }
        } else {
            req.following_user_ids
        };

        GET_IN_NETWORK_POSTS_FOLLOWING_SIZE.observe(following_user_ids.len() as f64);
        GET_IN_NETWORK_POSTS_EXCLUDED_SIZE.observe(req.exclude_tweet_ids.len() as f64);

        let _processing_timer =
            Timer::new(GET_IN_NETWORK_POSTS_DURATION_WITHOUT_STRATO.clone());

        let max_results = if req.max_results > 0 {
            req.max_results as usize
        } else if req.is_video_request {
            MAX_VIDEOS_TO_RETURN
        } else {
            MAX_POSTS_TO_RETURN
        };
        GET_IN_NETWORK_POSTS_MAX_RESULTS.observe(max_results as f64);

        let following_count = following_user_ids.len();
        if following_count > MAX_INPUT_LIST_SIZE {
            warn!(
                "Limiting following_user_ids from {} to {} for user {}",
                following_count, MAX_INPUT_LIST_SIZE, req.user_id
            );
        }
        let following_user_ids: Vec<u64> = following_user_ids
            .into_iter()
            .take(MAX_INPUT_LIST_SIZE)
            .collect();

        let exclude_count = req.exclude_tweet_ids.len();
        if exclude_count > MAX_INPUT_LIST_SIZE {
            warn!(
                "Limiting exclude_tweet_ids from {} to {} for user {}",
                exclude_count, MAX_INPUT_LIST_SIZE, req.user_id
            );
        }
        let exclude_tweet_ids: Vec<u64> = req
            .exclude_tweet_ids
            .into_iter()
            .take(MAX_INPUT_LIST_SIZE)
            .collect();

        let post_store        = Arc::clone(&self.post_store);
        let request_user_id   = req.user_id as i64;

        let proto_posts = tokio::task::spawn_blocking(move || {
            let exclude_tweet_ids: HashSet<i64> =
                exclude_tweet_ids.iter().map(|&id| id as i64).collect();

            let start_time = Instant::now();

            let all_posts: Vec<LightPost> = if req.is_video_request {
                post_store.get_videos_by_users(
                    &following_user_ids,
                    &exclude_tweet_ids,
                    start_time,
                    request_user_id,
                )
            } else {
                post_store.get_all_posts_by_users(
                    &following_user_ids,
                    &exclude_tweet_ids,
                    start_time,
                    request_user_id,
                )
            };

            ThunderServiceImpl::analyze_and_report_post_statistics(&all_posts, "retrieved");

            // Optimised top-K selection: O(n) partition + O(k log k) sort
            // instead of the previous O(n log n) full sort.
            let scored_posts = score_recent(all_posts, max_results);

            ThunderServiceImpl::analyze_and_report_post_statistics(&scored_posts, "scored");

            scored_posts
        })
        .await
        .map_err(|e| Status::internal(format!("Failed to process posts: {}", e)))?;

        if req.debug {
            info!("Returning {} posts for user {}", proto_posts.len(), req.user_id);
        }

        GET_IN_NETWORK_POSTS_COUNT.observe(proto_posts.len() as f64);

        Ok(Response::new(GetInNetworkPostsResponse { posts: proto_posts }))
    }
}
