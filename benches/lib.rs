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
        // Realistic production-like weights from the README description
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

    // vqv is only eligible if video is long enough
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
        let score = || -> Option<f64> { Some(rng.gen::<f64>() * 0.1) };
        PostCandidate {
            tweet_id:   i as u64 + 1,
            author_id:  (i % 200) as u64 + 1, // ~200 unique authors for diversity test
            in_network: Some(i % 3 != 0),
            min_video_duration_ms: if i % 5 == 0 { Some(10_000) } else { None },
            phoenix_scores: PhoenixScores {
                favorite_score:      score(),
                reply_score:         score(),
                retweet_score:       score(),
                photo_expand_score:  score(),
                click_score:         score(),
                profile_click_score: score(),
                vqv_score:           score(),
                share_score:         score(),
                dwell_score:         score(),
                follow_author_score: score(),
                not_interested_score: Some(-rng.gen::<f64>() * 0.05),
                report_score:        Some(-rng.gen::<f64>() * 0.01),
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
