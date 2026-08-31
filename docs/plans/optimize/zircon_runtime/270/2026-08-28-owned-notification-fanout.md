# Runtime270 Owned Notification Fanout

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime270-editor216-performance-batch-20260828hx-v1`

## Problem

UI notification broadcast cloned the complete notification for every subscriber and then dropped the
owned input. A single subscriber therefore paid for a deep clone of invocation values, bindings, and
errors even though broadcast already owned the notification.

## Optimization

- Iterate subscription senders from both ends and reserve the final sender for owned delivery.
- Clone the notification only for the preceding N-1 subscribers.
- Preserve deterministic subscription order, disconnected-sender behavior, and notification values.

## Regression Contract

The `optimization_batch_20260828hx_` Runtime tests prove that a single subscriber receives the
original allocation and that a two-subscriber fanout clones only the first delivery. The ignored
paired release benchmark emits `RUNTIME270_OWNED_NOTIFICATION_FANOUT_BENCH_V1`. It broadcasts 128
invocation notifications with 64 KiB string values per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
