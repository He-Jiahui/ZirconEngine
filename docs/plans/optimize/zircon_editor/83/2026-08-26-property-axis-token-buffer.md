# Editor83 Property Axis Borrowed Token Buffer

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime137-editor83-performance-batch-20260826ct-v1`

## Problem

Retained property-row axis parsing allocated a `String` for every whitespace-delimited value token
and then allocated the joined axis value. Multi-token units and expressions therefore created
several short-lived owners per axis on the paint projection path.

## Optimization

- Reuse one `Vec<&str>` token buffer across all axis groups in a property value.
- Borrow input tokens until `join` creates the single required owned value per emitted axis.
- Preserve whitespace normalization, ignored pre-axis text, empty-axis omission, and X/Y/Z/W
  ordering.

## Regression Contract

The shared `optimization_batch_20260826ct_` filter owns three Editor tests: grouping behavior,
source shape, and an ignored paired release P95 benchmark. The benchmark emits
`EDITOR83_PROPERTY_AXIS_BORROWED_TOKEN_BUFFER_BENCH_V1`, parses 256 three-token axis groups 160
times per sample, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
