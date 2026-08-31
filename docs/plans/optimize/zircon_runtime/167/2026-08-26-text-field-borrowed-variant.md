# Runtime167 Text Field Borrowed Variant

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime167-editor113-performance-batch-20260826dx-v1`

## Problem

MUI TextField and owned slot class generation cloned the validated `variant` attribute on each of
five call paths. Callers only matched or formatted the value and never retained the clone.

## Optimization

- Borrow and trim the first non-empty TextField variant attribute.
- Return the static `outlined` fallback for missing or unsupported values.
- Update the three match sites to consume the borrowed `str` directly.

## Regression Contract

The shared `optimization_batch_20260826dx_` filter owns three Runtime tests: alias/default
behavior, borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `RUNTIME167_TEXT_FIELD_BORROWED_VARIANT_BENCH_V1`, performs 524,288 lookups per
sample, reduces lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
