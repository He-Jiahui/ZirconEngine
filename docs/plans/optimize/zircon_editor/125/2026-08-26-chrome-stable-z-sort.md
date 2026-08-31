# Editor125 Chrome Stable Z Sort

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime179-editor125-performance-batch-20260826ej-v1`

## Problem

When chrome commands arrived out of z order, retained-host replay paired every command reference with
its original index and sorted by `(z_index, index)`. Rust's stable sort already preserves original
order for equal z values, making the index storage and comparison redundant.

## Optimization

- Collect only command references in the fallback ordering buffer.
- Stable-sort directly by `z_index` and rely on the stable ordering contract for equal z values.
- Preserve the existing already-sorted zero-allocation path and fallback performance counter.

## Regression Contract

The shared `optimization_batch_20260826ej_` filter owns three Editor tests: equal-z stable behavior,
index-free source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR125_CHROME_STABLE_Z_SORT_BENCH_V1`, performs 1,024 sorts of 256 commands per sample, reduces
temporary elements from index-plus-reference to reference-only, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
