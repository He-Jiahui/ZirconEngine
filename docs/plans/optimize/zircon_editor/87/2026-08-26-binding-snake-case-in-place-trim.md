# Editor87 Binding Snake-Case In-Place Trim

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime141-editor87-performance-batch-20260826cx-v1`

## Problem

Component-showcase binding lookup built a snake-case `String`, then called
`trim_matches('_').to_string()` to allocate and copy a second result. Leading separators were first
written only to be trimmed, and a trailing separator forced the full copy.

## Optimization

- Reserve the output buffer from the binding identifier byte length.
- Avoid emitting leading separators while retaining separator collapse and CamelCase boundaries.
- Remove the single collapsed trailing underscore in place and return the original buffer.
- Preserve empty, punctuation-only, non-ASCII, digit, existing underscore, and mixed-case output.

## Regression Contract

The shared `optimization_batch_20260826cx_` filter owns three Editor tests: legacy-output parity,
single-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR87_BINDING_SNAKE_CASE_IN_PLACE_TRIM_BENCH_V1`, normalizes 8,192 representative binding IDs,
records the per-binding allocation reduction from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
