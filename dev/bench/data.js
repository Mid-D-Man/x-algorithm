window.BENCHMARK_DATA = {
  "lastUpdate": 1779158885978,
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
          "id": "f5a4be0f620974d15b1d7affc373249332110a81",
          "message": "Update bench.yml",
          "timestamp": "2026-05-18T21:06:12+01:00",
          "tree_id": "0acdd376b273929a738de45243aad1c06361c82a",
          "url": "https://github.com/Mid-D-Man/x-algorithm/commit/f5a4be0f620974d15b1d7affc373249332110a81"
        },
        "date": 1779135144250,
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
            "value": 5922,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score/batch_2000",
            "value": 25790,
            "range": "± 272",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/single_candidate",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/batch_500",
            "value": 12398,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/batch_2000",
            "value": 50445,
            "range": "± 4221",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/500_candidates",
            "value": 20259,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/2000_candidates",
            "value": 101914,
            "range": "± 720",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity_table/500_candidates",
            "value": 14643,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity_table/2000_candidates",
            "value": 78414,
            "range": "± 526",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/offset_score/1000_scores",
            "value": 1704,
            "range": "± 179",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_500",
            "value": 30335,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_1000",
            "value": 63803,
            "range": "± 1863",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_2000",
            "value": 141934,
            "range": "± 11606",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_50",
            "value": 3679,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_100",
            "value": 4268,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_200",
            "value": 5618,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline/2000_candidates_to_top50",
            "value": 146132,
            "range": "± 1379",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline_optimised/2000_candidates_to_top50",
            "value": 123721,
            "range": "± 916",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/2000_lookups",
            "value": 8946,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/500_lookups",
            "value": 2227,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/drop_duplicates/2200_with_200_dupes",
            "value": 686191,
            "range": "± 1569",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/dedup_conversation/2000_candidates_30pct_replies",
            "value": 187272,
            "range": "± 1528",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/socialgraph_filter/2000_candidates",
            "value": 118095,
            "range": "± 680",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/10000_posts_top_200",
            "value": 10941,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/50000_posts_top_200",
            "value": 58467,
            "range": "± 7216",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/100000_posts_top_400",
            "value": 114965,
            "range": "± 936",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/dashmap_post_store/500_following_20_posts_each",
            "value": 562944,
            "range": "± 9542",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_none_expired",
            "value": 122944,
            "range": "± 2642",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_90pct_expired",
            "value": 24138,
            "range": "± 749",
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
          "id": "6cd7500f75a828dec18ec9ef063bb6013ebf9f22",
          "message": "Update lib.rs",
          "timestamp": "2026-05-18T21:07:05+01:00",
          "tree_id": "2b27beaeada1dbc2dd62763ab0567a824a61afd3",
          "url": "https://github.com/Mid-D-Man/x-algorithm/commit/6cd7500f75a828dec18ec9ef063bb6013ebf9f22"
        },
        "date": 1779135195967,
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
            "value": 6041,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score/batch_2000",
            "value": 26295,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/single_candidate",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/batch_500",
            "value": 15239,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/batch_2000",
            "value": 61124,
            "range": "± 411",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/500_candidates",
            "value": 21252,
            "range": "± 444",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/2000_candidates",
            "value": 105515,
            "range": "± 887",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity_table/500_candidates",
            "value": 14532,
            "range": "± 501",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity_table/2000_candidates",
            "value": 76723,
            "range": "± 922",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/offset_score/1000_scores",
            "value": 1486,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_500",
            "value": 30299,
            "range": "± 812",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_1000",
            "value": 72550,
            "range": "± 1047",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_2000",
            "value": 208110,
            "range": "± 2322",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_50",
            "value": 3584,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_100",
            "value": 4108,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_200",
            "value": 5300,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline/2000_candidates_to_top50",
            "value": 152052,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline_optimised/2000_candidates_to_top50",
            "value": 133414,
            "range": "± 1102",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/2000_lookups",
            "value": 7934,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/500_lookups",
            "value": 1953,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/drop_duplicates/2200_with_200_dupes",
            "value": 602742,
            "range": "± 2643",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/dedup_conversation/2000_candidates_30pct_replies",
            "value": 284729,
            "range": "± 37619",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/socialgraph_filter/2000_candidates",
            "value": 118302,
            "range": "± 4622",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/10000_posts_top_200",
            "value": 10256,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/50000_posts_top_200",
            "value": 67759,
            "range": "± 689",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/100000_posts_top_400",
            "value": 135698,
            "range": "± 748",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/dashmap_post_store/500_following_20_posts_each",
            "value": 584632,
            "range": "± 5539",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_none_expired",
            "value": 110563,
            "range": "± 1775",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_90pct_expired",
            "value": 21152,
            "range": "± 709",
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
          "id": "cc63c41570295022546c793e0934118f306ee3e4",
          "message": "Update bench.yml",
          "timestamp": "2026-05-19T03:42:02+01:00",
          "tree_id": "8a65d1047fe9af43de23f9f466fb009d3733ae65",
          "url": "https://github.com/Mid-D-Man/x-algorithm/commit/cc63c41570295022546c793e0934118f306ee3e4"
        },
        "date": 1779158885648,
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
            "value": 6127,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score/batch_2000",
            "value": 26745,
            "range": "± 880",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/single_candidate",
            "value": 32,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/batch_500",
            "value": 15084,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/weighted_score_simd/batch_2000",
            "value": 61162,
            "range": "± 899",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/500_candidates",
            "value": 21902,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity/2000_candidates",
            "value": 114674,
            "range": "± 806",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity_table/500_candidates",
            "value": 14179,
            "range": "± 674",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/author_diversity_table/2000_candidates",
            "value": 78846,
            "range": "± 771",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/offset_score/1000_scores",
            "value": 1486,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_500",
            "value": 29753,
            "range": "± 486",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_1000",
            "value": 72248,
            "range": "± 2861",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/sort_candidates/sort_2000",
            "value": 208661,
            "range": "± 2722",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_50",
            "value": 3499,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_100",
            "value": 4017,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/top_k_select/2000_to_top_200",
            "value": 5160,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline/2000_candidates_to_top50",
            "value": 159342,
            "range": "± 1082",
            "unit": "ns/iter"
          },
          {
            "name": "scoring/full_pipeline_optimised/2000_candidates_to_top50",
            "value": 133895,
            "range": "± 1201",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/2000_lookups",
            "value": 7895,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/bloom_filter/500_lookups",
            "value": 1957,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/drop_duplicates/2200_with_200_dupes",
            "value": 609575,
            "range": "± 9671",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/dedup_conversation/2000_candidates_30pct_replies",
            "value": 189216,
            "range": "± 740",
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/socialgraph_filter/2000_candidates",
            "value": 121069,
            "range": "± 1582",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/10000_posts_top_200",
            "value": 10304,
            "range": "± 1053",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/50000_posts_top_200",
            "value": 51374,
            "range": "± 1311",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/score_recent/100000_posts_top_400",
            "value": 102787,
            "range": "± 1899",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/dashmap_post_store/500_following_20_posts_each",
            "value": 555530,
            "range": "± 2217",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_none_expired",
            "value": 134198,
            "range": "± 1867",
            "unit": "ns/iter"
          },
          {
            "name": "thunder/trim_old_posts/100k_posts_90pct_expired",
            "value": 19734,
            "range": "± 630",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}