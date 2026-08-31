# Runtime217 Mesh Management Record Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime217-editor163-performance-batch-20260826fv-v1`

## Problem

Mesh management result projection sorted a complete result batch and then appended successful
records into a vector grown from empty, despite the batch length being an exact upper bound and the
normal asset-management path producing predominantly successful records.

## Optimization

- Capture result count before consuming the sorted batch and reserve it for successful records.
- Leave the uncommon failure vector demand-grown to avoid reserving a second full batch.
- Preserve mesh-id sorting, success/failure partitioning, diagnostics, summaries, and output order;
  invalid results only leave spare record capacity.

## Regression Contract

The `optimization_batch_20260826fv_` Runtime tests cover empty-set summary behavior, enforce the
production reservation contract, and provide an ignored paired release benchmark emitting
`RUNTIME217_MESH_MANAGEMENT_RECORD_CAPACITY_BENCH_V1`. It builds 64 sets of 4,096 record-sized
fixtures per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
