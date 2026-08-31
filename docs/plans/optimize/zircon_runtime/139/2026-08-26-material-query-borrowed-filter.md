# Runtime139 Material Query Borrowed Filter

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime139-editor85-performance-batch-20260826cv-v1`

## Problem

Every material-management query trimmed and allocated an ASCII-lowercase copy of its text filter.
The downstream substring matcher already uses `eq_ignore_ascii_case`, so the allocated normalized
buffer duplicated work without changing matching behavior.

## Optimization

- Return the non-empty trimmed filter as a borrowed `&str`.
- Feed that slice directly into the existing ASCII case-insensitive matcher.
- Preserve empty-filter handling, material name/id matching, record sorting, indexes, and paging.

## Regression Contract

The shared `optimization_batch_20260826cv_` filter owns three Runtime tests: matching behavior,
trimmed-storage reuse, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME139_MATERIAL_QUERY_BORROWED_FILTER_BENCH_V1`, normalizes 16,384 representative filters,
records the reduction from 16,384 allocations to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
