# Editor93 Drag Payload Direct Join

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime147-editor93-performance-batch-20260826dd-v1`

## Problem

Popup-action projection collected accepted drag payload names into a temporary `Vec<&str>` before
joining them into the owned protocol string. Every retained pane conversion therefore allocated
both the temporary vector and the required output string.

## Optimization

- Compute the exact comma-separated output capacity from the fixed payload names.
- Append payload names and delimiters directly into the result string.
- Preserve empty, ordered, and duplicate payload policy serialization.

## Regression Contract

The shared `optimization_batch_20260826dd_` filter owns three Editor tests: protocol behavior,
single-result-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `EDITOR93_DRAG_PAYLOAD_DIRECT_JOIN_BENCH_V1`, performs 262,144 joins per sample, records
temporary-vector allocations from 262,144 to zero while retaining one result allocation, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
