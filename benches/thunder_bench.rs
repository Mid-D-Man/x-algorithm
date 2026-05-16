// benches/thunder_bench.rs
// Thunder post store benchmarks — DashMap lookups, recency sort, bulk retrieval.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use dashmap::DashMap;
use std::collections::{HashSet, VecDeque};
use std::cmp::Reverse;
use std::sync::Arc;
use x_bench_lib::*;

// ─────────────────────────────────────────────────────────────────────────────
// 1. score_recent — thunder_service.rs hot path.
//    Sort N posts by created_at descending, take top-K.
//    Called once per get_in_network_posts request.
// ─────────────────────────────────────────────────────────────────────────────

fn score_recent(mut posts: Vec<LightPost>, max_results: usize) -> Vec<LightPost> {
    posts.sort_unstable_by_key(|p| Reverse(p.created_at));
    posts.into_iter().take(max_results).collect()
}

fn bench_score_recent(c: &mut Criterion) {
    let mut g = c.benchmark_group("thunder/score_recent");

    for (n, k) in [(10_000, 200), (50_000, 200), (100_000, 400)] {
        let posts = make_light_posts(n);
        g.throughput(Throughput::Elements(n as u64));
        g.bench_function(format!("{n}_posts_top_{k}"), |b| {
            b.iter_batched(
                || posts.clone(),
                |p| black_box(score_recent(p, k)),
                BatchSize::LargeInput,
            )
        });
    }

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. DashMap post store — concurrent-safe in production.
//    get_posts_from_map iterates user IDs, looks up VecDeque<TinyPost>,
//    then does a nested DashMap lookup for each post's full data.
//    This double-lookup is the key bottleneck in thunder under load.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct TinyPost {
    pub post_id:    i64,
    pub created_at: i64,
}

fn bench_dashmap_post_store(c: &mut Criterion) {
    let mut g = c.benchmark_group("thunder/dashmap_post_store");

    // Build a post store: 500 users × 200 posts each = 100k posts total
    let posts_map:    Arc<DashMap<i64, LightPost>>              = Arc::new(DashMap::new());
    let by_user_map:  Arc<DashMap<i64, VecDeque<TinyPost>>>     = Arc::new(DashMap::new());

    let all_posts = make_light_posts(100_000);
    for post in &all_posts {
        posts_map.insert(post.post_id, post.clone());
        by_user_map
            .entry(post.author_id)
            .or_default()
            .push_back(TinyPost { post_id: post.post_id, created_at: post.created_at });
    }

    // Simulate get_posts_from_map for a user following 500 accounts
    let following: Vec<i64> = (1..=500).collect();
    let exclude: HashSet<i64> = HashSet::new();

    // Simulate the inner loop from post_store.rs
    let lookup = |following: &[i64], posts_map: &DashMap<i64, LightPost>,
                  by_user: &DashMap<i64, VecDeque<TinyPost>>,
                  exclude: &HashSet<i64>| -> Vec<LightPost> {
        let mut result = Vec::with_capacity(1_000);
        for user_id in following {
            if let Some(user_posts) = by_user.get(user_id) {
                let iter = user_posts.iter().rev()
                    .filter(|tp| !exclude.contains(&tp.post_id))
                    .take(20); // MAX_ORIGINAL_POSTS_PER_AUTHOR
                for tiny in iter {
                    if let Some(post) = posts_map.get(&tiny.post_id) {
                        result.push(post.clone());
                    }
                }
            }
        }
        result
    };

    g.throughput(Throughput::Elements(500));
    g.bench_function("500_following_20_posts_each", |b| {
        let pm  = Arc::clone(&posts_map);
        let bum = Arc::clone(&by_user_map);
        b.iter(|| {
            black_box(lookup(
                black_box(&following),
                &pm,
                &bum,
                &exclude,
            ))
        })
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Post store trim — trim_old_posts is called every 2 minutes.
//    Walks all VecDeques from front, pops expired posts.
//    This is a maintenance path but affects p99 latency when it runs.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_trim_old_posts(c: &mut Criterion) {
    let mut g = c.benchmark_group("thunder/trim_old_posts");

    let build_store = || -> DashMap<i64, VecDeque<TinyPost>> {
        let store: DashMap<i64, VecDeque<TinyPost>> = DashMap::new();
        let posts = make_light_posts(100_000);
        for post in &posts {
            store.entry(post.author_id).or_default().push_back(
                TinyPost { post_id: post.post_id, created_at: post.created_at }
            );
        }
        store
    };

    let retention_secs: i64 = 2 * 24 * 3600; // 2 days
    let now = 1_700_000_000i64 + 100_000 * 10; // just past all posts

    g.bench_function("100k_posts_none_expired", |b| {
        b.iter_batched(
            build_store,
            |store| {
                let cutoff = now - retention_secs;
                let mut trimmed = 0usize;
                for mut entry in store.iter_mut() {
                    while let Some(tp) = entry.front() {
                        if tp.created_at < cutoff {
                            entry.pop_front();
                            trimmed += 1;
                        } else {
                            break;
                        }
                    }
                }
                black_box(trimmed)
            },
            BatchSize::LargeInput,
        )
    });

    // Simulate 10% expired
    let old_now = 1_700_000_000i64 + 10_000 * 10; // only first 10k still valid
    g.bench_function("100k_posts_90pct_expired", |b| {
        b.iter_batched(
            build_store,
            |store| {
                let cutoff = old_now - retention_secs;
                let mut trimmed = 0usize;
                for mut entry in store.iter_mut() {
                    while let Some(tp) = entry.front() {
                        if tp.created_at < cutoff {
                            entry.pop_front();
                            trimmed += 1;
                        } else {
                            break;
                        }
                    }
                }
                black_box(trimmed)
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_score_recent,
    bench_dashmap_post_store,
    bench_trim_old_posts,
);
criterion_main!(benches);
