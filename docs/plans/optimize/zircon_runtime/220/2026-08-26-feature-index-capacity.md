# Runtime220 Feature Index Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime220-editor166-performance-batch-20260826fy-v1`

## Problem

Runtime plugin feature completion rebuilt an index for every existing project selection from an
empty HashMap even though the selection's current feature count is the exact initial entry count.

## Optimization

- Preallocate each feature-index HashMap from `selection.features.len()` before indexing features.
- Preserve first-index-wins duplicate handling through `entry(...).or_insert(index)`.
- Preserve owner completion order, external-provider completion, feature identities, and manifest
  mutation behavior; newly completed features may still grow the map when required.

## Regression Contract

The `optimization_batch_20260826fy_` Runtime tests cover index capacity, values, and the production
reserve contract, and provide an ignored paired release benchmark emitting
`RUNTIME220_FEATURE_INDEX_CAPACITY_BENCH_V1`. It builds 64 maps containing 4,096 feature indices per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
