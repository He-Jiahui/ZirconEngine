# Editor151 Asset Refresh Drain Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime205-editor151-performance-batch-20260826fj-v1`

## Problem

Editor asset, editor-asset, and resource refresh queues each drained into an empty vector even
though their pending counts and the per-stream limit were available before the drain loops.

## Optimization

- Reserve `min(pending_count, 256)` independently for all three refresh streams.
- Preserve zero allocation for empty streams, the per-stream count and time budgets, queue-age
  accounting, lag handling, and pending-backlog reporting.

## Regression Contract

The `optimization_batch_20260826fj_` Editor tests cover empty, partial, full, and over-limit stream
capacities, enforce all three production queue connections, and provide an ignored paired release
benchmark emitting `EDITOR151_ASSET_REFRESH_STREAM_CAPACITY_BENCH_V1`. It fills three 256-event
streams 2,048 times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
