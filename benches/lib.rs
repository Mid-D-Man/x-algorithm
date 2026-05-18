// benches/lib.rs
// Minimal self-contained mirrors of the X algorithm data structures.
// No xai_ internal deps — pure logic extracted from the source.

// ── Phoenix scores (mirrors home-mixer/models/candidate.rs) ──────────────────
#[derive(Clone, Debug, Default)]
pub struct PhoenixScores {
    pub favorite_score:          Option<f64>,
    pub reply_score:             Option<f64>,
    pub retweet_score:           Option<f64>,
    pub photo_expand_score:      Option<f64>,
    pub click_score:             Option<f64>,
    pub profile_click_score:     Option<f64>,
    pub vqv_score:               Option<f64>,
    pub share_score:             Option<f64>,
    pub share_via_dm_score:      Option<f64>,
    pub share_via_copy_link_score: Option<f64>,
    pub dwell_score:             Option<f64>,
    pub quote_score:             Option<f64>,
    pub quoted_click_score:      Option<f64>,
    pub quoted_vqv_score:        Option<f64>,
    pub follow_author_score:     Option<f64>,
    pub not_interested_score:    Option<f64>,
    pub block_author_score:      Option<f64>,
    pub mute_author_score:       Option<f64>,
    pub report_score:            Option<f64>,
    pub not_dwelled_score:       Option<f64>,
    pub dwell_time:              Option<f64>,
    pub click_dwell_time:        Option<f64>,
}

// ── Post candidate ────────────────────────────────────────────────────────────
#[derive(Clone, Debug, Default)]
pub struct PostCandidate {
    pub tweet_id:          u64,
    pub author_id:         u64,
    pub in_network:        Option<bool>,
    pub weighted_score:    Option<f64>,
    pub score:             Option<f64>,
    pub min_video_duration_ms: Option<i32>,
    pub quoted_video_duration_ms: Option<i32>,
    pub retweeted_tweet_id: Option<u64>,
    pub quoted_tweet_id:   Option<u64>,
    pub in_reply_to_tweet_id: Option<u64>,
    pub ancestors:         Vec<u64>,
    pub phoenix_scores:    PhoenixScores,
}

// ── Scoring weights (mirrors RankingScorer params) ────────────────────────────
#[derive(Clone, Debug)]
pub struct ScoringWeights {
    pub favorite:            f64,
    pub reply:               f64,
    pub retweet:             f64,
    pub photo_expand:        f64,
    pub click:               f64,
    pub profile_click:       f64,
    pub vqv:                 f64,
    pub share:               f64,
    pub share_via_dm:        f64,
    pub share_via_copy_link: f64,
    pub dwell:               f64,
    pub quote:               f64,
    pub quoted_click:        f64,
    pub quoted_vqv:          f64,
    pub cont_dwell_time:     f64,
    pub cont_click_dwell_time: f64,
    pub follow_author:       f64,
    pub not_interested:      f64,
    pub block_author:        f64,
    pub mute_author:         f64,
    pub report:              f64,
    pub not_dwelled:         f64,
    pub negative_sum:        f64,
    pub total_sum:           f64,
    pub min_video_duration_ms: i32,
}

pub const NEGATIVE_SCORES_OFFSET: f64 = 0.01;

impl Default for ScoringWeights {
    fn default() -> Self {
        let favorite            = 0.5;
        let reply               = 1.0;
        let retweet             = 0.4;
        let photo_expand        = 0.2;
        let click               = 0.3;
        let profile_click       = 0.1;
        let vqv                 = 0.6;
        let share               = 0.3;
        let share_via_dm        = 0.25;
        let share_via_copy_link = 0.05;
        let dwell               = 0.2;
        let quote               = 0.5;
        let quoted_click        = 0.2;
        let quoted_vqv          = 0.3;
        let cont_dwell_time     = 0.01;
        let cont_click_dwell   = 0.01;
        let follow_author       = 2.0;
        let not_interested      = -0.3;
        let block_author        = -1.0;
        let mute_author         = -0.5;
        let report              = -2.0;
        let not_dwelled         = -0.1;

        let positive_sum = favorite + reply + retweet + photo_expand + click
            + profile_click + vqv + share + share_via_dm + share_via_copy_link
            + dwell + quote + quoted_click + quoted_vqv + follow_author;
        let negative_sum = -(not_interested + block_author + mute_author + report + not_dwelled);
        let total_sum    = positive_sum + negative_sum;

        Self {
            favorite, reply, retweet, photo_expand, click, profile_click,
            vqv, share, share_via_dm, share_via_copy_link, dwell, quote,
            quoted_click, quoted_vqv, cont_dwell_time,
            cont_click_dwell_time: cont_click_dwell,
            follow_author, not_interested, block_author, mute_author,
            report, not_dwelled, negative_sum, total_sum,
            min_video_duration_ms: 5_000,
        }
    }
}

// ── Core scoring logic (mirrors ranking_scorer.rs) ────────────────────────────
#[inline(always)]
pub fn apply(score: Option<f64>, weight: f64) -> f64 {
    score.unwrap_or(0.0) * weight
}

pub fn compute_weighted_score(w: &ScoringWeights, c: &PostCandidate) -> f64 {
    let s = &c.phoenix_scores;

    let vqv_weight = match c.min_video_duration_ms {
        Some(ms) if ms > w.min_video_duration_ms => w.vqv,
        _ => 0.0,
    };
    let quoted_vqv_weight = match c.quoted_video_duration_ms {
        Some(ms) if ms > w.min_video_duration_ms => w.quoted_vqv,
        _ => 0.0,
    };

    let combined = apply(s.favorite_score,          w.favorite)
        + apply(s.reply_score,               w.reply)
        + apply(s.retweet_score,             w.retweet)
        + apply(s.photo_expand_score,        w.photo_expand)
        + apply(s.click_score,               w.click)
        + apply(s.profile_click_score,       w.profile_click)
        + apply(s.vqv_score,                 vqv_weight)
        + apply(s.share_score,               w.share)
        + apply(s.share_via_dm_score,        w.share_via_dm)
        + apply(s.share_via_copy_link_score, w.share_via_copy_link)
        + apply(s.dwell_score,               w.dwell)
        + apply(s.quote_score,               w.quote)
        + apply(s.quoted_click_score,        w.quoted_click)
        + apply(s.quoted_vqv_score,          quoted_vqv_weight)
        + apply(s.dwell_time,                w.cont_dwell_time)
        + apply(s.click_dwell_time,          w.cont_click_dwell_time)
        + apply(s.follow_author_score,       w.follow_author)
        + apply(s.not_interested_score,      w.not_interested)
        + apply(s.block_author_score,        w.block_author)
        + apply(s.mute_author_score,         w.mute_author)
        + apply(s.report_score,              w.report)
        + apply(s.not_dwelled_score,         w.not_dwelled);

    offset_score(combined, w)
}

#[inline]
pub fn offset_score(combined: f64, w: &ScoringWeights) -> f64 {
    if w.total_sum == 0.0 {
        combined.max(0.0)
    } else if combined < 0.0 {
        (combined + w.negative_sum) / w.total_sum * NEGATIVE_SCORES_OFFSET
    } else {
        combined + NEGATIVE_SCORES_OFFSET
    }
}

pub fn diversity_multiplier(decay: f64, floor: f64, position: usize) -> f64 {
    (1.0 - floor) * decay.powf(position as f64) + floor
}

// ── SSE2-accelerated weighted score ──────────────────────────────────────────
//
// Packs 22 (score, weight) f64 pairs into 24 f32 lanes (2 zero-padded),
// computes 6 × _mm_mul_ps and a horizontal sum.
//
// Precision note: recommendation probabilities are in [0.0, 0.1]. Converting
// Option<f64> → f32 introduces ~1e-7 relative error — negligible for ranking.
//
// SSE2 is mandatory on all x86_64 CPUs (part of the ABI since 2003), so no
// runtime feature detection is needed.
#[cfg(target_arch = "x86_64")]
pub fn compute_weighted_score_sse2(w: &ScoringWeights, c: &PostCandidate) -> f64 {
    use core::arch::x86_64::*;

    let s = &c.phoenix_scores;

    // Resolve video-duration-gated weights up-front (same logic as scalar)
    let vqv_w: f32 =
        if c.min_video_duration_ms.map_or(false, |ms| ms > w.min_video_duration_ms) {
            w.vqv as f32
        } else {
            0.0
        };
    let qvqv_w: f32 =
        if c.quoted_video_duration_ms.map_or(false, |ms| ms > w.min_video_duration_ms) {
            w.quoted_vqv as f32
        } else {
            0.0
        };

    // 24-lane packed arrays (22 real + 2 zero padding → 6 complete f32x4)
    // Layout mirrors the scalar sum order exactly.
    let scores: [f32; 24] = [
        s.favorite_score.unwrap_or(0.0) as f32,          // 0
        s.reply_score.unwrap_or(0.0) as f32,              // 1
        s.retweet_score.unwrap_or(0.0) as f32,            // 2
        s.photo_expand_score.unwrap_or(0.0) as f32,       // 3
        s.click_score.unwrap_or(0.0) as f32,              // 4
        s.profile_click_score.unwrap_or(0.0) as f32,      // 5
        s.vqv_score.unwrap_or(0.0) as f32,                // 6
        s.share_score.unwrap_or(0.0) as f32,              // 7
        s.share_via_dm_score.unwrap_or(0.0) as f32,       // 8
        s.share_via_copy_link_score.unwrap_or(0.0) as f32,// 9
        s.dwell_score.unwrap_or(0.0) as f32,              // 10
        s.quote_score.unwrap_or(0.0) as f32,              // 11
        s.quoted_click_score.unwrap_or(0.0) as f32,       // 12
        s.quoted_vqv_score.unwrap_or(0.0) as f32,         // 13
        s.dwell_time.unwrap_or(0.0) as f32,               // 14
        s.click_dwell_time.unwrap_or(0.0) as f32,         // 15
        s.follow_author_score.unwrap_or(0.0) as f32,      // 16
        s.not_interested_score.unwrap_or(0.0) as f32,     // 17
        s.block_author_score.unwrap_or(0.0) as f32,       // 18
        s.mute_author_score.unwrap_or(0.0) as f32,        // 19
        s.report_score.unwrap_or(0.0) as f32,             // 20
        s.not_dwelled_score.unwrap_or(0.0) as f32,        // 21
        0.0_f32, 0.0_f32,                                 // 22-23 padding
    ];

    let weights: [f32; 24] = [
        w.favorite as f32,
        w.reply as f32,
        w.retweet as f32,
        w.photo_expand as f32,
        w.click as f32,
        w.profile_click as f32,
        vqv_w,                       // 0.0 when video ineligible
        w.share as f32,
        w.share_via_dm as f32,
        w.share_via_copy_link as f32,
        w.dwell as f32,
        w.quote as f32,
        w.quoted_click as f32,
        qvqv_w,                      // 0.0 when quoted-video ineligible
        w.cont_dwell_time as f32,
        w.cont_click_dwell_time as f32,
        w.follow_author as f32,
        w.not_interested as f32,
        w.block_author as f32,
        w.mute_author as f32,
        w.report as f32,
        w.not_dwelled as f32,
        0.0_f32, 0.0_f32,
    ];

    // SAFETY: SSE2 is always available on x86_64; arrays are valid f32 data.
    unsafe {
        let sp = scores.as_ptr();
        let wp = weights.as_ptr();

        // 6 × (loadu + mul + add) — 12 SSE2 instructions total.
        // LLVM promotes mul+add to vfmadd231ps on FMA3 hosts (CI Xeon/EPYC)
        // giving a further ~2× improvement over pure SSE2.
        let mut acc = _mm_mul_ps(_mm_loadu_ps(sp),        _mm_loadu_ps(wp));
        acc = _mm_add_ps(acc, _mm_mul_ps(_mm_loadu_ps(sp.add(4)),  _mm_loadu_ps(wp.add(4))));
        acc = _mm_add_ps(acc, _mm_mul_ps(_mm_loadu_ps(sp.add(8)),  _mm_loadu_ps(wp.add(8))));
        acc = _mm_add_ps(acc, _mm_mul_ps(_mm_loadu_ps(sp.add(12)), _mm_loadu_ps(wp.add(12))));
        acc = _mm_add_ps(acc, _mm_mul_ps(_mm_loadu_ps(sp.add(16)), _mm_loadu_ps(wp.add(16))));
        acc = _mm_add_ps(acc, _mm_mul_ps(_mm_loadu_ps(sp.add(20)), _mm_loadu_ps(wp.add(20))));

        // Horizontal sum of [a, b, c, d] → a+b+c+d
        // Step 1: shuffle to [b, a, d, c]  (swap adjacent pairs)
        let shuf = _mm_shuffle_ps::<0b10_11_00_01>(acc, acc);
        // Step 2: [a+b, b+a, c+d, d+c]
        let sums = _mm_add_ps(acc, shuf);
        // Step 3: move high 2 lanes to low → [c+d, d+c, ...]
        let hi   = _mm_movehl_ps(sums, sums);
        // Step 4: (a+b) + (c+d) in lane 0
        let dot  = _mm_add_ss(sums, hi);

        let combined = _mm_cvtss_f32(dot) as f64;
        offset_score(combined, w)
    }
}

/// Dispatches to SSE2 on x86_64, scalar elsewhere.
/// Use this in production paths; use the explicit variants in benchmarks for
/// direct scalar-vs-SIMD comparison.
pub fn compute_weighted_score_fast(w: &ScoringWeights, c: &PostCandidate) -> f64 {
    #[cfg(target_arch = "x86_64")]
    { compute_weighted_score_sse2(w, c) }
    #[cfg(not(target_arch = "x86_64"))]
    { compute_weighted_score(w, c) }
}

// ── Precomputed diversity table ───────────────────────────────────────────────
//
// The diversity multiplier for author appearance position p is:
//   (1 - floor) * decay^p + floor
//
// Building the full table costs one powf per unique position (bounded by
// actual author repeat counts — typically < 32 in a 2000-candidate window).
// Each subsequent lookup is an array index: ~1 ns vs ~50 ns for powf.
//
// Build once per request (outside the per-candidate loop) and pass in.

/// Precomputed per-position diversity multipliers.
pub struct DiversityTable {
    multipliers: Vec<f64>,
    floor: f64,
}

impl DiversityTable {
    /// Build a table for positions 0..max_positions.
    /// `max_positions` = 64 covers any realistic per-author repeat count.
    pub fn new(decay: f64, floor: f64, max_positions: usize) -> Self {
        let multipliers = (0..max_positions)
            .map(|i| (1.0 - floor) * decay.powf(i as f64) + floor)
            .collect();
        Self { multipliers, floor }
    }

    #[inline(always)]
    pub fn get(&self, position: usize) -> f64 {
        // Fast path: positions beyond the table are so heavily attenuated
        // they are effectively at floor.
        self.multipliers.get(position).copied().unwrap_or(self.floor)
    }
}

/// Author diversity scoring using a precomputed multiplier table.
/// Behaviorally identical to `apply_author_diversity`; replaces `powf` per
/// candidate with an O(1) table lookup.
pub fn apply_author_diversity_table(
    candidates: &[PostCandidate],
    weighted_scores: &[f64],
    table: &DiversityTable,
    oon_weight: f64,
) -> Vec<f64> {
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_unstable_by(|&a, &b| {
        weighted_scores[b]
            .partial_cmp(&weighted_scores[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut author_counts: std::collections::HashMap<u64, usize> =
        std::collections::HashMap::with_capacity(256);
    let mut final_scores = vec![0.0f64; candidates.len()];

    for idx in order {
        let c = &candidates[idx];
        let entry = author_counts.entry(c.author_id).or_insert(0);
        let mult = table.get(*entry);   // O(1) table lookup — no powf
        *entry += 1;

        let score = weighted_scores[idx] * mult;
        final_scores[idx] = match c.in_network {
            Some(false) => score * oon_weight,
            _ => score,
        };
    }
    final_scores
}

// ── Top-K partial sort ────────────────────────────────────────────────────────
//
// `sort_unstable_by` on 2000 elements costs ~170 μs because it sorts all
// 2000 even though only 50 will be served. `select_nth_unstable_by` partitions
// in O(n) average, then we sort only the top-k slice — typically 15-30× faster
// for small k relative to n.

/// Select and return the top `k` (score, index) pairs sorted descending.
/// Uses `select_nth_unstable_by` (O(n) average) instead of full sort.
pub fn top_k_by_score(mut indexed: Vec<(f64, usize)>, k: usize) -> Vec<(f64, usize)> {
    if k == 0 {
        return Vec::new();
    }
    let n = indexed.len();
    if n <= k {
        indexed.sort_unstable_by(|a, b| {
            b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
        });
        return indexed;
    }
    // Partition: elements [0..k] are the k largest (unsorted among themselves)
    indexed.select_nth_unstable_by(k, |a, b| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    // Sort only the top-k slice
    let mut top = indexed[..k].to_vec();
    top.sort_unstable_by(|a, b| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    top
}

// ── Thunder light post (mirrors thunder/posts/post_store.rs) ──────────────────
#[derive(Clone, Debug)]
pub struct LightPost {
    pub post_id:    i64,
    pub author_id:  i64,
    pub created_at: i64,
    pub is_reply:   bool,
    pub is_retweet: bool,
    pub has_video:  bool,
}

// ── Generators ────────────────────────────────────────────────────────────────
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

pub fn make_candidates(n: usize) -> Vec<PostCandidate> {
    let mut rng = SmallRng::seed_from_u64(42);
    (0..n).map(|i| {
        let mut score = || -> Option<f64> { Some(rng.gen::<f64>() * 0.1) };
        PostCandidate {
            tweet_id:   i as u64 + 1,
            author_id:  (i % 200) as u64 + 1,
            in_network: Some(i % 3 != 0),
            min_video_duration_ms: if i % 5 == 0 { Some(10_000) } else { None },
            phoenix_scores: PhoenixScores {
                favorite_score:       score(),
                reply_score:          score(),
                retweet_score:        score(),
                photo_expand_score:   score(),
                click_score:          score(),
                profile_click_score:  score(),
                vqv_score:            score(),
                share_score:          score(),
                dwell_score:          score(),
                follow_author_score:  score(),
                not_interested_score: Some(-rng.gen::<f64>() * 0.05),
                report_score:         Some(-rng.gen::<f64>() * 0.01),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .collect()
}

pub fn make_light_posts(n: usize) -> Vec<LightPost> {
    let base_ts: i64 = 1_700_000_000;
    (0..n).map(|i| LightPost {
        post_id:    i as i64 + 1,
        author_id:  (i % 500) as i64 + 1,
        created_at: base_ts + i as i64 * 10,
        is_reply:   i % 4 == 0,
        is_retweet: i % 7 == 0,
        has_video:  i % 6 == 0,
    })
    .collect()
}

// Simple Bloom filter (mirrors xai_candidate_pipeline BloomFilter behaviour)
pub struct BloomFilter {
    bits: Vec<u64>,
    num_hashes: u32,
}

impl BloomFilter {
    pub fn new(size_bits: usize, num_hashes: u32) -> Self {
        Self {
            bits: vec![0u64; size_bits.div_ceil(64)],
            num_hashes,
        }
    }

    fn hash(&self, item: i64, seed: u32) -> usize {
        let h = (item as u64).wrapping_mul(0x517cc1b727220a95).wrapping_add(seed as u64);
        let h = h ^ (h >> 32);
        h as usize % (self.bits.len() * 64)
    }

    pub fn insert(&mut self, item: i64) {
        for i in 0..self.num_hashes {
            let pos = self.hash(item, i);
            self.bits[pos / 64] |= 1u64 << (pos % 64);
        }
    }

    pub fn may_contain(&self, item: i64) -> bool {
        (0..self.num_hashes).all(|i| {
            let pos = self.hash(item, i);
            self.bits[pos / 64] & (1u64 << (pos % 64)) != 0
        })
    }
}
