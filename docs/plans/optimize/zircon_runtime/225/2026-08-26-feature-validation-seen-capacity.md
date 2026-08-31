# Runtime225 Feature Validation Seen Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime225-editor171-performance-batch-20260826gd-v1`

## Problem

Standalone runtime feature validation grew temporary seen sets for capabilities, dependencies,
module names, and per-module capabilities from empty despite knowing each source row count.

## Optimization

- Preallocate each temporary HashSet from its corresponding feature or module input length.
- Leave duplicate-occurrence output sets demand-grown so valid manifests do not retain spare memory.
- Preserve first occurrence semantics, duplicate indices, module/capability pairing, and all lookup
  results exposed by the validation projection.

## Regression Contract

The `optimization_batch_20260826gd_` Runtime tests cover input-sized set capacity and all four source
contracts, and provide an ignored paired release benchmark emitting
`RUNTIME225_FEATURE_VALIDATION_SEEN_CAPACITY_BENCH_V1`. It builds 64 projections with three 4,096-row
seen sets per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
