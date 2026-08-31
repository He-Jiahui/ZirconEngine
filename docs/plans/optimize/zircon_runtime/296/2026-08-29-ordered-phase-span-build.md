# Runtime296 Ordered Phase-Span Build

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime296-editor242-performance-batch-20260829w-v1`

## Problem

Every render-queue summary rebuilt the fixed phase-order spans by linearly searching every span
already emitted. The 13 phases are already stored in queue order, but the builder performed 58
phase-order comparisons, grew the outer vector incrementally, and rebuilt three diagnostic names
after discovering the second phase in a shared order bucket.

## Optimization

- Partition the canonical phase array into adjacent equal-order slices once per process.
- Reserve the template's outer span vector once from the canonical phase count.
- Clone the immutable template into an independent mutable summary skeleton for each queue.
- Preserve phase order, shared-order grouping, diagnostics, and summary lookup behavior.

## Regression Contract

The `optimization_batch_20260829w_` Runtime tests verify all 13 phases flatten back to the
canonical order, the 10 order buckets remain strictly ordered, and diagnostic names match their
final phase groups. The ignored paired release benchmark emits
`RUNTIME296_ORDERED_PHASE_SPAN_BUILD_BENCH_V1`. It performs 20,000 complete span builds per sample,
reduces steady-state phase-order comparisons per build from 58 to zero and diagnostic builds from
13 to zero after one process-wide template initialization, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
