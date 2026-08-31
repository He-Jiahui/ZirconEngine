# Editor173 Window Minimum Width Stack Constraints

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime227-editor173-performance-batch-20260826gf-v1`

## Problem

Editor workbench minimum-width computation allocated a Vec on every layout pass to hold only the
visible left, required document, and visible right region constraints.

## Optimization

- Select one of four fixed stack slices for the left/right visibility combination and aggregate it
  in the original left-document-right order.
- Derive separator count directly from the number of visible side regions.
- Preserve constraint aggregation, compact-window clamping, document inclusion, and every
  visibility combination while eliminating the temporary heap allocation.

## Regression Contract

The `optimization_batch_20260826gf_` Editor tests compare all four visibility combinations with the
legacy Vec result and enforce stack-slice construction, and provide an ignored paired release
benchmark emitting `EDITOR173_WINDOW_MINIMUM_WIDTH_STACK_CONSTRAINTS_BENCH_V1`. It computes 262,144
window widths per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
