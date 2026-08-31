---
title: Editor49 Event Retention Hash Delivery Queue
category: zircon_editor
report_id: Editor49-event-retention-hash-delivery-queue-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor49 Event Retention Hash Delivery Queue

## Scope

This slice replaces retention payload storage by delivery cursor from `BTreeMap` to `HashMap` and
makes arrival order explicit with `VecDeque`. Delivery cursors remain monotonically assigned by the
store. Page lookup uses partition points over the deque's at most two contiguous slices; event
sequence and age order remain owned by their existing `BTreeSet` indexes.

Latest-state coalescing removes payloads immediately and leaves stale delivery cursors for cheap
amortized cleanup. The deque compacts when it exceeds twice the live payload count plus a named
64-cursor slack, keeping pointer-move storms bounded without paying an ordered-map mutation for
every retained payload.

## Performance Workload

The release workload inserts 65,536 monotonically assigned delivery cursors into the legacy and
optimized payload indexes.

| Work per workload | Before | After |
|---|---:|---:|
| Ordered payload insertions | 65,536 | 0 |
| Hash payload insertions | 0 | 65,536 |
| Explicit deque appends | 0 | 65,536 |
| Age and event-sequence indexes | unchanged | unchanged |

The ignored release gate runs 21 alternating sample pairs and emits
`EDITOR49_EVENT_RETENTION_HASH_DELIVERY_QUEUE_BENCH_V1`. Acceptance requires hash payload indexing
and explicit order tracking P95 to be at least 30% below the legacy `BTreeMap` payload index. Exact
Windows P50/P95 timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826cj_editor_hash_delivery_queue_preserves_pages_and_coalescing` covers
  cursor pages, acknowledgement, latest-state replacement, and bounded stale-cursor compaction.
- `optimization_batch_20260826cj_editor_retention_payload_index_is_hash_based_and_order_explicit`
  locks the hash payload owner and explicit delivery-order deque.
- `optimization_batch_20260826cj_editor_retention_hash_delivery_queue_release_benchmark` reports
  paired release P50/P95 samples and enforces the 30% P95 reduction gate.

## Remaining Parent-plan Work

Editor49 still owns replay safety, listener ownership, revision semantics, backpressure,
observability, and product-scale qualification. This slice only converges the retained payload
index and preserves all existing delivery, age, and event-sequence contracts.
