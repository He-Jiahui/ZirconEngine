---
title: Editor01 Sprite Atlas Resolution Hash Index
category: zircon_editor
report_id: Editor01-sprite-atlas-resolution-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Sprite Atlas Resolution Hash Index

## Scope

This slice replaces the ordered sprite-atlas resolution cache index with a `HashMap`. Repeated
template-image resolution now uses expected constant-time lookup instead of ordered comparisons
across up to 128 resident source-key and source-path pairs.

The complete two-field key, positive and negative result caching, 128-entry bound, cache
invalidation, manifest discovery, and decoded-image ownership are unchanged. On a capacity miss,
the cache still evicts the lexicographically smallest complete key by selecting `keys().min()`, so
the previous deterministic policy is preserved; that bounded miss-only scan is not part of the
stable-hit claim.

## Performance Workload

The release workload fills all 128 entries with source keys and paths sharing a long prefix, then
performs 4,096 stable hits.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered-index lookups | 4,096 | 0 |
| Hash-index lookups | 0 | 4,096 |
| Manifest or image reloads on hits | 0 | 0 |
| Capacity or eviction-policy changes | 0 | 0 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_SPRITE_ATLAS_RESOLUTION_HASH_INDEX_BENCH_V1`. Acceptance requires HashMap lookup P95 to
be at least 30% below the legacy BTreeMap path. Exact Windows P50/P95 timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826bp_sprite_atlas_hash_index_preserves_full_key_and_negative_cache`
  covers full-key separation and positive/negative cache entries.
- `optimization_batch_20260826bp_sprite_atlas_hash_index_preserves_deterministic_capacity` locks
  the 128-entry bound and lexicographically smallest-key eviction.
- `optimization_batch_20260826bp_sprite_atlas_resolution_hash_index_p95` reports paired release
  P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges sprite-atlas
resolution indexing.
