// benches/lib.rs
// Minimal self-contained mirrors of the X algorithm data structures.
// No xai_ internal deps — pure logic extracted from the source.

// ── Phoenix scores (mirrors home-mixer/models/candidate.rs) ──────────────────
#[derive(Clone, Debug, Default)]
pub struct PhoenixScores {
    pub favorite_score:            Option<f64>,
    pub reply_score:               Option<f64>,
    pub retweet_score:             Option<f64>,
    pub photo_expand_score:        Option<f64>,
    pub click_score:               Option<f64>,
    pub profile_click_score:       Option<f64>,
    pub vqv_score:                 Option<f64>,
    pub share_score:               Option<f64>,
    pub share_via_dm_score:        Option<f64>,
    pub share_via_copy_link_score: Option<f64>,
    pub dwell_score:               Option<f64>,
    pub quote_score:               Option<f64>,
    pub quoted_click_score:        Option<f64>,
    pub quoted_vqv_score:          Option<f64>,
    pub follow_author_score:       Option<f64>,
    pub not_interested_score:      Option<f64>,
    pub block_author_score:        Option<f64>,
    pub mute_author_score:         Option<f64>,
    pub report_score:              Option<f64>,
    pub not_dwelled_score:         Option<f64>,
    pub dwell_time:                Option<f64>,
    pub click_dwell_time:          Option<f64>,
}

// ── Post candidate ────────────────────────────────────────────────────────────
#[derive(Clone, Debug, Default)]
pub struct PostCandidate {
    pub tweet_id:                 u64,
    pub author_id:                u64,
    pub in_network:               Option<bool>,
    pub weighted_score:           Option<f64>,
    pub score:                    Option<f64>,
    pub min_video_duration_ms:    Option<i32>,
    pub quoted_video_duration_ms: Option<i32>,
    pub retweeted_tweet_id:       Option<u64>,
    pub quoted_tweet_id:          Option<u64>,
    pub in_reply_to_tweet_id:     Option<u64>,
    pub ancestors:                Vec<u64>,
    pub phoenix_scores:           PhoenixScores,
}

// ── Scoring weights (mirrors RankingScorer params) ────────────────────────────
#[derive(Clone, Debug)]
pub struct ScoringWeights {
    pub favorite:              f64,
    pub reply:                 f64,
    pub retweet:               f64,
    pub photo_expand:          f64,
    pub click:                 f64,
    pub profile_click:         f64,
    pub vqv:                   f64,
    pub share:                 f64,
    pub share_via_dm:          f64,
    pub share_via_copy_link:   f64,
    pub dwell:                 f64,
    pub quote:                 f64,
    pub quoted_click:          f64,
    pub quoted_vqv:            f64,
    pub cont_dwell_time:       f64,
    pub cont_click_dwell_time: f64,
    pub follow_author:         f64,
    pub not_interested:        f64,
    pub block_author:          f64,
    pub mute_author:           f64,
    pub report:                f64,
    pub not_dwelled:           f64,
    pub negative_sum:          f64,
    pub total_sum:             f64,
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
        let cont_click_dwell    = 0.01;
        let follow_author       = 2.0;
        let not_interested      = -0.3;
        let block_author        = -1.0;
        let mute_author         = -0.5;
        let report              = -2.0;
        let not_dwelled         = -0.1;

        let positive_sum = favorite + reply + retweet + photo_expand + click
            + profile_click + vqv + share + share_via_dm + share_via_copy_link
            + dwell + quote + quoted_click + quoted_vqv + follow_author;
        let negative_sum =
            -(not_interested + block_author + mute_author + report + not_dwelled);
        let total_sum = positive_sum + negative_sum;

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

// ── Core scoring logic — scalar f64 baseline ─────────────────────────────────
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

// ── AoS f32 variant — kept as a comparison baseline ──────────────────────────
//
// RESULT (build #29): this is 2.3× SLOWER than the scalar f64 path above.
//
// Why it loses:
//   • 22 sequential f64→f32 widening conversions per candidate
//   • All 22 products written to a stack array, then re-read for .sum()
//   • LLVM can vectorize the 22-element sum (~3 AVX2 ops) but still loops
//     one candidate at a time — no vectorization across the candidate batch
//
// The correct way to vectorize across candidates is Struct-of-Arrays (below).
pub fn compute_weighted_score_fast(w: &ScoringWeights, c: &PostCandidate) -> f64 {
    let s = &c.phoenix_scores;

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

    let products: [f32; 22] = [
        s.favorite_score.unwrap_or(0.0) as f32            * w.favorite            as f32,
        s.reply_score.unwrap_or(0.0) as f32               * w.reply               as f32,
        s.retweet_score.unwrap_or(0.0) as f32             * w.retweet             as f32,
        s.photo_expand_score.unwrap_or(0.0) as f32        * w.photo_expand        as f32,
        s.click_score.unwrap_or(0.0) as f32               * w.click               as f32,
        s.profile_click_score.unwrap_or(0.0) as f32       * w.profile_click       as f32,
        s.vqv_score.unwrap_or(0.0) as f32                 * vqv_w,
        s.share_score.unwrap_or(0.0) as f32               * w.share               as f32,
        s.share_via_dm_score.unwrap_or(0.0) as f32        * w.share_via_dm        as f32,
        s.share_via_copy_link_score.unwrap_or(0.0) as f32 * w.share_via_copy_link as f32,
        s.dwell_score.unwrap_or(0.0) as f32               * w.dwell               as f32,
        s.quote_score.unwrap_or(0.0) as f32               * w.quote               as f32,
        s.quoted_click_score.unwrap_or(0.0) as f32        * w.quoted_click        as f32,
        s.quoted_vqv_score.unwrap_or(0.0) as f32          * qvqv_w,
        s.dwell_time.unwrap_or(0.0) as f32                * w.cont_dwell_time     as f32,
        s.click_dwell_time.unwrap_or(0.0) as f32          * w.cont_click_dwell_time as f32,
        s.follow_author_score.unwrap_or(0.0) as f32       * w.follow_author       as f32,
        s.not_interested_score.unwrap_or(0.0) as f32      * w.not_interested      as f32,
        s.block_author_score.unwrap_or(0.0) as f32        * w.block_author        as f32,
        s.mute_author_score.unwrap_or(0.0) as f32         * w.mute_author         as f32,
        s.report_score.unwrap_or(0.0) as f32              * w.report              as f32,
        s.not_dwelled_score.unwrap_or(0.0) as f32         * w.not_dwelled         as f32,
    ];

    let combined: f32 = products.iter().copied().sum();
    offset_score(combined as f64, w)
}

// ── Struct-of-Arrays Phoenix scores ──────────────────────────────────────────
//
// AoS layout (PostCandidate slice): scores for candidate-0 sit next to
// candidate-0's author_id, tweet_id, ancestors, etc. — struct fields are
// ~500 bytes apart in memory for adjacent candidates.  LLVM can only
// vectorize the 22-term inner sum (~3 AVX2 ops) and must fetch a new cache
// line for each candidate.
//
// SoA layout (this struct): all favorite_scores are contiguous, all
// reply_scores are contiguous, etc.  To score N candidates we make 22 passes;
// each pass is a tight loop over a contiguous Vec<f32> that LLVM auto-
// vectorizes to `vfmadd231ps ymm` — 8 candidates per instruction.
//
//   22 passes × ceil(N/8) AVX2 ops  vs  N × (22/8) AVX2 ops for AoS
//
// The instruction counts are similar (~2.75N each), but SoA streams memory
// sequentially and fits entirely in L2 for N=2000 (176 KB vs ~1 MB for AoS),
// giving substantially better throughput.
//
// Construction cost: one scatter pass over the AoS slice (N × 22 reads +
// 22 contiguous writes).  Amortised cheaply across a full request.
#[derive(Debug, Default)]
pub struct PhoenixScoresSoA {
    pub favorite_score:            Vec<f32>,
    pub reply_score:               Vec<f32>,
    pub retweet_score:             Vec<f32>,
    pub photo_expand_score:        Vec<f32>,
    pub click_score:               Vec<f32>,
    pub profile_click_score:       Vec<f32>,
    /// Zero for ineligible video candidates; score × 1.0 for eligible.
    /// Multiplied by w.vqv in the scoring pass (scalar broadcast).
    pub vqv_score:                 Vec<f32>,
    pub share_score:               Vec<f32>,
    pub share_via_dm_score:        Vec<f32>,
    pub share_via_copy_link_score: Vec<f32>,
    pub dwell_score:               Vec<f32>,
    pub quote_score:               Vec<f32>,
    pub quoted_click_score:        Vec<f32>,
    /// Zero for ineligible quoted-video candidates.
    pub quoted_vqv_score:          Vec<f32>,
    pub follow_author_score:       Vec<f32>,
    pub not_interested_score:      Vec<f32>,
    pub block_author_score:        Vec<f32>,
    pub mute_author_score:         Vec<f32>,
    pub report_score:              Vec<f32>,
    pub not_dwelled_score:         Vec<f32>,
    pub dwell_time:                Vec<f32>,
    pub click_dwell_time:          Vec<f32>,
    pub len: usize,
}

impl PhoenixScoresSoA {
    /// Scatter one AoS slice into SoA layout.
    /// VQV eligibility is resolved here so the scoring pass needs no branches.
    pub fn from_candidates(candidates: &[PostCandidate], w: &ScoringWeights) -> Self {
        let n = candidates.len();
        let mut s = PhoenixScoresSoA {
            favorite_score:            Vec::with_capacity(n),
            reply_score:               Vec::with_capacity(n),
            retweet_score:             Vec::with_capacity(n),
            photo_expand_score:        Vec::with_capacity(n),
            click_score:               Vec::with_capacity(n),
            profile_click_score:       Vec::with_capacity(n),
            vqv_score:                 Vec::with_capacity(n),
            share_score:               Vec::with_capacity(n),
            share_via_dm_score:        Vec::with_capacity(n),
            share_via_copy_link_score: Vec::with_capacity(n),
            dwell_score:               Vec::with_capacity(n),
            quote_score:               Vec::with_capacity(n),
            quoted_click_score:        Vec::with_capacity(n),
            quoted_vqv_score:          Vec::with_capacity(n),
            follow_author_score:       Vec::with_capacity(n),
            not_interested_score:      Vec::with_capacity(n),
            block_author_score:        Vec::with_capacity(n),
            mute_author_score:         Vec::with_capacity(n),
            report_score:              Vec::with_capacity(n),
            not_dwelled_score:         Vec::with_capacity(n),
            dwell_time:                Vec::with_capacity(n),
            click_dwell_time:          Vec::with_capacity(n),
            len: n,
        };
        for c in candidates {
            let ps = &c.phoenix_scores;
            // Resolve VQV eligibility once per candidate so the hot scoring
            // loop has no branches — ineligible candidates get score 0.0 and
            // still participate in the fma_pass without skewing results.
            let vqv_eligible = c.min_video_duration_ms
                .map_or(false, |ms| ms > w.min_video_duration_ms);
            let qvqv_eligible = c.quoted_video_duration_ms
                .map_or(false, |ms| ms > w.min_video_duration_ms);

            s.favorite_score.push(ps.favorite_score.unwrap_or(0.0) as f32);
            s.reply_score.push(ps.reply_score.unwrap_or(0.0) as f32);
            s.retweet_score.push(ps.retweet_score.unwrap_or(0.0) as f32);
            s.photo_expand_score.push(ps.photo_expand_score.unwrap_or(0.0) as f32);
            s.click_score.push(ps.click_score.unwrap_or(0.0) as f32);
            s.profile_click_score.push(ps.profile_click_score.unwrap_or(0.0) as f32);
            s.vqv_score.push(
                if vqv_eligible { ps.vqv_score.unwrap_or(0.0) as f32 } else { 0.0 }
            );
            s.share_score.push(ps.share_score.unwrap_or(0.0) as f32);
            s.share_via_dm_score.push(ps.share_via_dm_score.unwrap_or(0.0) as f32);
            s.share_via_copy_link_score.push(ps.share_via_copy_link_score.unwrap_or(0.0) as f32);
            s.dwell_score.push(ps.dwell_score.unwrap_or(0.0) as f32);
            s.quote_score.push(ps.quote_score.unwrap_or(0.0) as f32);
            s.quoted_click_score.push(ps.quoted_click_score.unwrap_or(0.0) as f32);
            s.quoted_vqv_score.push(
                if qvqv_eligible { ps.quoted_vqv_score.unwrap_or(0.0) as f32 } else { 0.0 }
            );
            s.follow_author_score.push(ps.follow_author_score.unwrap_or(0.0) as f32);
            s.not_interested_score.push(ps.not_interested_score.unwrap_or(0.0) as f32);
            s.block_author_score.push(ps.block_author_score.unwrap_or(0.0) as f32);
            s.mute_author_score.push(ps.mute_author_score.unwrap_or(0.0) as f32);
            s.report_score.push(ps.report_score.unwrap_or(0.0) as f32);
            s.not_dwelled_score.push(ps.not_dwelled_score.unwrap_or(0.0) as f32);
            s.dwell_time.push(ps.dwell_time.unwrap_or(0.0) as f32);
            s.click_dwell_time.push(ps.click_dwell_time.unwrap_or(0.0) as f32);
        }
        s
    }
}

/// One vectorised multiply-accumulate pass over all N candidates for a single
/// weight.  LLVM emits `vfmadd231ps ymm` (8-wide f32 FMA) with a scalar
/// broadcast of `w` when compiled with AVX2+FMA.
#[inline(always)]
fn fma_pass(out: &mut [f32], src: &[f32], w: f32) {
    for (o, &s) in out.iter_mut().zip(src) {
        *o += s * w;
    }
}

/// Batch weighted score over N candidates using SoA layout.
///
/// Makes 22 `fma_pass` calls — one per weight — each vectorized 8-wide across
/// candidates.  Returns raw f32 combined scores (before `offset_score`).
/// Call `apply_offset_scores` to convert to the final f64 pipeline output.
pub fn compute_batch_weighted_scores_soa(w: &ScoringWeights, s: &PhoenixScoresSoA) -> Vec<f32> {
    let n = s.len;
    let mut out = vec![0.0f32; n];

    fma_pass(&mut out, &s.favorite_score,            w.favorite            as f32);
    fma_pass(&mut out, &s.reply_score,               w.reply               as f32);
    fma_pass(&mut out, &s.retweet_score,             w.retweet             as f32);
    fma_pass(&mut out, &s.photo_expand_score,        w.photo_expand        as f32);
    fma_pass(&mut out, &s.click_score,               w.click               as f32);
    fma_pass(&mut out, &s.profile_click_score,       w.profile_click       as f32);
    fma_pass(&mut out, &s.vqv_score,                 w.vqv                 as f32);
    fma_pass(&mut out, &s.share_score,               w.share               as f32);
    fma_pass(&mut out, &s.share_via_dm_score,        w.share_via_dm        as f32);
    fma_pass(&mut out, &s.share_via_copy_link_score, w.share_via_copy_link as f32);
    fma_pass(&mut out, &s.dwell_score,               w.dwell               as f32);
    fma_pass(&mut out, &s.quote_score,               w.quote               as f32);
    fma_pass(&mut out, &s.quoted_click_score,        w.quoted_click        as f32);
    fma_pass(&mut out, &s.quoted_vqv_score,          w.quoted_vqv          as f32);
    fma_pass(&mut out, &s.follow_author_score,       w.follow_author       as f32);
    fma_pass(&mut out, &s.not_interested_score,      w.not_interested      as f32);
    fma_pass(&mut out, &s.block_author_score,        w.block_author        as f32);
    fma_pass(&mut out, &s.mute_author_score,         w.mute_author         as f32);
    fma_pass(&mut out, &s.report_score,              w.report              as f32);
    fma_pass(&mut out, &s.not_dwelled_score,         w.not_dwelled         as f32);
    fma_pass(&mut out, &s.dwell_time,                w.cont_dwell_time     as f32);
    fma_pass(&mut out, &s.click_dwell_time,          w.cont_click_dwell_time as f32);

    out
}

/// Convert raw SoA combined f32 scores → final f64 pipeline scores.
/// This is the same `offset_score` normalisation applied per-element.
pub fn apply_offset_scores(combined: &[f32], w: &ScoringWeights) -> Vec<f64> {
    combined.iter().map(|&c| offset_score(c as f64, w)).collect()
}

// ── Precomputed diversity table ───────────────────────────────────────────────
pub struct DiversityTable {
    multipliers: Vec<f64>,
    floor: f64,
}

impl DiversityTable {
    pub fn new(decay: f64, floor: f64, max_positions: usize) -> Self {
        let multipliers = (0..max_positions)
            .map(|i| (1.0 - floor) * decay.powf(i as f64) + floor)
            .collect();
        Self { multipliers, floor }
    }

    #[inline(always)]
    pub fn get(&self, position: usize) -> f64 {
        self.multipliers.get(position).copied().unwrap_or(self.floor)
    }
}

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
        let mult = table.get(*entry);
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
    indexed.select_nth_unstable_by(k, |a, b| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut top = indexed[..k].to_vec();
    top.sort_unstable_by(|a, b| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    top
}

// ── Thunder light post ────────────────────────────────────────────────────────
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
    (0..n)
        .map(|i| {
            let mut score = || -> Option<f64> { Some(rng.gen::<f64>() * 0.1) };
            PostCandidate {
                tweet_id:  i as u64 + 1,
                author_id: (i % 200) as u64 + 1,
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
    (0..n)
        .map(|i| LightPost {
            post_id:    i as i64 + 1,
            author_id:  (i % 500) as i64 + 1,
            created_at: base_ts + i as i64 * 10,
            is_reply:   i % 4 == 0,
            is_retweet: i % 7 == 0,
            has_video:  i % 6 == 0,
        })
        .collect()
}

// ── Bloom filter ──────────────────────────────────────────────────────────────
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
        let h = (item as u64)
            .wrapping_mul(0x517cc1b727220a95)
            .wrapping_add(seed as u64);
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
