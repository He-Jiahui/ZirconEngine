# Editor162 Chip Label Deferred Clone

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime216-editor162-performance-batch-20260826fu-v1`

## Problem

Editor Chip painting cloned the selected label before the frame builder could reject an
unrenderable chip, allocating and copying text for commands that were never emitted.

## Optimization

- Keep the selected Chip label borrowed through empty-label and frame validation.
- Clone the label exactly once only when constructing the owned text paint command.
- Preserve text-before-value fallback order, frame geometry, typography, color, opacity, clipping,
  and emitted command contents.

## Regression Contract

The `optimization_batch_20260826fu_` Editor tests cover label fallback order, enforce clone
placement after the frame gate, and provide an ignored paired release benchmark emitting
`EDITOR162_CHIP_LABEL_DEFERRED_CLONE_BENCH_V1`. It rejects 8,192 labels of 4,096 bytes per sample and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
