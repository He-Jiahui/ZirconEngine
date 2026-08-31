# Runtime137 Inspection Field Hash Delta

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime137-editor83-performance-batch-20260826ct-v1`

## Problem

Each focused inspector artifact delta built two `BTreeMap` indexes over current and previous field
paths. These maps were used only for equality and membership lookup; changed and removed output
ordering already came from the original field slices.

## Optimization

- Build both borrowed `(component_type_path, field_name)` indexes as `HashMap` values.
- Keep keys borrowed, avoiding path copies while changing lookup complexity.
- Preserve current-slice changed-field order, previous-slice removed-field order, generations, and
  entity identity.

## Regression Contract

The shared `optimization_batch_20260826ct_` filter owns three Runtime tests: delta behavior and
order, source shape, and an ignored paired release P95 benchmark. The benchmark emits
`RUNTIME137_INSPECTION_FIELD_HASH_DELTA_BENCH_V1`, indexes and probes 16,384 fields per side, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
