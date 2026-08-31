# Runtime154 Shader IDE Path Direct Join

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime154-editor100-performance-batch-20260826dk-v1`

## Problem

Shader IDE artifact path projection converted every path component to lossy text, collected all
component values into a temporary vector, and then allocated the required slash-separated string.
Module maps and preview artifacts repeatedly use this projection while refreshing generated IDE
state.

## Optimization

- Reserve one result buffer from the source path's encoded byte length.
- Append lossy component text and slash separators directly during a single conversion pass.
- Preserve `Path::components()` normalization, empty paths, component order, and portable `/` output.

## Regression Contract

The shared `optimization_batch_20260826dk_` filter owns three Runtime tests: path behavior,
direct-append source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME154_SHADER_IDE_PATH_DIRECT_JOIN_BENCH_V1`, renders 16,384 paths with 32 components per
sample, removes one temporary component vector per path, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
