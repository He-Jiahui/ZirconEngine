# Editor180 Drawer Dock Tab Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime234-editor180-performance-batch-20260826gn-v1`

## Problem

Profiling artifact generation collected the left, right, and bottom drawer tab frames into an empty
final Vec, causing reallocations and moving previously collected profile frames as each dock was
appended.

## Optimization

- Sum the three fixed dock tab model row counts before collecting drawer frames.
- Preallocate the final result for all fixed dock rows with saturating capacity arithmetic.
- Keep floating-window collection single-pass and preserve the original surface and tab order.

## Regression Contract

The `optimization_batch_20260826gn_` Editor tests cover exact and saturating dock-row capacity and
enforce the production preallocation contract, and provide an ignored paired release benchmark
emitting `EDITOR180_DRAWER_DOCK_TAB_CAPACITY_BENCH_V1`. It aggregates three 1,365-frame dock chunks
across 128 builds per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
