# Runtime312 Preallocated SDF Native Fallback Indices

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime312-editor257-performance-batch-20260829al-v1`

## Problem

Once an SDF text batch required fallback, native fallback run indices still started in an empty
vector even though the pending batch count was already known. Frames with broad atlas failure grew
and copied the index buffer repeatedly.

## Optimization

- Preallocate native fallback run indices to the pending SDF batch count.
- Keep the no-fallback early return unchanged, so normal frames do not allocate this buffer.
- Preserve fallback ordering, report accounting, and retained SDF run behavior.

## Regression Contract

The `optimization_batch_20260829al_` Runtime tests verify the exact full-batch capacity and guard
the production preallocation. The ignored paired release benchmark emits
`RUNTIME312_PREALLOCATED_SDF_NATIVE_FALLBACK_INDICES_BENCH_V1`. It builds 40,000 256-run index
buffers per sample, changes seven vector allocation operations to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
