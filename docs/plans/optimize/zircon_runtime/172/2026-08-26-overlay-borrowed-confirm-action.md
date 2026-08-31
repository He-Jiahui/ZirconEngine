# Runtime172 Overlay Borrowed Confirm Action

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime172-editor118-performance-batch-20260826ec-v1`

## Problem

Dialog commit handling cloned the configured confirmation action into a new `String` merely to
compare it with the incoming action. This added one heap allocation to every custom confirm-action
comparison even though component state and descriptor defaults already outlive the comparison.

## Optimization

- Borrow string and enum settings from component state or descriptor defaults.
- Preserve state-over-default precedence and the canonical `confirm` alias.
- Materialize an owned string only where the committed action must be stored in component state.

## Regression Contract

The shared `optimization_batch_20260826ec_` filter owns three Runtime tests: value/default
semantics, borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `RUNTIME172_OVERLAY_BORROWED_CONFIRM_ACTION_BENCH_V1`, performs 131,072 comparisons
per sample, reduces comparison allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
