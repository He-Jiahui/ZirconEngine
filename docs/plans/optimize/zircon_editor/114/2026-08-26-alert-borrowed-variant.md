# Editor114 Alert Borrowed Variant

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime168-editor114-performance-batch-20260826dy-v1`

## Problem

Retained Alert variant projection cloned an authored variant string before appending it to the one
final variant buffer. The temporary owned value did not escape the append operation.

## Optimization

- Borrow the selected Alert variant directly from TOML attributes.
- Return the static `standard` fallback without allocation.
- Preserve first-present-key precedence and final alert token behavior.

## Regression Contract

The shared `optimization_batch_20260826dy_` filter owns three Editor tests: alias/default behavior,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR114_ALERT_BORROWED_VARIANT_BENCH_V1`, performs 524,288 lookups per sample, reduces lookup
allocations from one to zero, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
