# Editor166 Present Stats Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime220-editor166-performance-batch-20260826fy-v1`

## Problem

Editor GPU presentation telemetry appended a known 46-counter base batch plus up to three optional
timing counters into an empty vector without reserving the exact batch size first.

## Optimization

- Compute the exact appended counter count from timestamp support and optional GPU time.
- Reserve that count before recording the first present statistic, avoiding repeated vector growth.
- Preserve counter ordering, region/full-rebuild selection, optional timestamp semantics, values,
  and the existing 46-counter and 49-counter contracts.

## Regression Contract

The `optimization_batch_20260826fy_` Editor tests cover exact base/full counts, resulting capacity,
and reserve-before-recording order, and provide an ignored paired release benchmark emitting
`EDITOR166_PRESENT_STATS_CAPACITY_BENCH_V1`. It builds 8,192 full 49-counter batches per sample and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
