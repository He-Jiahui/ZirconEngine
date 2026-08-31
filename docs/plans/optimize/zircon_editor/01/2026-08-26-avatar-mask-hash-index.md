---
title: Editor01 Avatar Mask Hash Index
category: zircon_editor
report_id: Editor01-avatar-mask-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Avatar Mask Hash Index

## Scope

This slice replaces the ordered avatar-mask cache index with a `HashMap`. Stable avatar paint now
uses expected constant-time lookup instead of ordered key traversal across up to 64 resident mask
variants.

The 64-entry and 16 MiB bounds, immutable Arc-backed image pixels, access clock, duplicate
replacement, resident-byte accounting, and least-recently-used eviction policy are unchanged. The
cache key still includes the full resource key, dimensions, and radius bits, preserving collision
correctness.

## Performance Workload

The release workload fills all 64 entries with resource keys sharing a 130-byte prefix and performs
4,096 stable hits.

| Work per stable frame | Before | After |
|---|---:|---:|
| Ordered-index lookups | 4,096 | 0 |
| Hash-index lookups | 0 | 4,096 |
| Pixel-buffer copies | 0 | 0 |
| Capacity, byte budget, or eviction-policy changes | 0 | 0 |

The index changes from ordered traversal to expected constant-time hash lookup while preserving
full-key equality. The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR01_AVATAR_MASK_HASH_INDEX_BENCH_V1`. Acceptance requires HashMap lookup P95 to be at least
30% below the legacy `BTreeMap` path. Exact Windows P50/P95 timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826bo_avatar_mask_hash_index_preserves_arc_lru` covers Arc identity,
  hit promotion, capacity, and least-recent eviction in the production cache.
- `optimization_batch_20260826bo_avatar_mask_hash_index_eliminates_ordered_lookup` locks HashMap
  ownership and rejects the legacy BTreeMap index.
- `optimization_batch_20260826bo_avatar_mask_hash_index_p95` reports paired release P50/P95
  samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization,
accessibility, and product-scale interaction evidence. This slice only converges avatar-mask cache
indexing.
