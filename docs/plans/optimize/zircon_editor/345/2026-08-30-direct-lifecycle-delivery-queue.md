---
title: Editor345 Direct Lifecycle Delivery Queue
category: zircon_editor
report_id: Editor345-direct-lifecycle-delivery-queue-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor345 Direct Lifecycle Delivery Queue

The plugin lifecycle message pump now consumes freshly drained deliveries directly when no failed
delivery is pending. If a callback rejects a delivery, the current item and the unprocessed suffix
are restored to the lossless queue in original order. A pre-existing pending queue retains priority
and continues to use the established merge-and-retry path.

The previous common path moved every fresh delivery from the bus-owned `Vec` into a `VecDeque`,
then popped every item back out before dispatch. Successful host ticks now avoid that intermediate
queue allocation and full-batch transfer while preserving callback accounting and retry semantics.

The ignored Windows Release benchmark emits
`EDITOR345_DIRECT_LIFECYCLE_DELIVERY_QUEUE_BENCH_V1` over 17 alternating paired samples, each
processing 65,536 successful deliveries, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Editor345 is prepared with Runtime417 under request
`runtime417-editor345-performance-batch-20260830di-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
