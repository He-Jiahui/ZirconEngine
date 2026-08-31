# Runtime155 WGSL Include Strip Single Buffer

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime155-editor101-performance-batch-20260826dl-v1`

## Problem

WGSL include preprocessing collected every retained source line into a temporary `Vec<&str>` and
then allocated the required stripped shader source. Shader import refresh repeats this work for
every module before downstream compilation.

## Optimization

- Reserve the stripped output from the source length and append retained lines directly.
- Track whether any line was emitted independently from output length, preserving leading blanks.
- Preserve `str::lines()` trailing-newline behavior and complete include-only output.

## Regression Contract

The shared `optimization_batch_20260826dl_` filter owns three Runtime tests: line behavior,
single-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME155_WGSL_INCLUDE_STRIP_SINGLE_BUFFER_BENCH_V1`, strips 2,048 sources with 256 lines per
sample, removes one temporary retained-line vector per strip, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
