# Editor Event Listener Audit

## Current Hard-Cut Contract

- External consumers register a listener, configure `SetEnabled` or `SetFilter`, and inspect it
  with `QueryListenerStatus`.
- The sole delivery read request is
  `QueryDeliveriesPage { listener_id, after_delivery_cursor, max_deliveries }`. Page size is
  required to be in `1..=256`.
- Every delivery exposes its listener-local `delivery_cursor`. A nonempty page returns the final
  cursor as `next_delivery_cursor`; `has_more` indicates whether an immediate successor page is
  available. An empty page returns no continuation.
- Consumers preserve the last observed delivery cursor, not event sequence. A delayed event with a
  lower sequence therefore remains visible after the existing cursor watermark.
- `AckDeliveriesThrough { listener_id, delivery_cursor }` removes only this listener's entries at
  or below the acknowledged delivery cursor. It cannot discard an unread late lower-sequence
  delivery.
- Pre-page delivery requests and sequence-based acknowledgement are removed. No compatibility
  request, default unbounded page, or forwarding path is supported.

## Retention And Dispatch Invariants

- Durable replay, frame-local, and latest-state queues retain delivery-cursor indexes. Event
  sequence indexes remain only for journal/replay ordering.
- Listener pages use a cursor-first three-way merge and access at most the requested page plus one
  successor candidate per queue. They do not materialize, sort, or JSON-project every retained
  record under a registry lock.
- Latest-state coalescing owns a key-to-delivery-cursor index; acknowledgement and pruning remove
  all corresponding indexes atomically.
- A `SharedEditorEventRecord` JSON-encodes once at construction for its byte budget. Fanout shares
  an `Arc`; owned delivery DTO and JSON projection happen only at the control-response boundary.
- The registry rebuilds an immutable route snapshot only when listener configuration changes.
  `record` releases the registry guard before filter evaluation and before locking any
  listener-owned inbox.

## Regression Inventory

- `listener_delivery_pages_are_bounded_cursor_ordered_and_continuable`
- `delivery_cursor_does_not_skip_a_late_lower_event_sequence`
- `immutable_routes_keep_one_thousand_listener_inboxes_bounded_and_cursor_addressable`
- `out_of_order_fanout_keeps_the_newest_state_and_delivery_cursor_arrival_order`
- `event_listener_control_queries_delivery_pages_after_a_delivery_cursor`
- `event_listener_control_acknowledges_deliveries_through_delivery_cursor`
- `event_listener_control_rejects_unknown_listener_queries`
- `listener_delivery_filter_and_enqueue_stay_outside_the_registry_guard`

## Current Evidence

- Source and static formatting evidence, scoped diff checks, and independent second review are
  recorded by the canonical Editor02 failure handoff:
  [`failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md`](../../docs/plans/zircon_editor/editor/02/failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md).
- Managed current-source Cargo and the 0/1/1k/10k listener contention evidence remain pending.
  Historical Cargo attempts for the removed protocol are not acceptance evidence for this
  hard-cut contract.

## Acceptance Decision

The listener retention failure remains open. It may move to `fixed` only after coordinator-managed
current-source validation covers the cursor-page control path, retention regressions, and required
listener contention evidence.
