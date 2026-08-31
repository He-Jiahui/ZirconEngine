# Editor156 Watcher Deferred Path Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime210-editor156-performance-batch-20260826fo-v1`

## Problem

When an Editor asset-watch poll exhausted its time budget, it rebuilt the remaining pending-path
batch from an empty vector despite the exact `IntoIter` remainder being available at that branch.

## Optimization

- Allocate the deferred batch once from the remaining iterator length plus the first rejected path.
- Preserve FIFO ordering, first-seen timestamps, ingress restoration, budget accounting, and the
  allocation-free normal path where no event is deferred.

## Regression Contract

The `optimization_batch_20260826fo_` Editor tests preserve 128 deferred paths and timestamps in
order, enforce the production source shape, and provide an ignored paired release benchmark
emitting `EDITOR156_WATCHER_DEFERRED_PATH_CAPACITY_BENCH_V1`. It rebuilds 4,096 batches of 64
pending-path-sized fixtures per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
