# Runtime306 Allocation-Free Scene Binding Root Advance

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime306-editor252-performance-batch-20260829ag-v1`

## Problem

Scene binding generation invalidation collected every root into a temporary vector only to detect
empty input before assigning one generation to the roots. Both ordinary and replacement-world
advances paid for this adapter allocation.

## Optimization

- Convert the incoming root iterator to `Peekable` without materializing it.
- Preserve the empty-input early return by peeking once.
- Consume the same iterator directly while assigning the shared generation.

## Regression Contract

The `optimization_batch_20260829ag_` Runtime tests cover empty input, ordinary root generations,
replacement-world generations, and both production source contracts. The ignored paired release
benchmark emits `RUNTIME306_ALLOCATION_FREE_SCENE_BINDING_ROOT_ADVANCE_BENCH_V1`. It advances an
eight-root set 200,000 times per sample, reduces adapter allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
