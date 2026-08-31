# Runtime205 Runtime Feature Registration Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime205-editor151-performance-batch-20260826fj-v1`

## Problem

Runtime plugin feature definition merging grew its registered-feature deduplication set from empty
even though the complete runtime feature registration count was already available as a slice.

## Optimization

- Reserve the runtime feature registration count before inserting registered feature keys.
- Preserve feature order, package-declaration conflict handling, duplicate diagnostics, and final
  definition projection semantics.

## Regression Contract

The `optimization_batch_20260826fj_` Runtime tests merge 256 real feature registration reports,
check definition order and diagnostics, enforce the production capacity source shape, and provide
an ignored paired release benchmark emitting `RUNTIME205_REGISTERED_FEATURE_CAPACITY_BENCH_V1`.
It inserts 256 lightweight keys 2,048 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
