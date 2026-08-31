---
title: Editor01 Sprite Atlas Manifest Hash Index
category: zircon_editor
report_id: Editor01-sprite-atlas-manifest-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Sprite Atlas Manifest Hash Index

## Scope

This slice replaces the ordered sprite-atlas manifest cache index with a `HashMap`. Repeated atlas
manifest lookup now uses expected constant time instead of ordered path comparisons across up to
64 positive or negative cache entries.

The complete path key, validated TOML payload, negative-result caching, 64-entry bound, and cache
invalidation contract are unchanged. On a capacity miss, the cache still evicts the
lexicographically smallest complete path by selecting `keys().min()`, preserving the previous
deterministic policy; that bounded miss-only scan is outside the stable-hit claim.

## Performance Workload

The release workload fills all 64 entries with paths sharing a long prefix and performs 4,096
stable hits.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered-index lookups | 4,096 | 0 |
| Hash-index lookups | 0 | 4,096 |
| Filesystem reads on hits | 0 | 0 |
| Capacity or eviction-policy changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_SPRITE_ATLAS_MANIFEST_HASH_INDEX_BENCH_V1`. Acceptance requires HashMap lookup P95 to be
at least 30% below the legacy BTreeMap path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bq_sprite_atlas_manifest_hash_index_preserves_negative_cache`
  covers complete-path separation and negative cache entries.
- `optimization_batch_20260826bq_sprite_atlas_manifest_hash_index_preserves_deterministic_capacity`
  locks the 64-entry bound and lexicographically smallest-path eviction.
- `optimization_batch_20260826bq_sprite_atlas_manifest_hash_index_p95` reports paired release
  P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges sprite-atlas
manifest indexing.
