// benches/scoring_bench.rs
// Benchmarks for the scoring and ranking hot paths.
// Includes scalar baselines and optimised (SSE2 + table) variants side-by-side.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::collections::HashMap;
use x_bench_lib::*;

// ─────────────────────────────────────────────────────────────────────────────
// 1. Per-candidate weighted score — scalar baseline.
//    22 floating-point multiply-adds per call.
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
// 2. Per-candidate weighted score — SSE2 f32x4 variant.
//    6 SIMD mul+add vs 22 scalar. Scores widened f64→f32.
//    Compare directly against scoring/weighted_score/single_candidate.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_weighted_score_simd(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/weighted_score_simd");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);

    g.bench_function("single_candidate", |b| {
        let cand = &candidates[0];
        b.iter(|| compute_weighted_score_fast(black_box(&weights), black_box(cand)))
    });

    g.throughput(Throughput::Elements(500));
    g.bench_function("batch_500", |b| {
        b.iter_batched(
            || candidates[..500].to_vec(),
            |batch| {
                batch.iter()
                    .map(|c| compute_weighted_score_fast(&weights, black_box(c)))
                    .sum::<f64>()
            },
            BatchSize::LargeInput,
        )
    });

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("batch_2000", |b| {
        b.iter_batched(
            || candidates.clone(),
            |batch| {
                batch.iter()
                    .map(|c| compute_weighted_score_fast(&weights, black_box(c)))
                    .sum::<f64>()
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Author diversity — scalar baseline (powf per candidate).
// ─────────────────────────────────────────────────────────────────────────────

fn apply_author_diversity(
    candidates: &[PostCandidate],
    weighted_scores: &[f64],
    decay: f64,
    floor: f64,
    oon_weight: f64,
) -> Vec<f64> {
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
        let mult  = diversity_multiplier(decay, floor, *entry);  // powf here
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
                black_box(&cands), black_box(&sc), 0.85, 0.1, 0.5,
            ),
            BatchSize::LargeInput,
        )
    });

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates", |b| {
        b.iter_batched(
            || (candidates.clone(), scores.clone()),
            |(cands, sc)| apply_author_diversity(
                black_box(&cands), black_box(&sc), 0.85, 0.1, 0.5,
            ),
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Author diversity — precomputed table variant.
//    Table built once, lookup replaces powf per candidate.
//    Compare directly against scoring/author_diversity/N_candidates.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_author_diversity_table(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/author_diversity_table");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);
    let scores: Vec<f64> = candidates.iter()
        .map(|c| compute_weighted_score(&weights, c))
        .collect();

    // Build table once — this is done outside the measured loop (amortised
    // across the full request, not per-candidate). Max 64 positions covers
    // any realistic per-author repeat count within a 2000-candidate window.
    let table = DiversityTable::new(0.85, 0.1, 64);

    g.throughput(Throughput::Elements(500));
    g.bench_function("500_candidates", |b| {
        b.iter_batched(
            || (candidates[..500].to_vec(), scores[..500].to_vec()),
            |(cands, sc)| apply_author_diversity_table(
                black_box(&cands), black_box(&sc), &table, 0.5,
            ),
            BatchSize::LargeInput,
        )
    });

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates", |b| {
        b.iter_batched(
            || (candidates.clone(), scores.clone()),
            |(cands, sc)| apply_author_diversity_table(
                black_box(&cands), black_box(&sc), &table, 0.5,
            ),
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Offset score — called per-candidate, vectorizable.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_offset_score(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/offset_score");
    let weights = ScoringWeights::default();

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
// 6. Sort candidates by final score — full sort (selector stage baseline).
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
// 7. Top-K partial sort — select_nth_unstable_by (O(n) average).
//    Compare against scoring/sort_candidates/sort_2000 for the common case
//    of 2000 candidates → top 50.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_top_k_select(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/top_k_select");
    let candidates = make_candidates(2_000);
    let weights    = ScoringWeights::default();

    let indexed: Vec<(f64, usize)> = candidates.iter()
        .enumerate()
        .map(|(i, c)| (compute_weighted_score(&weights, c), i))
        .collect();

    // k=50 matches the full_pipeline benchmark (2000 → top 50)
    for k in [50usize, 100, 200] {
        g.throughput(Throughput::Elements(2_000));
        g.bench_function(format!("2000_to_top_{k}"), |b| {
            b.iter_batched(
                || indexed.clone(),
                |v| black_box(top_k_by_score(v, k)),
                BatchSize::LargeInput,
            )
        });
    }

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Full scoring pipeline — end-to-end per request.
//    Baseline: scalar weighted score + powf diversity + full sort → top 50.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_full_scoring_pipeline(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/full_pipeline");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);

    // Baseline: all-scalar
    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates_to_top50", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                let weighted: Vec<f64> = cands.iter()
                    .map(|c| compute_weighted_score(&weights, c))
                    .collect();

                let final_scores = apply_author_diversity(&cands, &weighted, 0.85, 0.1, 0.5);

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

// ─────────────────────────────────────────────────────────────────────────────
// 9. Full pipeline — optimised variant.
//    SSE2 weighted score + precomputed table diversity + partial sort top-K.
//    Compare against scoring/full_pipeline/2000_candidates_to_top50.
// ─────────────────────────────────────────────────────────────────────────────

fn bench_full_pipeline_optimised(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/full_pipeline_optimised");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);
    // Table is built once per request in production; here it's built once
    // for the entire benchmark run (same amortisation).
    let div_table  = DiversityTable::new(0.85, 0.1, 64);

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates_to_top50", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                // Stage 1: SSE2 weighted score (f32x4 on x86_64)
                let weighted: Vec<f64> = cands.iter()
                    .map(|c| compute_weighted_score_fast(&weights, c))
                    .collect();

                // Stage 2: table-based diversity (no powf)
                let final_scores =
                    apply_author_diversity_table(&cands, &weighted, &div_table, 0.5);

                // Stage 3: partial sort → top 50 (O(n) average instead of O(n log n))
                let indexed: Vec<(f64, usize)> = final_scores.iter()
                    .enumerate()
                    .map(|(i, &s)| (s, i))
                    .collect();
                black_box(top_k_by_score(indexed, 50))
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

criterion_group!(
    benches,
    bench_weighted_score,
    bench_weighted_score_simd,
    bench_author_diversity,
    bench_author_diversity_table,
    bench_offset_score,
    bench_candidate_sort,
    bench_top_k_select,
    bench_full_scoring_pipeline,
    bench_full_pipeline_optimised,
);
criterion_main!(benches);
