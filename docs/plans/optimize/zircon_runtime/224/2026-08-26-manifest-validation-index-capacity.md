# Runtime224 Manifest Validation Index Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime224-editor170-performance-batch-20260826gc-v1`

## Problem

Project plugin manifest validation grew several selection/feature indices and per-refresh duplicate
maps from empty even though their corresponding manifest row counts are known at construction time.

## Optimization

- Preallocate selection indices, enabled-provider IDs, and refresh selection IDs from selection count.
- Preallocate per-selection feature indices, feature IDs, and refresh feature IDs from feature count.
- Leave global feature locations and short IDs demand-grown because their unique counts require
  additional projection work; preserve duplicate order, identity facts, refresh state, and lookups.

## Regression Contract

The `optimization_batch_20260826gc_` Runtime tests cover input-sized map/set capacity and all six
production reserves, and provide an ignored paired release benchmark emitting
`RUNTIME224_MANIFEST_VALIDATION_INDEX_CAPACITY_BENCH_V1`. It builds 64 projections with three 4,096
row indices per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
