# Editor248 Single-Pass Feature Action Selection

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime302-editor248-performance-batch-20260829ac-v1`

## Problem

The module-plugin feature row searched the same feature slice up to three times to select its
highest-priority action. Rows whose only action was a late disable candidate paid three complete
linear scans before formatting the result.

## Optimization

- Select enable and disable fallback candidates during one traversal.
- Keep immediate return for the highest-priority dependency action.
- Preserve first-match order and dependency, enable, then disable priority.

## Regression Contract

The `optimization_batch_20260829ac_` Editor tests cover priority and first-match behavior and
guard the single-scan source contract. The ignored paired release benchmark emits
`EDITOR248_SINGLE_PASS_FEATURE_ACTION_SELECTION_BENCH_V1`. It selects from 2,048 features 1,000
times per sample, reduces representative visits from 6,144 to 2,048 per action, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
