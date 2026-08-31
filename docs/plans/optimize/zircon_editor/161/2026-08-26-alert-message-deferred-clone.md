# Editor161 Alert Message Deferred Clone

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime215-editor161-performance-batch-20260826ft-v1`

## Problem

Editor Alert painting cloned the selected message before rejecting invalid horizontal geometry or
a failed message-frame layout, allocating and copying text for commands that were never emitted.

## Optimization

- Keep the selected Alert message borrowed through empty-text, geometry, and layout validation.
- Clone the message exactly once only when constructing the owned wrapped-text paint command.
- Preserve text/value/validation/options fallback order, geometry checks, wrapping style, colors,
  opacity, clipping, and emitted command contents.

## Regression Contract

The `optimization_batch_20260826ft_` Editor tests cover fallback selection, enforce clone placement
after both early-return gates, and provide an ignored paired release benchmark emitting
`EDITOR161_ALERT_MESSAGE_DEFERRED_CLONE_BENCH_V1`. It rejects 8,192 messages of 4,096 bytes per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
