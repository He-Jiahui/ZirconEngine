# Runtime226 Native Diagnostics Map Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime226-editor172-performance-batch-20260826ge-v1`

## Problem

Native plugin diagnostic projection grew both plugin-indexed maps from empty even though every
loaded plugin is guaranteed to establish an entry in each map.

## Optimization

- Preallocate diagnostic and loaded-state HashMaps from the loaded plugin row count.
- Allow raw diagnostics to grow the diagnostics map beyond that lower bound when they mention
  plugins that were not loaded.
- Leave descriptor and entry diagnostic vectors demand-grown because their output counts depend on
  validation results rather than the number of loaded rows.
- Preserve diagnostic routing, sorting and deduplication, loaded-state coalescing, and descriptor
  conjunction semantics for repeated plugin IDs.

## Regression Contract

The `optimization_batch_20260826ge_` Runtime tests cover loaded-row capacity and the two-map source
contract, and provide an ignored paired release benchmark emitting
`RUNTIME226_NATIVE_DIAGNOSTICS_MAP_CAPACITY_BENCH_V1`. It builds 64 projections with 4,096 loaded
plugins per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
