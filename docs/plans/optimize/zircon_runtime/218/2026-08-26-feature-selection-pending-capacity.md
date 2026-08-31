# Runtime218 Feature Selection Pending Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime218-editor164-performance-batch-20260826fw-v1`

## Problem

Runtime plugin feature partitioning first materialized every active selection, then appended the
catalog-backed majority into a second vector grown from empty even though the active count was an
immediate upper bound.

## Optimization

- Retain the active-selection vector and reserve its length for pending catalog-backed selections.
- Leave unknown feature blocks demand-grown to avoid reserving a second full selection batch.
- Preserve active-selection order, definition lookup, unknown blocking details, required flags,
  owner identities, and partition contents; unknown features only leave spare pending capacity.

## Regression Contract

The `optimization_batch_20260826fw_` Runtime tests cover capacity and order, enforce the production
reservation contract, and provide an ignored paired release benchmark emitting
`RUNTIME218_FEATURE_SELECTION_PENDING_CAPACITY_BENCH_V1`. It builds 64 partitions of 4,096
pending-sized fixtures per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
