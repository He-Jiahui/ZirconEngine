# Editor119 Page Overflow Focus Single Scan

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime173-editor119-performance-batch-20260826ed-v1`

## Problem

Host-page overflow keyboard projection built every visible row, then traversed the completed row
vector again to locate the first focused or selected row. Large tab sets paid two full passes on
every target reconstruction.

## Optimization

- Build overflow rows with one explicit traversal.
- Record the first focused/selected index while each valid row is appended.
- Use the collected row count so skipped or stale page indices cannot shift keyboard focus.

## Regression Contract

The shared `optimization_batch_20260826ed_` filter owns three Editor tests: first-index semantics,
single-pass source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR119_PAGE_OVERFLOW_FOCUS_SINGLE_SCAN_BENCH_V1`, processes 256 rows 8,192 times per sample,
reduces traversal passes from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
