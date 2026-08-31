# Editor117 Popup Action Single Binding Scan

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime171-editor117-performance-batch-20260826eb-v1`

## Problem

Retained popup action projection independently scanned the same binding slice for click binding,
click action, submit action, and change action. Nodes without preferred showcase overrides paid up
to four linear passes before building one action model.

## Optimization

- Collect the first Click, Change, and Submit bindings in one pass.
- Stop once all three event kinds are found.
- Preserve preferred showcase overrides, first-binding priority, and final owned IDs.

## Regression Contract

The shared `optimization_batch_20260826eb_` filter owns three Editor tests: first-binding behavior,
single-loop source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR117_POPUP_ACTION_SINGLE_BINDING_SCAN_BENCH_V1`, uses 256 bindings per scan, reduces primary
lookup passes from four to one, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
