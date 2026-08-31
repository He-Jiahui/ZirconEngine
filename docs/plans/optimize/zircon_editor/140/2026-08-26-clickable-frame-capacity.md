# Editor140 Clickable Frame Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime194-editor140-performance-batch-20260826ey-v1`

## Problem

Retained-host profiling assembled seven known clickable-frame collections into a growth-driven
output vector despite knowing every input length before cloning frames.

## Optimization

- Compute the exact combined frame count with saturating additions and allocate the result once.
- Preserve group order, tab-to-named-frame conversion, frame contents, and empty-input behavior.

## Regression Contract

The `optimization_batch_20260826ey_` Editor tests cover a 256-frame seven-group projection and
boundary ordering, source shape, and an ignored paired release benchmark emitting
`EDITOR140_CLICKABLE_FRAME_CAPACITY_BENCH_V1`. It writes 256 lightweight frame entries 2,048 times
per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
