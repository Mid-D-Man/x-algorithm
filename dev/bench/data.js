window.BENCHMARK_DATA = {
  "lastUpdate": 1779110851639,
  "repoUrl": "https://github.com/Mid-D-Man/x-algorithm",
  "entries": {
    "X Algorithm Benchmarks": [
      {
        "commit": {
          "author": {
            "name": "AbdulHamid Mamman Suleiman",
            "username": "Mid-D-Man",
            "email": "94022993+Mid-D-Man@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "704fa5828d3222ac57b69adfee806394d1508dd8",
          "message": "Update bench.yml",
          "timestamp": "2026-05-17T07:33:12Z",
          "url": "https://github.com/Mid-D-Man/x-algorithm/commit/704fa5828d3222ac57b69adfee806394d1508dd8"
        },
        "date": 1779008454795,
        "tool": "cargo",
        "benches": [
          {
            "name": "scoring/weighted_score/single_candidate",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score/batch_500",
            "value": 7795,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score/batch_2000",
            "value": 33934,
            "range": "± 393",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/500_candidates",
            "value": 21691,
            "range": "± 772",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/2000_candidates",
            "value": 101917,
            "range": "± 2072",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/offset_score/1000_scores",
            "value": 1257,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_500",
            "value": 22404,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_1000",
            "value": 49800,
            "range": "± 1262",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_2000",
            "value": 169660,
            "range": "± 1594",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline/2000_candidates_to_top50",
            "value": 155164,
            "range": "± 562",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/2000_lookups",
            "value": 10143,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/500_lookups",
            "value": 2551,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/drop_duplicates/2200_with_200_dupes",
            "value": 421152,
            "range": "± 4492",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/dedup_conversation/2000_candidates_30pct_replies",
            "value": 210965,
            "range": "± 761",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/socialgraph_filter/2000_candidates",
            "value": 126027,
            "range": "± 1122",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/10000_posts_top_200",
            "value": 9439,
            "range": "± 326",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/50000_posts_top_200",
            "value": 86596,
            "range": "± 425",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/100000_posts_top_400",
            "value": 198317,
            "range": "± 759",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/dashmap_post_store/500_following_20_posts_each",
            "value": 442963,
            "range": "± 3456",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_none_expired",
            "value": 159522,
            "range": "± 4027",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_90pct_expired",
            "value": 27044,
            "range": "± 619",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "94022993+Mid-D-Man@users.noreply.github.com",
            "name": "AbdulHamid Mamman Suleiman",
            "username": "Mid-D-Man"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "f4ea42a98d2fcad1bd1b3d338dab2b4bb80193cc",
          "message": "Update thunder_service.rs",
          "timestamp": "2026-05-18T14:22:15+01:00",
          "tree_id": "74a265d94f04933458bda8d28475121c3bfdefdf",
          "url": "https://github.com/Mid-D-Man/x-algorithm/commit/f4ea42a98d2fcad1bd1b3d338dab2b4bb80193cc"
        },
        "date": 1779110850974,
        "tool": "cargo",
        "benches": [
          {
            "name": "scoring/weighted_score/single_candidate",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score/batch_500",
            "value": 5702,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score/batch_2000",
            "value": 24052,
            "range": "± 1922",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/single_candidate",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/batch_500",
            "value": 14618,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/batch_2000",
            "value": 58847,
            "range": "± 5098",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/500_candidates",
            "value": 16381,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/2000_candidates",
            "value": 81038,
            "range": "± 499",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity_table/500_candidates",
            "value": 11105,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity_table/2000_candidates",
            "value": 60627,
            "range": "± 4230",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/offset_score/1000_scores",
            "value": 1283,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_500",
            "value": 25128,
            "range": "± 134",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_1000",
            "value": 52877,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_2000",
            "value": 113669,
            "range": "± 880",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_50",
            "value": 3077,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_100",
            "value": 3516,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_200",
            "value": 4532,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline/2000_candidates_to_top50",
            "value": 118158,
            "range": "± 5020",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline_optimised/2000_candidates_to_top50",
            "value": 108371,
            "range": "± 4880",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/2000_lookups",
            "value": 6892,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/500_lookups",
            "value": 1731,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/drop_duplicates/2200_with_200_dupes",
            "value": 517181,
            "range": "± 2408",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/dedup_conversation/2000_candidates_30pct_replies",
            "value": 148063,
            "range": "± 3288",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/socialgraph_filter/2000_candidates",
            "value": 83406,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/10000_posts_top_200",
            "value": 6129,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/50000_posts_top_200",
            "value": 48964,
            "range": "± 573",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/100000_posts_top_400",
            "value": 99943,
            "range": "± 2062",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/dashmap_post_store/500_following_20_posts_each",
            "value": 358676,
            "range": "± 9601",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_none_expired",
            "value": 101771,
            "range": "± 7186",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_90pct_expired",
            "value": 18982,
            "range": "± 974",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}