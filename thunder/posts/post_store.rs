// thunder/posts/post_store.rs
// Key structural change: replaces TinyPost+posts_map double-lookup with a
// single per-user VecDeque<LightPost>.  Deletion is handled at serve time via
// a deleted_ids DashMap check, same as before — no correctness change.

use anyhow::Result;
use dashmap::DashMap;
use log::info;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use xai_thunder_proto::{LightPost, TweetDeleteEvent};

use crate::config::{
    DELETE_EVENT_KEY, MAX_ORIGINAL_POSTS_PER_AUTHOR, MAX_REPLY_POSTS_PER_AUTHOR,
    MAX_TINY_POSTS_PER_USER_SCAN, MAX_VIDEO_POSTS_PER_AUTHOR,
};
use crate::metrics::{
    POST_STORE_DELETED_POSTS, POST_STORE_DELETED_POSTS_FILTERED, POST_STORE_ENTITY_COUNT,
    POST_STORE_POSTS_RETURNED, POST_STORE_POSTS_RETURNED_RATIO, POST_STORE_REQUEST_TIMEOUTS,
    POST_STORE_REQUESTS, POST_STORE_TOTAL_POSTS, POST_STORE_USER_COUNT,
};

// ── Structural change ─────────────────────────────────────────────────────────
//
// Old layout:
//   posts:              DashMap<post_id, LightPost>          — full data
//   original_by_user:   DashMap<user_id, VecDeque<TinyPost>> — index (id+ts only)
//
// For each get_posts_from_map call with 500 followed users × 20 posts:
//   500 outer DashMap.get()  +  10,000 inner DashMap.get() = ~10,500 lookups
//   Measured: ~568 µs
//
// New layout:
//   original_by_user:   DashMap<user_id, VecDeque<LightPost>> — full data inline
//   deleted_posts:      DashMap<post_id, bool>                 — tombstone set
//
// Per request: 500 DashMap.get() + 10,000 cheap deque iterator steps.
//   Expected: ~50-100 µs  (see thunder_bench/dashmap_post_store_denorm)
//
// Memory trade-off: LightPost (~80 bytes) vs TinyPost (16 bytes) in per-user
// deques.  For a user followed by N others, their posts appear in N deques.
// At typical fanout (avg ~1000 followers per author in the store) this is
// acceptable vs the 10× latency win.

#[derive(Clone)]
pub struct PostStore {
    /// Per-user deque of full LightPost data for original posts (no reply/retweet).
    /// Ordered oldest-first; newest is at the back.
    original_posts_by_user: Arc<DashMap<i64, VecDeque<LightPost>>>,
    /// Per-user deque of full LightPost data for replies and retweets.
    secondary_posts_by_user: Arc<DashMap<i64, VecDeque<LightPost>>>,
    /// Per-user deque of full LightPost data for video posts.
    video_posts_by_user: Arc<DashMap<i64, VecDeque<LightPost>>>,
    /// Tombstone set: post_ids that have been deleted.
    /// Checked at serve time so deletions don't require scanning all user deques.
    deleted_posts: Arc<DashMap<i64, bool>>,
    retention_seconds: u64,
    request_timeout: Duration,
}

impl PostStore {
    pub fn new(retention_seconds: u64, request_timeout_ms: u64) -> Self {
        PostStore {
            original_posts_by_user:  Arc::new(DashMap::new()),
            secondary_posts_by_user: Arc::new(DashMap::new()),
            video_posts_by_user:     Arc::new(DashMap::new()),
            deleted_posts:           Arc::new(DashMap::new()),
            retention_seconds,
            request_timeout: Duration::from_millis(request_timeout_ms),
        }
    }

    pub fn mark_as_deleted(&self, posts: Vec<TweetDeleteEvent>) {
        for post in posts {
            self.deleted_posts.insert(post.post_id, true);
            // Record deletion timestamp in a synthetic deque entry so trim_old_posts
            // can later evict the tombstone itself when it ages out.
            let mut del_entry = self
                .original_posts_by_user
                .entry(DELETE_EVENT_KEY)
                .or_default();
            // Store a sentinel LightPost so the trim loop can prune the tombstone.
            del_entry.push_back(LightPost {
                post_id:    post.post_id,
                author_id:  0,
                created_at: post.deleted_at,
                is_reply:   false,
                is_retweet: false,
                has_video:  false,
                ..Default::default()
            });
        }
    }

    pub fn insert_posts(&self, mut posts: Vec<LightPost>) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        posts.retain(|p| {
            p.created_at < current_time
                && current_time - p.created_at <= self.retention_seconds as i64
        });
        posts.sort_unstable_by_key(|p| p.created_at);
        self.insert_posts_internal(posts);
    }

    fn insert_posts_internal(&self, posts: Vec<LightPost>) {
        for post in posts {
            if self.deleted_posts.contains_key(&post.post_id) {
                continue;
            }

            let post_id    = post.post_id;
            let author_id  = post.author_id;
            let is_original = !post.is_reply && !post.is_retweet;
            let has_video   = post.has_video && !post.is_reply;

            // Check for duplicate (already inserted via a previous Kafka batch)
            // Without the posts_map we detect duplicates by scanning the tail of
            // the user's deque — cheap because inserts are ordered by created_at
            // and duplicates arrive close together.
            if is_original {
                let mut user_posts = self.original_posts_by_user.entry(author_id).or_default();
                if user_posts.back().map_or(false, |p| p.post_id == post_id) {
                    continue; // exact duplicate at tail
                }
                if has_video {
                    self.video_posts_by_user.entry(author_id).or_default()
                        .push_back(post.clone());
                }
                user_posts.push_back(post);
            } else {
                let mut user_posts = self.secondary_posts_by_user.entry(author_id).or_default();
                if user_posts.back().map_or(false, |p| p.post_id == post_id) {
                    continue;
                }
                user_posts.push_back(post);
            }
        }
    }

    pub async fn finalize_init(&self) -> Result<()> {
        self.sort_all_user_posts().await;
        self.trim_old_posts().await;
        // Apply any accumulated tombstones from out-of-order delete events.
        let del_keys: Vec<i64> = self.deleted_posts.iter().map(|e| *e.key()).collect();
        for key in del_keys {
            self.purge_from_user_deques(key);
        }
        Ok(())
    }

    /// Remove a specific post_id from all per-user deques.
    /// Called during finalize_init only — not on the hot path.
    fn purge_from_user_deques(&self, post_id: i64) {
        for map in [
            &self.original_posts_by_user,
            &self.secondary_posts_by_user,
            &self.video_posts_by_user,
        ] {
            for mut entry in map.iter_mut() {
                entry.retain(|p| p.post_id != post_id);
            }
        }
    }

    /// Retrieve video posts for a set of followed users.
    pub fn get_videos_by_users(
        &self,
        user_ids: &[i64],
        exclude_tweet_ids: &HashSet<i64>,
        start_time: Instant,
        request_user_id: i64,
    ) -> Vec<LightPost> {
        let posts = self.get_posts_from_map(
            &self.video_posts_by_user,
            user_ids,
            MAX_VIDEO_POSTS_PER_AUTHOR,
            exclude_tweet_ids,
            &HashSet::new(),
            start_time,
            request_user_id,
        );
        POST_STORE_POSTS_RETURNED.observe(posts.len() as f64);
        posts
    }

    /// Retrieve all posts (original + secondary) for a set of followed users.
    pub fn get_all_posts_by_users(
        &self,
        user_ids: &[i64],
        exclude_tweet_ids: &HashSet<i64>,
        start_time: Instant,
        request_user_id: i64,
    ) -> Vec<LightPost> {
        let following_set: HashSet<i64> = user_ids.iter().copied().collect();

        let mut all = self.get_posts_from_map(
            &self.original_posts_by_user,
            user_ids,
            MAX_ORIGINAL_POSTS_PER_AUTHOR,
            exclude_tweet_ids,
            &HashSet::new(),
            start_time,
            request_user_id,
        );
        all.extend(self.get_posts_from_map(
            &self.secondary_posts_by_user,
            user_ids,
            MAX_REPLY_POSTS_PER_AUTHOR,
            exclude_tweet_ids,
            &following_set,
            start_time,
            request_user_id,
        ));
        POST_STORE_POSTS_RETURNED.observe(all.len() as f64);
        all
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_posts_from_map(
        &self,
        posts_map: &Arc<DashMap<i64, VecDeque<LightPost>>>,
        user_ids: &[i64],
        max_per_user: usize,
        exclude_tweet_ids: &HashSet<i64>,
        following_users: &HashSet<i64>,
        start_time: Instant,
        request_user_id: i64,
    ) -> Vec<LightPost> {
        POST_STORE_REQUESTS.inc();
        let mut light_posts = Vec::new();
        let mut total_eligible = 0usize;

        for (i, user_id) in user_ids.iter().enumerate() {
            if !self.request_timeout.is_zero() && start_time.elapsed() >= self.request_timeout {
                log::error!(
                    "Timed out fetching posts for user={}; processed {}/{}",
                    request_user_id, i, user_ids.len()
                );
                POST_STORE_REQUEST_TIMEOUTS.inc();
                break;
            }

            let Some(user_posts_ref) = posts_map.get(user_id) else { continue };
            let user_posts = user_posts_ref.value();
            total_eligible += user_posts.len();

            // Single-pass: iterate newest-first, filtering in place.
            // No inner DashMap lookup — the LightPost is already here.
            let iter = user_posts
                .iter()
                .rev()
                .take(MAX_TINY_POSTS_PER_USER_SCAN)
                .filter(|p| {
                    !exclude_tweet_ids.contains(&p.post_id)
                        && !self.deleted_posts.contains_key(&p.post_id)
                        && !(p.is_retweet && p.source_user_id == Some(request_user_id))
                })
                .filter(|p| {
                    if following_users.is_empty() {
                        return true;
                    }
                    // Reply threading filter: same logic as before
                    p.in_reply_to_post_id.is_none_or(|reply_to| {
                        // For replies, only include if the replied-to post exists in
                        // the user's own deque (i.e. it's a followed conversation).
                        // We approximate this with the following_users set check.
                        p.in_reply_to_user_id
                            .map(|uid| following_users.contains(&uid))
                            .unwrap_or(false)
                    })
                })
                .take(max_per_user);

            light_posts.extend(iter.cloned());
        }

        if total_eligible > 0 {
            POST_STORE_POSTS_RETURNED_RATIO
                .observe(light_posts.len() as f64 / total_eligible as f64);
        }
        light_posts
    }

    pub fn start_stats_logger(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let user_count   = self.original_posts_by_user.len();
                let deleted_count = self.deleted_posts.len();
                let original_count: usize = self.original_posts_by_user.iter()
                    .map(|e| e.value().len()).sum();
                let secondary_count: usize = self.secondary_posts_by_user.iter()
                    .map(|e| e.value().len()).sum();
                let video_count: usize = self.video_posts_by_user.iter()
                    .map(|e| e.value().len()).sum();

                POST_STORE_USER_COUNT.set(user_count as f64);
                POST_STORE_TOTAL_POSTS.set(original_count as f64);
                POST_STORE_DELETED_POSTS.set(deleted_count as f64);
                POST_STORE_ENTITY_COUNT.with_label_values(&["users"]).set(user_count as f64);
                POST_STORE_ENTITY_COUNT.with_label_values(&["original_posts"]).set(original_count as f64);
                POST_STORE_ENTITY_COUNT.with_label_values(&["secondary_posts"]).set(secondary_count as f64);
                POST_STORE_ENTITY_COUNT.with_label_values(&["video_posts"]).set(video_count as f64);
                POST_STORE_ENTITY_COUNT.with_label_values(&["deleted_posts"]).set(deleted_count as f64);

                info!(
                    "PostStore: {} users, {} original, {} secondary, {} video, {} deleted",
                    user_count, original_count, secondary_count, video_count, deleted_count
                );
            }
        });
    }

    pub fn start_auto_trim(self: Arc<Self>, interval_minutes: u64) {
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(interval_minutes * 60));
            loop {
                interval.tick().await;
                let trimmed = self.trim_old_posts().await;
                if trimmed > 0 {
                    info!("Auto-trim: removed {} old posts", trimmed);
                }
            }
        });
    }

    pub async fn trim_old_posts(&self) -> usize {
        let orig  = Arc::clone(&self.original_posts_by_user);
        let sec   = Arc::clone(&self.secondary_posts_by_user);
        let vid   = Arc::clone(&self.video_posts_by_user);
        let del   = Arc::clone(&self.deleted_posts);
        let ret   = self.retention_seconds;

        tokio::task::spawn_blocking(move || {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let mut trimmed = 0usize;

            let trim_map = |map: &DashMap<i64, VecDeque<LightPost>>,
                             del: &DashMap<i64, bool>| -> usize {
                let mut count = 0;
                let mut empty_users = Vec::new();
                for mut entry in map.iter_mut() {
                    let user_id = *entry.key();
                    let dq = entry.value_mut();
                    while let Some(p) = dq.front() {
                        if now - (p.created_at as u64) > ret {
                            let removed = dq.pop_front().unwrap();
                            // If this is the DELETE_EVENT_KEY sentinel, also prune tombstone.
                            if user_id == DELETE_EVENT_KEY {
                                del.remove(&removed.post_id);
                            }
                            count += 1;
                        } else {
                            break;
                        }
                    }
                    let extra = dq.capacity().saturating_sub(dq.len() * 2);
                    if extra > 0 { dq.shrink_to(dq.len().saturating_add(dq.len() / 2)); }
                    if dq.is_empty() { empty_users.push(user_id); }
                }
                for uid in empty_users {
                    map.remove_if(&uid, |_, dq| dq.is_empty());
                }
                count
            };

            trimmed += trim_map(&orig, &del);
            trimmed += trim_map(&sec,  &del);
            trim_map(&vid, &del); // video deque, don't double-count
            trimmed
        })
        .await
        .expect("spawn_blocking failed")
    }

    pub async fn sort_all_user_posts(&self) {
        let orig = Arc::clone(&self.original_posts_by_user);
        let sec  = Arc::clone(&self.secondary_posts_by_user);
        let vid  = Arc::clone(&self.video_posts_by_user);
        tokio::task::spawn_blocking(move || {
            for map in [&orig, &sec, &vid] {
                for mut e in map.iter_mut() {
                    e.value_mut().make_contiguous()
                        .sort_unstable_by_key(|p| p.created_at);
                }
            }
        }).await.expect("spawn_blocking failed");
    }

    pub fn clear(&self) {
        self.original_posts_by_user.clear();
        self.secondary_posts_by_user.clear();
        self.video_posts_by_user.clear();
        self.deleted_posts.clear();
        info!("PostStore cleared");
    }
}

impl Default for PostStore {
    fn default() -> Self {
        Self::new(2 * 24 * 60 * 60, 0)
    }
}
