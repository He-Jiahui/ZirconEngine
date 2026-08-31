# Editor164 Badge Display Borrow

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime218-editor164-performance-batch-20260826fw-v1`

## Problem

Editor Badge overlay projection cloned its value/validation display text before geometry checks and
cloned it again into the final paint command. Invalid overlay rectangles paid the first copy without
emitting text.

## Optimization

- Return the trimmed overlay display as a borrow while retaining the root-label `String` contract.
- Keep the existing ownership conversion at the final overlay text paint command.
- Preserve value-before-validation fallback, trimming, dot behavior, visibility and finite-geometry
  gates, typography, colors, clipping, opacity, and emitted command text.

## Regression Contract

The `optimization_batch_20260826fw_` Editor tests cover fallback/trim behavior and the display-only
borrow contract, and provide an ignored paired release benchmark emitting
`EDITOR164_BADGE_DISPLAY_BORROW_BENCH_V1`. It draws 8,192 labels of 4,096 bytes per sample and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
