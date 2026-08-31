# Runtime166 Tab Panel Borrowed Value Comparison

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime166-editor112-performance-batch-20260826dw-v1`

## Problem

MUI TabPanel visibility cloned both the authored tab value and the selected context value before a
single equality comparison. Neither owned string escaped the mismatch predicate.

## Optimization

- Borrow and trim the first matching current and selected tab values.
- Preserve alias priority, whitespace-only filtering, and missing-value behavior.
- Keep the helper local to the MUI lab class owner.

## Regression Contract

The shared `optimization_batch_20260826dw_` filter owns three Runtime tests: alias/mismatch
behavior, borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `RUNTIME166_TAB_PANEL_BORROWED_VALUE_COMPARISON_BENCH_V1`, performs 524,288
comparisons per sample, reduces comparison allocations from two to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
