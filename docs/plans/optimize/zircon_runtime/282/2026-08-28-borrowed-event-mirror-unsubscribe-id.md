# Runtime282 Borrowed Event-Mirror Unsubscribe ID

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime282-editor228-performance-batch-20260828ij-v1`

## Problem

Runtime event-mirror unsubscribe converted the subscription record event ID into a new String
before cloning its registration. The cloned registration already retained the same stable event ID,
so every unsubscribe paid for an unnecessary allocation and copy.

## Optimization

- Resolve the registration directly from the record's borrowed event ID.
- Borrow the stable ID from the retained registration for decrement and rollback operations.
- Preserve disconnect/reconnect ordering, reader callbacks, subscription restoration, and errors.

## Regression Contract

The `optimization_batch_20260828ij_` Runtime tests prove descriptor allocation identity and prevent
the record-ID `to_string` from returning. The ignored paired release benchmark emits
`RUNTIME282_BORROWED_EVENT_MIRROR_UNSUBSCRIBE_ID_BENCH_V1`. It processes 32,768 representative
event IDs per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
