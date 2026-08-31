# Runtime221 Native Manifest Projection Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime221-editor167-performance-batch-20260826fz-v1`

## Problem

Native plugin discovery projected the ordered manifest index into a candidate Vec grown from empty,
although the manifest index size already bounds and commonly equals the emitted candidate count.

## Optimization

- Preallocate the candidate output Vec from the indexed manifest count.
- Leave duplicate diagnostics demand-grown because their count is independent of valid candidates.
- Preserve deterministic manifest-path order, first package-id winner selection, duplicate messages,
  candidate cloning, and returned tuple shape; duplicate package IDs may leave spare capacity.

## Regression Contract

The `optimization_batch_20260826fz_` Runtime tests cover output capacity, values, and the production
reserve contract, and provide an ignored paired release benchmark emitting
`RUNTIME221_NATIVE_MANIFEST_PROJECTION_CAPACITY_BENCH_V1`. It builds 64 projections containing 4,096
candidate-sized payloads per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
