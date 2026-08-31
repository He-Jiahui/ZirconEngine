# Runtime237 Config Percentile Stack Buffer

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime237-editor183-performance-batch-20260826gq-v1`

## Problem

Every config persistence report copied at most 64 latency samples into a newly allocated Vec before
sorting them for the P95 metric. Reporting therefore performed a heap allocation even though the
sample count already had a compile-time upper bound.

## Optimization

- Copy the two possible VecDeque slices into a fixed 64-element stack buffer.
- Sort only the initialized prefix and keep the existing percentile rank calculation.
- Preserve empty, contiguous, and wrapped VecDeque behavior without a reporting-path heap allocation.

## Regression Contract

The `optimization_batch_20260826gq_` Runtime tests cover empty and wrapped sample inventories,
enforce the fixed-stack source contract, and provide an ignored paired release benchmark emitting
`RUNTIME237_CONFIG_PERCENTILE_STACK_BUFFER_BENCH_V1`. It measures repeated 64-sample P95 reports and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
