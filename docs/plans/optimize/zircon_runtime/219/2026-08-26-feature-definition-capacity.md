# Runtime219 Feature Definition Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime219-editor165-performance-batch-20260826fx-v1`

## Problem

Runtime plugin catalog construction grew both the feature-definition map and its stable definition
order from empty while the package-registration and standalone feature-registration counts already
provided a useful allocation lower bound.

## Optimization

- Sum both registration input counts with saturation and use the shared capacity for the definition
  HashMap and definition-order Vec.
- Leave diagnostics demand-grown because their count is unrelated to valid registration volume.
- Preserve package/runtime merge order, duplicate handling, declared-feature tracking, diagnostics,
  definition identities, and stable order; packages declaring multiple features may still grow.

## Regression Contract

The `optimization_batch_20260826fx_` Runtime tests cover map/order capacity and order, enforce both
production preallocations, and provide an ignored paired release benchmark emitting
`RUNTIME219_FEATURE_DEFINITION_CAPACITY_BENCH_V1`. It builds 64 maps and order vectors containing
4,096 definitions per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
