# Editor111 Popup Placement Borrowed Check

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime165-editor111-performance-batch-20260826dv-v1`

## Problem

Retained popup overlay classification cloned the `placement` TOML string only to test whether it
contained a direction separator. The owned value was discarded immediately after the predicate.

## Optimization

- Borrow the placement string directly from the attribute map.
- Preserve component-role precedence and separator classification behavior.
- Keep the helper private to popup overlay projection.

## Regression Contract

The shared `optimization_batch_20260826dv_` filter owns three Editor tests: role/separator
behavior, borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `EDITOR111_POPUP_PLACEMENT_BORROWED_CHECK_BENCH_V1`, performs 524,288 lookups per
sample, reduces lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
