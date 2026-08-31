# Editor97 Target Mode Direct Join

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime151-editor97-performance-batch-20260826dh-v1`

## Problem

Module-plugin project-policy status projection collected fixed target-mode labels into a temporary
`Vec<&str>` before joining them into the owned status string. Every policy refresh allocated both
the temporary vector and the required result.

## Optimization

- Compute exact comma-separated capacity from fixed target-mode names.
- Append labels and separators directly into the result string.
- Preserve empty `all`, ordering, and repeated target-mode behavior.

## Regression Contract

The shared `optimization_batch_20260826dh_` filter owns three Editor tests: output behavior,
exact-capacity source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR97_TARGET_MODE_DIRECT_JOIN_BENCH_V1`, renders 262,144 status labels per sample, records
temporary-vector allocations from 262,144 to zero while retaining one result allocation, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
