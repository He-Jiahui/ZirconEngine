# Editor201 Reused Overlay Vector Storage

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime255-editor201-performance-batch-20260826hi-v1`

## Problem

Every viewport interaction projection collected handle and scene-gizmo slices into new vectors,
then discarded the previous vectors and their capacities. Stable editor frames therefore repeated
two backing-storage allocations even when overlay cardinality stayed unchanged.

## Optimization

- Clear the retained overlay vectors without releasing their backing storage.
- Extend each vector from its corresponding immutable interaction slice.
- Preserve element cloning, ordering, and complete replacement semantics.

## Regression Contract

The `optimization_batch_20260826hi_` Editor tests preserve replacement contents and retained
capacity; enforce both overlay paths using the shared clear-and-extend helper; and provide an
ignored paired release benchmark emitting `EDITOR201_REUSED_OVERLAY_VECTOR_STORAGE_BENCH_V1`. It
replaces 16,384 elements 128 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
