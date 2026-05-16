// benches/scoring_bench.rs
// Benchmarks for the scoring and ranking hot paths.
// These are the float-heavy operations most likely improvable with mid-math SIMD.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::collections::HashMap;
use x_bench_lib::*;

// ─────────────────────────────────────────────────────────────────────────────
// 1. Per-candidate weighted score — THE hottest path in the whole pipeline.
//    Called once per candidate per request. 500–2000 candidates typical.
//    22 floating-point multiply-adds per call.
//    TARGET for mid-math SIMD: batch 4 or 8 candidates simultaneously.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_weighted_score(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/weighted_score");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);

    // Single candidate — baseline
    g.bench_function("single_candidate", |b| {
        let cand = &candidates[0];
        b.iter(|| compute_weighted_score(black_box(&weights), black_box(cand)))
    });

    // Batch of 500 — realistic feed request size
    g.throughput(Throughput::Elements(500));
    g.bench_function("batch_500", |b| {
        b.iter_batched(
            || candidates[..500].to_vec(),
            |batch| {
                batch.iter()
                    .map(|c| compute_weighted_score(&weights, black_box(c)))
                    .sum::<f64>()
            },
            BatchSize::LargeInput,
        )
    });

    // Batch of 2000 — large request with retrieval
    g.throughput(Throughput::Elements(2_000));
    g.bench_function("batch_2000", |b| {
        b.iter_batched(
            || candidates.clone(),
            |batch| {
                batch.iter()
                    .map(|c| compute_weighted_score(&weights, black_box(c)))
                    .sum::<f64>()
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Author diversity attenuation — mirrors RankingScorer::apply_author_diversity.
//    Sort by score → walk in order → decay repeated authors.
//    HashMap per request: author_id → appearance_count.
//    TARGET: the powf() per candidate is expensive when decay < 1.0.
// ─────────────────────────────────────────────────────────────────────────────

fn apply_author_diversity(
    candidates: &[PostCandidate],
    weighted_scores: &[f64],
    decay: f64,
    floor: f64,
    oon_weight: f64,
) -> Vec<f64> {
    // Sort indices by descending weighted score
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_unstable_by(|&a, &b| {
        weighted_scores[b].partial_cmp(&weighted_scores[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut author_counts: HashMap<u64, usize> = HashMap::with_capacity(256);
    let mut final_scores = vec![0.0f64; candidates.len()];

    for idx in order {
        let c     = &candidates[idx];
        let entry = author_counts.entry(c.author_id).or_insert(0);
        let mult  = diversity_multiplier(decay, floor, *entry);
        *entry   += 1;

        let score = weighted_scores[idx] * mult;
        final_scores[idx] = match c.in_network {
            Some(false) => score * oon_weight,
            _           => score,
        };
    }
    final_scores
}

fn bench_author_diversity(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/author_diversity");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);
    let scores: Vec<f64> = candidates.iter()
        .map(|c| compute_weighted_score(&weights, c))
        .collect();

    g.throughput(Throughput::Elements(500));
    g.bench_function("500_candidates", |b| {
        b.iter_batched(
            || (candidates[..500].to_vec(), scores[..500].to_vec()),
            |(cands, sc)| apply_author_diversity(
                black_box(&cands),
                black_box(&sc),
                0.85, 0.1, 0.5,
            ),
            BatchSize::LargeInput,
        )
    });

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates", |b| {
        b.iter_batched(
            || (candidates.clone(), scores.clone()),
            |(cands, sc)| apply_author_diversity(
                black_box(&cands),
                black_box(&sc),
                0.85, 0.1, 0.5,
            ),
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Offset score — tiny but called per-candidate, vectorizable.
//    Branching on negative vs positive score. SIMD could remove the branch.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_offset_score(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/offset_score");
    let weights = ScoringWeights::default();

    // Mix of positive and negative scores
    let scores: Vec<f64> = (0..1000)
        .map(|i| if i % 3 == 0 { -(i as f64 * 0.001) } else { i as f64 * 0.002 })
        .collect();

    g.throughput(Throughput::Elements(1_000));
    g.bench_function("1000_scores", |b| {
        b.iter_batched(
            || scores.clone(),
            |sc| sc.iter().map(|&s| offset_score(black_box(s), &weights)).sum::<f64>(),
            BatchSize::SmallInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Sort candidates by final score — selector stage.
//    Unstable sort on 500–2000 f64 scores.
//    This runs once per request after all scoring.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_candidate_sort(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/sort_candidates");
    let candidates = make_candidates(2_000);
    let weights    = ScoringWeights::default();

    let with_scores: Vec<(f64, PostCandidate)> = candidates.iter()
        .map(|c| (compute_weighted_score(&weights, c), c.clone()))
        .collect();

    for n in [500usize, 1_000, 2_000] {
        g.throughput(Throughput::Elements(n as u64));
        g.bench_function(format!("sort_{n}"), |b| {
            b.iter_batched(
                || with_scores[..n].to_vec(),
                |mut v| {
                    v.sort_unstable_by(|a, b| {
                        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    black_box(v)
                },
                BatchSize::LargeInput,
            )
        });
    }

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Full scoring pipeline simulation (score → diversity → sort → select top-K)
//    End-to-end per request. This is what users experience as feed latency.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_full_scoring_pipeline(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/full_pipeline");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates_to_top50", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                // Stage 1: weighted score
                let weighted: Vec<f64> = cands.iter()
                    .map(|c| compute_weighted_score(&weights, c))
                    .collect();

                // Stage 2: author diversity + OON adjustment
                let final_scores = apply_author_diversity(&cands, &weighted, 0.85, 0.1, 0.5);

                // Stage 3: sort and select top 50
                let mut indexed: Vec<(f64, usize)> = final_scores.iter()
                    .enumerate()
                    .map(|(i, &s)| (s, i))
                    .collect();
                indexed.sort_unstable_by(|a, b| {
                    b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
                });
                indexed.truncate(50);
                black_box(indexed)
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_weighted_score,
    bench_author_diversity,
    bench_offset_score,
    bench_candidate_sort,
    bench_full_scoring_pipeline,
);
criterion_main!(benches);
