# Runtime164 MUI Navigation Borrowed Default Attributes

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime164-editor110-performance-batch-20260826du-v1`

## Problem

MUI navigation class generation cloned ten node and owner string attributes before comparing or
writing them into final class names. Default underline, pagination, tabs, step, and orientation
paths also allocated standalone fallback strings.

## Optimization

- Borrow and trim the first matching TOML string from node or owner attributes.
- Return static defaults without allocation.
- Apply the local helper to seven node defaults and three owner defaults without changing shared
  style compiler APIs.

## Regression Contract

The shared `optimization_batch_20260826du_` filter owns three Runtime tests: alias/default behavior,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME164_MUI_NAVIGATION_BORROWED_DEFAULT_ATTRIBUTES_BENCH_V1`, performs 524,288 lookups per
sample, reduces allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
