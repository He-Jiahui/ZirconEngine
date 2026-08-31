# Editor178 Viewport Snap Label Single-Pass Formatting

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime232-editor178-performance-batch-20260826gl-v1`

## Problem

Viewport chrome formatted each snap value into a temporary String and then formatted that String
again with its T, R, or S prefix, causing six String allocations for three labels per projection.

## Optimization

- Format the prefix and precision-selected numeric value directly into each final label.
- Reduce String allocation count from six to three for every viewport chrome projection.
- Preserve integral, one-decimal, and two-decimal display text exactly.

## Regression Contract

The `optimization_batch_20260826gl_` Editor tests cover all three precision branches and enforce
single-pass formatting, and provide an ignored paired release benchmark emitting
`EDITOR178_VIEWPORT_SNAP_LABEL_SINGLE_PASS_FORMATTING_BENCH_V1`. It formats 131,072 three-label
chrome states per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
