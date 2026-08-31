# Runtime184 SSS Diagnostic Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime184-editor130-performance-batch-20260826eo-v1`

## Problem

Runtime subsurface-profile resolution supports 16 GPU slots. Any input after 16 must produce an
overflow or duplicate diagnostic, but large profile batches still grew the diagnostic vector from
zero and repeatedly moved its entries.

## Optimization

- Preserve a zero-capacity diagnostic vector for the normal 16-or-fewer profile path.
- For batches above the GPU-slot limit, allocate once to the input-count diagnostic upper bound.
- Preserve first-slot ownership, active-mask generation, sparse slot filling, diagnostic order,
  text, and profile-table output.

## Regression Contract

The shared `optimization_batch_20260826eo_` filter owns three Runtime tests: 256-profile overflow
behavior, conditional-capacity source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `RUNTIME184_SSS_DIAGNOSTIC_CAPACITY_BENCH_V1`, writes 256 real
`SubsurfaceProfileDiagnostic` values 2,048 times per sample, replaces growth-driven allocation
with one allocation, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
