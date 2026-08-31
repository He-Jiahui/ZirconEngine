# Runtime268 Pending Discovery Work Move

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime268-editor214-performance-batch-20260828hv-v1`

## Problem

The native plugin discovery service retained an owned manifest work batch for the active
generation. When the first superseding generation arrived, it deeply cloned the full path map and
notification-order vector even though the active generation never read that retained work again.

## Optimization

- Store the active generation's retained work in a one-shot ownership slot.
- Move that work into the first pending generation before merging the newest watcher work.
- Preserve the independent worker copy, latest-wins merging, cancellation, publication, and
  subsequent pending-generation launch behavior.

## Regression Contract

The `optimization_batch_20260828hv_` Runtime tests prove that the notification-order allocation is
transferred without copying and enforce the clone-free supersede source contract. The ignored
paired release benchmark emits `RUNTIME268_PENDING_DISCOVERY_WORK_MOVE_BENCH_V1`. It transfers a
1,024-path manifest batch with 512-byte path components sixteen times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
