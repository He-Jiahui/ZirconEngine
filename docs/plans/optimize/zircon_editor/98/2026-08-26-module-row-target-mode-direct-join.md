# Editor98 Module Row Target Mode Direct Join

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime152-editor98-performance-batch-20260826di-v1`

## Problem

Module-plugin pane projection collected borrowed target-mode labels into a temporary `Vec<&str>`
before joining the required owned row summary. Every status refresh therefore allocated a
short-lived vector in addition to the result string.

## Optimization

- Reserve one result buffer from the fixed maximum target-mode label width.
- Append labels and separators directly in report order.
- Preserve empty, repeated, and ordered target-mode output.

## Regression Contract

The shared `optimization_batch_20260826di_` filter owns three Editor tests: output behavior,
direct-join source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR98_MODULE_ROW_TARGET_MODE_DIRECT_JOIN_BENCH_V1`, renders 8,192 summaries containing 32
modes per sample, removes 8,192 temporary-vector allocations, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
