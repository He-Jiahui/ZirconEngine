# Editor143 Profile Hit Sample Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime197-editor143-performance-batch-20260826fb-v1`

## Problem

Retained-host profiling emits exactly three hit samples for every clickable frame but collected
both per-frame and combined sample vectors through implicit growth.

## Optimization

- Reserve three entries for each per-frame sample vector.
- Reserve `frame_count * 3` with saturating arithmetic for the combined sample list and preserve
  center, outside-left, outside-bottom ordering.

## Regression Contract

The `optimization_batch_20260826fb_` Editor tests cover the three-sample constant, zero, 256-frame,
and saturation capacity math, source shape, and an ignored paired release benchmark emitting
`EDITOR143_PROFILE_HIT_SAMPLE_CAPACITY_BENCH_V1`. It writes 768 lightweight sample entries 683
times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
