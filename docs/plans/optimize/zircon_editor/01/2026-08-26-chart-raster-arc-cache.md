---
title: Editor01 Chart Raster Arc Cache
category: zircon_editor
report_id: Editor01-chart-raster-arc-cache-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Chart Raster Arc Cache

## Scope

This slice removes full RGBA buffer clones from retained-host line, pie, sparkline, and gauge chart
cache hits. The cache owns immutable pixels as `Arc<[u8]>` and shares the same allocation directly
with `HostPaintCommand::image_pixels`.

The existing `BTreeMap` index, 128-entry limit, 32 MiB resident-byte limit, access clock, duplicate
replacement, and least-recently-used eviction policy are unchanged. Resource-key strings remain
owned by paint commands and therefore continue to clone once per cache hit.

## Deterministic Work Model

The release workload fills all 128 entries with 128 KiB rasters and performs 128 stable hits.

| Work per stable frame | Legacy | Optimized |
|---|---:|---:|
| BTreeMap lookups | 128 | 128 |
| Heap-backed pixel buffers cloned | 128 | 0 |
| Pixel bytes copied on hits | 16,777,216 | 0 |
| Capacity, byte budget, or eviction-policy changes | 0 | 0 |

Deterministic hit-path pixel copying falls by 100%. The ignored release gate runs 17 alternating
sample pairs and emits `EDITOR01_CHART_RASTER_ARC_CACHE_BENCH_V1`. Acceptance requires Arc-backed
lookup P95 to be at least 90% below the legacy `Vec<u8>` clone path. Exact Windows P50/P95 timings
remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bm_chart_raster_arc_cache_preserves_lru` covers Arc allocation
  identity, hit promotion, capacity, and least-recent eviction.
- `optimization_batch_20260826bm_chart_raster_arc_cache_eliminates_pixel_copy` locks Arc ownership
  through the cache and paint-command producer and verifies the deterministic copy model.
- `optimization_batch_20260826bm_chart_raster_arc_cache_p95` reports paired release P50/P95
  samples and enforces the 90% P95 reduction gate.

## Remaining Parent-plan Work

Editor01 still owns retained invalidation, layout/paint scaling, shared visual materialization, GPU
upload coordination, accessibility, and product-scale interaction evidence. This slice only
converges chart-raster cache pixel ownership.
