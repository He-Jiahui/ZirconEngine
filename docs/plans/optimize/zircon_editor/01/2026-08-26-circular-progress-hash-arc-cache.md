---
title: Editor01 Circular Progress Hash Arc Cache
category: zircon_editor
report_id: Editor01-circular-progress-hash-arc-cache-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Circular Progress Hash Arc Cache

## Scope

This slice removes the linear LRU hit scan and full RGBA clone from retained-host circular-progress
paint. The cache now indexes raster keys with a `HashMap`, records LRU access generations, and
shares immutable pixels as `Arc<[u8]>` directly with `HostPaintCommand::image_pixels`.

The 128-entry and 16 MiB limits are unchanged. Duplicate insertions still replace the prior raster,
stable hits still promote the key, and a full cache still evicts the least-recently-used entry.
Generation overflow rebases live entries in oldest-first order. Resource-key strings remain owned
by paint commands and therefore continue to clone once per hit.

## Deterministic Work Model

The release workload fills all 128 entries with 64 KiB rasters and performs 128 stable hits against
the legacy tail entry.

| Work per stable frame | Legacy | Optimized |
|---|---:|---:|
| Key comparisons / hash lookups | 16,384 comparisons | 128 lookups |
| Heap-backed pixel buffers cloned | 128 | 0 |
| Pixel bytes copied on hits | 8,388,608 | 0 |
| Capacity or resident-byte policy changes | 0 | 0 |

Deterministic lookup work falls by 99.2188%, while hit-path pixel copying falls by 100%. The ignored
release gate runs 17 alternating sample pairs and emits
`EDITOR01_CIRCULAR_PROGRESS_HASH_ARC_CACHE_BENCH_V1`. Acceptance requires Hash+Arc lookup P95 to be
at least 80% below the legacy `VecDeque`+`Vec<u8>` implementation. Exact Windows P50/P95 timings
remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bl_circular_progress_hash_arc_preserves_lru` covers Arc identity,
  hit promotion, capacity, and least-recent eviction.
- `optimization_batch_20260826bl_circular_progress_hash_arc_eliminates_hit_copy` locks HashMap/Arc
  production ownership and the deterministic lookup/copy model.
- `optimization_batch_20260826bl_circular_progress_hash_arc_p95` reports paired release P50/P95
  samples and enforces the 80% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization, GPU
upload coordination, accessibility, and product-scale interaction evidence. This slice only
converges circular-progress raster cache hits.
