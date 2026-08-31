# Editor254 Single-Allocation Move Handle Overlay

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime308-editor254-performance-batch-20260829ai-v1`

## Problem

Move-handle overlay extraction creates exactly three axis-line elements and one center anchor, but
started with an empty vector. The repeated pushes caused capacity growth during every overlay
build in the viewport path.

## Optimization

- Reserve the fixed four-element overlay capacity before building the axis lines.
- Preserve element order, values, and the existing optional-selection behavior.

## Regression Contract

The `optimization_batch_20260829ai_` Editor tests guard the fixed-capacity source contract and the
four-element builder shape. The ignored paired release benchmark emits
`EDITOR254_SINGLE_ALLOCATION_MOVE_HANDLE_OVERLAY_BENCH_V1`. It builds 200,000 four-element buffers
per sample, reduces buffer growth operations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
