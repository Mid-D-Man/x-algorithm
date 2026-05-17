window.BENCHMARK_DATA = {
  "lastUpdate": 1779008455161,
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
      }
    ]
  }
}