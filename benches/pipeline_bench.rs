// benches/pipeline_bench.rs
// Filter pipeline benchmarks — DropDuplicates, BloomFilter, Dedup conversation.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::collections::{HashMap, HashSet};
use x_bench_lib::*;

// ─────────────────────────────────────────────────────────────────────────────
// 1. Bloom filter — PreviouslySeenPostsFilter hot path.
//    Up to 50k seen IDs stored in bloom filter, 2000 candidates to check.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_bloom_filter(c: &mut Criterion) {
    let mut g = c.benchmark_group("pipeline/bloom_filter");

    // Build a realistic bloom filter with 50k entries
    let mut bf = BloomFilter::new(1_000_000, 5);
    for i in 0i64..50_000 {
        bf.insert(i * 13 + 7);
    }

    let candidates = make_candidates(2_000);

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_lookups", |b| {
        b.iter(|| {
            candidates.iter().filter(|c| {
                bf.may_contain(black_box(c.tweet_id as i64))
            }).count()
        })
    });

    g.throughput(Throughput::Elements(500));
    g.bench_function("500_lookups", |b| {
        b.iter(|| {
            candidates[..500].iter().filter(|c| {
                bf.may_contain(black_box(c.tweet_id as i64))
            }).count()
        })
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. DropDuplicatesFilter — HashSet insert, first occurrence wins.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_drop_duplicates(c: &mut Criterion) {
    let mut g = c.benchmark_group("pipeline/drop_duplicates");
    let mut candidates = make_candidates(2_000);
    // Inject ~10% duplicates (realistic when Thunder + Phoenix overlap)
    for i in 0..200 {
        candidates.push(candidates[i].clone());
    }

    g.throughput(Throughput::Elements(candidates.len() as u64));
    g.bench_function("2200_with_200_dupes", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                let mut seen: HashSet<u64> = HashSet::with_capacity(cands.len());
                let (kept, removed): (Vec<_>, Vec<_>) = cands.into_iter()
                    .partition(|c| seen.insert(c.tweet_id));
                black_box((kept.len(), removed.len()))
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. DedupConversationFilter — mirrors dedup_conversation_filter.rs.
//    Groups candidates by conversation_id, keeps highest score per thread.
//    HashMap<conversation_id, (index, best_score)>.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_dedup_conversation(c: &mut Criterion) {
    let mut g = c.benchmark_group("pipeline/dedup_conversation");

    let weights    = ScoringWeights::default();
    let mut candidates = make_candidates(2_000);

    // Assign scores and set ancestors so ~30% of candidates are replies
    for (i, cand) in candidates.iter_mut().enumerate() {
        cand.score = Some(compute_weighted_score(&weights, cand));
        if i % 3 == 0 {
            // Part of a conversation — ancestor is tweet_id of an earlier post
            cand.ancestors = vec![(i as u64).saturating_sub(1).max(1)];
        }
    }

    let dedup_conv = |mut cands: Vec<PostCandidate>| -> (Vec<PostCandidate>, Vec<PostCandidate>) {
        let mut kept: Vec<PostCandidate>    = Vec::new();
        let mut removed: Vec<PostCandidate> = Vec::new();
        let mut best_per_conv: HashMap<u64, (usize, f64)> = HashMap::new();

        for cand in cands.drain(..) {
            // conversation_id = min ancestor or tweet_id
            let conv_id = cand.ancestors.iter().copied().min().unwrap_or(cand.tweet_id);
            let score   = cand.score.unwrap_or(0.0);

            if let Some((kept_idx, best)) = best_per_conv.get_mut(&conv_id) {
                if score > *best {
                    let prev = std::mem::replace(&mut kept[*kept_idx], cand);
                    removed.push(prev);
                    *best = score;
                } else {
                    removed.push(cand);
                }
            } else {
                let idx = kept.len();
                best_per_conv.insert(conv_id, (idx, score));
                kept.push(cand);
            }
        }
        (kept, removed)
    };

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates_30pct_replies", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                let (kept, removed) = dedup_conv(cands);
                black_box((kept.len(), removed.len()))
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. AuthorSocialgraphFilter — HashSet lookups for blocked/muted authors.
//    Called on every candidate with viewer's block/mute lists.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_socialgraph_filter(c: &mut Criterion) {
    let mut g = c.benchmark_group("pipeline/socialgraph_filter");
    let candidates = make_candidates(2_000);

    // Realistic block/mute list sizes
    let blocked: HashSet<u64> = (1u64..=50).collect();
    let muted:   HashSet<u64> = (51u64..=200).collect();

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                let (kept, removed): (Vec<_>, Vec<_>) = cands.into_iter().partition(|c| {
                    !blocked.contains(&c.author_id) && !muted.contains(&c.author_id)
                });
                black_box((kept.len(), removed.len()))
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_bloom_filter,
    bench_drop_duplicates,
    bench_dedup_conversation,
    bench_socialgraph_filter,
);
criterion_main!(benches);
