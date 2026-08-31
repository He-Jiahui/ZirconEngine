# Editor116 Surface Borrowed App Bar Color

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime170-editor116-performance-batch-20260826ea-v1`

## Problem

Retained AppBar surface projection first cloned the color attribute and then allocated the mapped
surface string. The intermediate color ownership was unnecessary.

## Optimization

- Borrow AppBar color directly from TOML attributes.
- Preserve static `primary` fallback and existing surface mapping.
- Retain only the allocation required by the final owned surface field.

## Regression Contract

The shared `optimization_batch_20260826ea_` filter owns three Editor tests: color mapping, borrowed
pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR116_SURFACE_BORROWED_APP_BAR_COLOR_BENCH_V1`, performs 524,288 mappings per sample, reduces
allocations from two to one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
