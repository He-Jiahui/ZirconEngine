# Editor127 Host Paint Command Stable Z Sort

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime181-editor127-performance-batch-20260826el-v1`

## Problem

The retained-host template painter paired every fallback command reference with its original index
and sorted by `(z_index, index)`. Rust's stable slice sort already retains input order for equal
z values, so the index doubled temporary element width and added a redundant comparison field.

## Optimization

- Collect only `HostPaintCommand` references in the fallback ordering buffer.
- Stable-sort directly by `z_index` while preserving equal-z paint order.
- Retain the ordered fast path, fallback counter, and existing collect/sort profiling scopes.

## Regression Contract

The shared `optimization_batch_20260826el_` filter owns three Editor tests: equal-z ordering,
index-free source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR127_HOST_PAINT_COMMAND_STABLE_Z_SORT_BENCH_V1`, performs 1,024 sorts of 256 commands per
sample, reduces temporary elements from index-plus-reference to reference-only, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
