# Editor181 Output Capture Buffer Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime235-editor181-performance-batch-20260826go-v1`

## Problem

Each export-process stdout and stderr polling read grew an empty byte Vec incrementally for large
compiler output chunks. Unconditionally reserving the 64 KiB budget would instead regress the
common empty-output polling path.

## Optimization

- Read a 4 KiB stack prefix before allocating the returned byte Vec.
- Keep empty output heap-allocation free, size small output from its first read, and preallocate the
  existing 64 KiB budget only when the prefix fills.
- Eliminate growth reallocations for full stdout and stderr chunks.
- Preserve the 64 KiB yield boundary, stream ordering, and I/O error reporting.

## Regression Contract

The `optimization_batch_20260826go_` Editor tests perform a real full-budget capture read and enforce
the source contract, and provide an ignored paired release benchmark emitting
`EDITOR181_OUTPUT_CAPTURE_BUFFER_CAPACITY_BENCH_V1`. It reads 256 fixed 64 KiB streams in 4 KiB
blocks per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
