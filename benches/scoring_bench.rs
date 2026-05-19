// benches/scoring_bench.rs
// Benchmarks for the scoring and ranking hot paths.
// Scalar baseline · AoS f32 (comparison) · SoA batch · optimised full pipeline.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::collections::HashMap;
use x_bench_lib::*;

// ─────────────────────────────────────────────────────────────────────────────
// 1. Per-candidate weighted score — scalar f64 baseline.
// ─────────────────────────────────────────────────────────────────────────────
fn bench_weighted_score(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/weighted_score");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);

    g.bench_function("single_candidate", |b| {
        let cand = &candidates[0];
        b.iter(|| compute_weighted_score(black_box(&weights), black_box(cand)))
    });

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
// 2. AoS f32 variant — kept to show why it LOST to scalar (2.3× slower).
//    The f64→f32 conversion + per-candidate array overhead dominates.
// ─────────────────────────────────────────────────────────────────────────────
fn bench_weighted_score_aos_f32(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/weighted_score_aos_f32");
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
// 3. SoA batch score — 22 fma_pass calls, each 8-wide AVX2 across candidates.
//    Measures ONLY the scoring step (SoA already built — amortised cost).
// ─────────────────────────────────────────────────────────────────────────────
fn bench_weighted_score_soa(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/weighted_score_soa");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);

    let soa_500  = PhoenixScoresSoA::from_candidates(&candidates[..500], &weights);
    let soa_2000 = PhoenixScoresSoA::from_candidates(&candidates,        &weights);

    g.throughput(Throughput::Elements(500));
    g.bench_function("batch_500", |b| {
        b.iter(|| {
            black_box(compute_batch_weighted_scores_soa(
                black_box(&weights),
                black_box(&soa_500),
            ))
        })
    });

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("batch_2000", |b| {
        b.iter(|| {
            black_box(compute_batch_weighted_scores_soa(
                black_box(&weights),
                black_box(&soa_2000),
            ))
        })
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. SoA build + score combined — true end-to-end cost including scatter.
//    Tells us whether the construction overhead amortises over the scoring win.
// ─────────────────────────────────────────────────────────────────────────────
fn bench_weighted_score_soa_with_build(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/weighted_score_soa_with_build");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);

    g.throughput(Throughput::Elements(500));
    g.bench_function("batch_500", |b| {
        b.iter_batched(
            || candidates[..500].to_vec(),
            |cands| {
                let soa = PhoenixScoresSoA::from_candidates(black_box(&cands), &weights);
                black_box(compute_batch_weighted_scores_soa(&weights, &soa))
            },
            BatchSize::LargeInput,
        )
    });

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("batch_2000", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                let soa = PhoenixScoresSoA::from_candidates(black_box(&cands), &weights);
                black_box(compute_batch_weighted_scores_soa(&weights, &soa))
            },
            BatchSize::LargeInput,
        )
    });

    g.finish();
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. Author diversity — scalar baseline (powf per candidate).
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
// 6. Author diversity — precomputed table variant.
// ─────────────────────────────────────────────────────────────────────────────
fn bench_author_diversity_table(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/author_diversity_table");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);
    let scores: Vec<f64> = candidates.iter()
        .map(|c| compute_weighted_score(&weights, c))
        .collect();
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
// 7. Offset score — called per-candidate, vectorizable.
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
// 8. Sort candidates — full sort baseline (selector stage).
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
// 9. Top-K partial sort.
// ─────────────────────────────────────────────────────────────────────────────
fn bench_top_k_select(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/top_k_select");
    let candidates = make_candidates(2_000);
    let weights    = ScoringWeights::default();

    let indexed: Vec<(f64, usize)> = candidates.iter()
        .enumerate()
        .map(|(i, c)| (compute_weighted_score(&weights, c), i))
        .collect();

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
// 10. Full pipeline — scalar baseline (weighted score + powf diversity + sort).
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
// 11. Full pipeline — table diversity + partial sort (previous best).
// ─────────────────────────────────────────────────────────────────────────────
fn bench_full_pipeline_optimised(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/full_pipeline_optimised");
    let weights   = ScoringWeights::default();
    let candidates = make_candidates(2_000);
    let div_table  = DiversityTable::new(0.85, 0.1, 64);

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates_to_top50", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                let weighted: Vec<f64> = cands.iter()
                    .map(|c| compute_weighted_score_fast(&weights, c))
                    .collect();
                let final_scores =
                    apply_author_diversity_table(&cands, &weighted, &div_table, 0.5);
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

// ─────────────────────────────────────────────────────────────────────────────
// 12. Full pipeline — SoA weighted score + table diversity + partial sort.
//     The target: SoA scoring replaces the per-candidate scalar loop.
//     Includes SoA construction cost (one scatter pass = amortised reality).
// ─────────────────────────────────────────────────────────────────────────────
fn bench_full_pipeline_soa(c: &mut Criterion) {
    let mut g = c.benchmark_group("scoring/full_pipeline_soa");
    let weights    = ScoringWeights::default();
    let candidates = make_candidates(2_000);
    let div_table  = DiversityTable::new(0.85, 0.1, 64);

    g.throughput(Throughput::Elements(2_000));
    g.bench_function("2000_candidates_to_top50", |b| {
        b.iter_batched(
            || candidates.clone(),
            |cands| {
                // Stage 1: scatter AoS → SoA, score 8 candidates/instruction
                let soa      = PhoenixScoresSoA::from_candidates(&cands, &weights);
                let raw      = compute_batch_weighted_scores_soa(&weights, &soa);
                let weighted = apply_offset_scores(&raw, &weights);

                // Stage 2: table-based diversity (no powf)
                let final_scores =
                    apply_author_diversity_table(&cands, &weighted, &div_table, 0.5);

                // Stage 3: partial sort → top 50 (O(n) average)
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
    bench_weighted_score_aos_f32,
    bench_weighted_score_soa,
    bench_weighted_score_soa_with_build,
    bench_author_diversity,
    bench_author_diversity_table,
    bench_offset_score,
    bench_candidate_sort,
    bench_top_k_select,
    bench_full_scoring_pipeline,
    bench_full_pipeline_optimised,
    bench_full_pipeline_soa,
);
criterion_main!(benches);
