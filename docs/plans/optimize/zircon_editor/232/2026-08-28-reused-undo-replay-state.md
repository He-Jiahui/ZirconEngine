# Editor232 Reused Undo Replay State

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime286-editor232-performance-batch-20260828in-v1`

## Problem

UI asset undo replay replaced retained selection, cursor, theme, and style-rule state with fresh
clones. Repeated undo/redo discarded Vec and nested String capacities even when the destination
already had compatible storage.

## Optimization

- Clone selection fields individually so the sibling Vec and nested strings reuse capacity.
- Reuse cursor-anchor and optional theme/style strings while preserving changed flags.
- Preserve source/document replay ordering and external-effect application.

## Regression Contract

The `optimization_batch_20260828in_` Editor tests prove nested allocation identity and prevent
whole-state replacing clones from returning. The ignored paired release benchmark emits
`EDITOR232_REUSED_UNDO_REPLAY_STATE_BENCH_V1`. It performs 4,096 alternating updates of 128
fixed-width node identifiers per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
