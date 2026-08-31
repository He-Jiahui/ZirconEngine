# Editor88 Popup Action Segment In-Place Trim

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime142-editor88-performance-batch-20260826cy-v1`

## Problem

Popup menu action generation built a snake-case label buffer, then used
`trim_matches('_').to_string()` to allocate and copy a second buffer. Leading separators were
written only to be trimmed and a trailing separator forced the full copy for each generated item.

## Optimization

- Reserve the action-segment buffer from the label byte length.
- Avoid emitting leading separators while retaining separator collapse and CamelCase boundaries.
- Remove the single collapsed trailing underscore in place and return the original allocation.
- Preserve punctuation-only, empty, non-ASCII, digit, existing underscore, and mixed-case output.

## Regression Contract

The shared `optimization_batch_20260826cy_` filter owns three Editor tests: legacy-output parity,
single-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR88_POPUP_ACTION_SEGMENT_IN_PLACE_TRIM_BENCH_V1`, normalizes 8,192 representative popup
labels, records the per-label allocation reduction from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
