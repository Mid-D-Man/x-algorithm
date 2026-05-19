// benches/thunder_bench.rs
// Thunder post store benchmarks — DashMap lookups, recency sort, bulk retrieval.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use dashmap::DashMap;
use std::collections::{HashSet, VecDeque};
use std::cmp::Reverse;
use std::sync::Arc;
use x_bench_lib::*;

// ─────────────────────────────────────────────────────────────────────────────
// 1. score_recent — select_nth_unstable_by (partial sort) vs full sort.
// ─────────────────────────────────────────────────────────────────────────────
fn score_recent(mut posts: Vec<LightPost>, max_results: usize) -> Vec<LightPost> {
    let n = posts.len();
    if max_results == 0 { return Vec::new(); }
    if max_results >= n {
        posts.sort_unstable_by_key(|p| Reverse(p.created_at));
        return posts;
    }
    posts.select_nth_unstable_by(max_results, |a, b| b.created_at.cmp(&a.created_at));
    let mut top = posts[..max_results].to_vec();
    top.sort_unstable_by_key(|p| Reverse(p.created_at));
    top
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
// 2. DashMap double-lookup (current production pattern) — the bottleneck.
//    Outer: user_id → VecDeque<TinyPost>
//    Inner: post_id → LightPost   (10k lookups per request)
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Clone, Debug)]
struct TinyPost {
    pub post_id:    i64,
    pub created_at: i64,
}

fn bench_dashmap_post_store(c: &mut Criterion) {
    let mut g = c.benchmark_group("thunder/dashmap_post_store");

    let posts_map:   Arc<DashMap<i64, LightPost>>           = Arc::new(DashMap::new());
    let by_user_map: Arc<DashMap<i64, VecDeque<TinyPost>>>  = Arc::new(DashMap::new());

    let all_posts = make_light_posts(100_000);
    for post in &all_posts {
        posts_map.insert(post.post_id, post.clone());
        by_user_map
            .entry(post.author_id)
            .or_default()
            .push_back(TinyPost { post_id: post.post_id, created_at: post.created_at });
    }

    let following: Vec<i64> = (1..=500).collect();
    let exclude: HashSet<i64> = HashSet::new();

    // Current production: two DashMap lookups per post
    let double_lookup = |following: &[i64],
                         posts_map: &DashMap<i64, LightPost>,
                         by_user: &DashMap<i64, VecDeque<TinyPost>>,
                         exclude: &HashSet<i64>| -> Vec<LightPost> {
        let mut result = Vec::with_capacity(1_000);
        for user_id in following {
            if let Some(user_posts) = by_user.get(user_id) {
                for tiny in user_posts.iter().rev()
                    .filter(|tp| !exclude.contains(&tp.post_id))
                    .take(20)
                {
                    // INNER LOOKUP — 10k of these per request
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
        b.iter(|| black_box(double_lookup(&following, &pm, &bum, &exclude)))
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Single-map lookup (denormalised) — stores LightPost directly in per-user
//    deque.  Eliminates the inner DashMap.get() entirely; deletion handled by
//    a separate deleted_ids HashSet checked at serve time.
// ─────────────────────────────────────────────────────────────────────────────
fn bench_dashmap_post_store_denorm(c: &mut Criterion) {
    let mut g = c.benchmark_group("thunder/dashmap_post_store_denorm");

    // Single map: user_id → VecDeque<LightPost>
    let by_user_map: Arc<DashMap<i64, VecDeque<LightPost>>> = Arc::new(DashMap::new());
    let deleted_ids: Arc<HashSet<i64>> = Arc::new(HashSet::new());

    let all_posts = make_light_posts(100_000);
    for post in &all_posts {
        by_user_map
            .entry(post.author_id)
            .or_default()
            .push_back(post.clone());
    }

    let following: Vec<i64> = (1..=500).collect();
    let exclude: HashSet<i64> = HashSet::new();

    let single_lookup = |following: &[i64],
                         by_user: &DashMap<i64, VecDeque<LightPost>>,
                         exclude: &HashSet<i64>,
                         deleted: &HashSet<i64>| -> Vec<LightPost> {
        let mut result = Vec::with_capacity(1_000);
        for user_id in following {
            if let Some(user_posts) = by_user.get(user_id) {
                for post in user_posts.iter().rev()
                    .filter(|p| !exclude.contains(&p.post_id) && !deleted.contains(&p.post_id))
                    .take(20)
                {
                    result.push(post.clone());
                }
            }
        }
        result
    };

    g.throughput(Throughput::Elements(500));
    g.bench_function("500_following_20_posts_each", |b| {
        let bum = Arc::clone(&by_user_map);
        let del = Arc::clone(&deleted_ids);
        b.iter(|| black_box(single_lookup(&following, &bum, &exclude, &del)))
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Post store trim.
// ─────────────────────────────────────────────────────────────────────────────
fn bench_trim_old_posts(c: &mut Criterion) {
    let mut g = c.benchmark_group("thunder/trim_old_posts");

    let build_store = || -> DashMap<i64, VecDeque<TinyPost>> {
        let store: DashMap<i64, VecDeque<TinyPost>> = DashMap::new();
        for post in &make_light_posts(100_000) {
            store.entry(post.author_id).or_default().push_back(
                TinyPost { post_id: post.post_id, created_at: post.created_at }
            );
        }
        store
    };

    let retention_secs: i64 = 2 * 24 * 3600;
    let now = 1_700_000_000i64 + 100_000 * 10;

    g.bench_function("100k_posts_none_expired", |b| {
        b.iter_batched(build_store, |store| {
            let cutoff = now - retention_secs;
            let mut trimmed = 0usize;
            for mut entry in store.iter_mut() {
                while let Some(tp) = entry.front() {
                    if tp.created_at < cutoff { entry.pop_front(); trimmed += 1; }
                    else { break; }
                }
            }
            black_box(trimmed)
        }, BatchSize::LargeInput)
    });

    let old_now = 1_700_000_000i64 + 10_000 * 10;
    g.bench_function("100k_posts_90pct_expired", |b| {
        b.iter_batched(build_store, |store| {
            let cutoff = old_now - retention_secs;
            let mut trimmed = 0usize;
            for mut entry in store.iter_mut() {
                while let Some(tp) = entry.front() {
                    if tp.created_at < cutoff { entry.pop_front(); trimmed += 1; }
                    else { break; }
                }
            }
            black_box(trimmed)
        }, BatchSize::LargeInput)
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_score_recent,
    bench_dashmap_post_store,
    bench_dashmap_post_store_denorm,
    bench_trim_old_posts,
);
criterion_main!(benches);
