# Runtime162 Material Query Borrowed Filter Check

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime162-editor108-performance-batch-20260826ds-v1`

## Problem

`RenderMaterialManagementQuery::has_active_filters` normalized its optional text filter into a new
owned string only to call `is_some`. Repeated material-management state checks therefore allocated
and copied a trimmed filter even though no owned value escaped the predicate.

## Optimization

- Add a borrowed normalization helper that trims and rejects empty text without allocation.
- Use the borrowed helper in the boolean hot check.
- Keep the owned normalization path for serialized query-state snapshots.

## Regression Contract

The shared `optimization_batch_20260826ds_` filter owns three Runtime tests: normalization behavior,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME162_MATERIAL_QUERY_BORROWED_FILTER_CHECK_BENCH_V1`, performs 524,288 checks per sample,
reduces allocations per check from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
