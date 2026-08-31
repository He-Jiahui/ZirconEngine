# Runtime276 Owned Tooltip Dismiss Result Event

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime276-editor222-performance-batch-20260828id-v1`

## Problem

Runtime accessibility tooltip dismissal received an owned input-dispatch result but cloned its
complete input event before replacing the result through effect application. The original result
was then discarded except for its diagnostic notes, so string-bearing event payloads were copied
without an additional owner that needed to survive.

## Optimization

- Destructure the owned dispatch result and move out its input event and diagnostic notes.
- Apply the tooltip-hide effect with the moved event instead of a cloned event.
- Preserve prior diagnostic notes ahead of notes emitted by effect application.
- Preserve tooltip ownership, handled target, effect routing, and final dismissal diagnostics.

## Regression Contract

The `optimization_batch_20260828id_` Runtime tests prove allocation identity for both the moved
event payload and prior diagnostic note and prevent event cloning from returning. The ignored
paired release benchmark emits `RUNTIME276_OWNED_TOOLTIP_DISMISS_RESULT_EVENT_BENCH_V1`. It converts
512 results with 64-KiB event and note strings per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
