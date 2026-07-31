---
related_code:
  - zircon_editor/src/core/editor_message/mod.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/inbox.rs
  - zircon_editor/src/core/editor_message/retention.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/core/editor_message/subscriber.rs
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/core/editor_message/refresh_report.rs
  - zircon_editor/src/core/editor_message/ids/mod.rs
  - zircon_editor/src/core/editor_message/ids/document_id.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editor_message/ids/play_state_kind.rs
  - zircon_editor/src/core/editor_message/ids/scene_mode_id.rs
  - zircon_editor/src/core/editor_message/ids/selection_domain.rs
  - zircon_editor/src/core/editor_message/message/mod.rs
  - zircon_editor/src/core/editor_message/message/delivery.rs
  - zircon_editor/src/core/editor_message/message/dirty_mark.rs
  - zircon_editor/src/core/editor_message/message/document.rs
  - zircon_editor/src/core/editor_message/message/envelope.rs
  - zircon_editor/src/core/editor_message/message/focus.rs
  - zircon_editor/src/core/editor_message/message/mode.rs
  - zircon_editor/src/core/editor_message/message/payload.rs
  - zircon_editor/src/core/editor_message/message/protocol.rs
  - zircon_editor/src/core/editor_message/message/request.rs
  - zircon_editor/src/core/editor_message/message/response.rs
  - zircon_editor/src/core/editor_message/message/transaction.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/editor_event/service/editor_event_service.rs
  - zircon_editor/src/core/jobs/event.rs
  - zircon_editor/src/core/jobs/pump.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
implementation_files:
  - zircon_editor/src/core/editor_message/mod.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/inbox.rs
  - zircon_editor/src/core/editor_message/retention.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/core/editor_message/subscriber.rs
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/core/editor_message/refresh_report.rs
  - zircon_editor/src/core/editor_message/ids/mod.rs
  - zircon_editor/src/core/editor_message/ids/document_id.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editor_message/ids/play_state_kind.rs
  - zircon_editor/src/core/editor_message/ids/scene_mode_id.rs
  - zircon_editor/src/core/editor_message/ids/selection_domain.rs
  - zircon_editor/src/core/editor_message/message/mod.rs
  - zircon_editor/src/core/editor_message/message/delivery.rs
  - zircon_editor/src/core/editor_message/message/dirty_mark.rs
  - zircon_editor/src/core/editor_message/message/document.rs
  - zircon_editor/src/core/editor_message/message/envelope.rs
  - zircon_editor/src/core/editor_message/message/focus.rs
  - zircon_editor/src/core/editor_message/message/mode.rs
  - zircon_editor/src/core/editor_message/message/payload.rs
  - zircon_editor/src/core/editor_message/message/protocol.rs
  - zircon_editor/src/core/editor_message/message/request.rs
  - zircon_editor/src/core/editor_message/message/response.rs
  - zircon_editor/src/core/editor_message/message/transaction.rs
  - zircon_editor/src/core/jobs/event.rs
  - zircon_editor/src/core/jobs/pump.rs
plan_sources:
  - user: 2026-07-10 完整实现 editor 架构并硬切旧架构
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/tests/editor_message/bus/mod.rs
  - zircon_editor/src/tests/editor_message/bus/broadcast.rs
  - zircon_editor/src/tests/editor_message/bus/dirty_set.rs
  - zircon_editor/src/tests/editor_message/bus/protocol_matrix.rs
  - zircon_editor/src/tests/editor_message/bus/publish.rs
  - zircon_editor/src/tests/editor_message/bus/request.rs
  - zircon_editor/src/tests/editor_message/refresh.rs
  - tests/acceptance/editor-architecture-plan-01-m1.md
doc_type: module-detail
---

# Editor Message Bus

## Purpose

`core::editor_message` is the L1 messaging boundary shared by headless editor services and UI consumers. It routes small typed facts to multiple subscribers, records request deliveries, and accumulates view invalidation without making the message bus a second source of authoritative editor state.

Plan 01 M1.1 removes the old `Empty` and `Text` payloads. A message now belongs to the document, transaction, mode, focus, or job family, or carries a schema-labelled JSON custom payload. Heavy document, history, world, selection, and active-job state remains behind its owning query surface.

## Related Files

The module is split by ownership. `bus.rs` owns mutable routing state, `inbox.rs` owns bounded subscriber storage and pressure counters, `retention.rs` owns protocol/payload retention classification, and `shared.rs` owns synchronization and the request callback boundary. `topics.rs` owns built-in topic names, `ids/` owns stable lightweight identifiers, and `message/` owns one transport declaration per file. `mod.rs` files contain only module wiring and curated re-exports.

## Behavior Model

Subscribers register exact `EditorTopic` values. `publish` targets matching subscribers, `broadcast` targets every subscriber, and `request` targets one subscriber. Each publication constructs one immutable delivery payload; subscriber fanout clones only its `Arc` handle, so custom JSON and job strings are not deep-cloned per recipient.

Each inbox has three independent count limits, a 2 MiB default single-delivery logical byte limit, a 16 MiB default total logical retained-payload limit, and one protocol-owned retention policy:

- `Lossless`: transaction events, document open/close/save, play-state edges, job start/terminal events, and every synchronous request. A full lossless lane preserves existing edges and reports backpressure instead of discarding them; a request does not invoke its handler when admission fails.
- `Latest`: document dirty state, focus request/object, scene mode, selection revision per domain, and job progress per job id. Publishing the same semantic key replaces the older queued state while moving the new state to its publication position. Under total-byte pressure the inbox first plans any required eviction of other older Latest entries, then atomically commits the replacement; dispatch reports expose both coalescing and dropping when both occur.
- `Bounded`: schema-labelled custom messages whose semantics are unknown to the core. A full lane evicts its oldest bounded item and increments an explicit drop counter.

`EditorMessageInboxStats` exposes depth by class, drained/coalesced/dropped/backpressured totals, and queue age in publication messages. Production consumers drain deliveries; the old cloning inspection helper is test-only and cannot become a per-frame production polling API.

The inbox stores surviving deliveries in a sequence-keyed `BTreeMap` and maintains lane depth/bytes at enqueue, replacement, eviction, and drain boundaries. `latest_by_key` resolves coalescing without searching the mixed queue; `latest_order` and `bounded_order` restrict pressure scans to their bounded lanes. Drain iterates the sequence map, preserving global surviving publication order. Payload byte cost, including the dynamic dirty-view identifier, is computed once before the delivery is shared and is never reserialized per subscriber. Dirty state is merged only when at least one inbox accepts the publication, so an oversized rejected message cannot retain its view identifier through the dirty set.

Subscriber allocation uses checked identity and returns `SubscriberIdExhausted` before changing subscriptions or inboxes. Delivery sequence exhaustion returns `DeliverySequenceExhausted` in publish/broadcast dispatch reports and synchronous request errors before dirty state or handler execution changes. No ID is saturated or reused.

`SharedEditorMessageBus::request` performs three phases:

1. lock the bus, validate the target, admit the lossless request delivery, then release the lock; a full lossless lane returns `EditorMessageBusError::Backpressured`;
2. invoke `EditorRequestHandler`, allowing the handler to publish or request through the same shared bus;
3. lock again, revalidate the target, and record response invalidation.

If the handler unregisters the target during phase 2, phase 3 returns `EditorMessageBusError::UnknownSubscriber`. Poisoned bus locks recover their owned state instead of panicking the editor process.

## Protocol And Payload Matrix

Every protocol transports every built-in payload family without conversion:

| Protocol | Document | Transaction | Mode | Focus | Job |
|---|---|---|---|---|---|
| Publish | exact-topic fan-out | exact-topic fan-out | exact-topic fan-out | exact-topic fan-out | main-thread pump fan-out |
| Request | one target + response | one target + response | one target + response | one target + response | one target + response |
| Broadcast | all subscribers | all subscribers | all subscribers | all subscribers | all subscribers |

Built-in topic strings are `editor.document`, `editor.transaction`, `editor.mode`, `editor.focus`, and `editor.job`. Job workers never invoke subscribers: they write `JobEvent` values to the jobs channel, and the main-thread `pump_events()` publishes them. The topic parser continues to require at least two non-empty lowercase namespace segments.

## Data And Invalidation Flow

`EditorMessage` may carry an `EditorViewDirtyMark`. The mutable bus merges marks in `ViewDirtySet` by `ViewInstanceId` until the frame boundary drains them. `EditorHostEventController::refresh_view` emits a schema-labelled internal custom message through the manager-owned `EditorContext` bus and still falls back to full reflection materialization. M1.2 already removed the bus from the deleted editor-event aggregate; editor-layout owns eventual partial snapshot publication.

## Edge Cases And Constraints

- Unknown and concurrently removed request targets return a typed error.
- Lossless inbox saturation returns typed backpressure and retains the already queued edge order.
- Latest and bounded pressure is visible through dispatch reports and inbox counters; no pane owns a private retention rule.
- Empty invalidation masks are ignored.
- The raw `EditorMessageBus` is crate-private; cross-service consumers use `SharedEditorMessageBus`.
- `serde_json::Value` makes message envelopes `PartialEq`, not `Eq`.
- No `EditorMessage::empty` or `EditorMessage::text` compatibility constructors remain.

## Test Coverage

The message tests cover exact-topic routing, protocol/payload matrix, broadcast, dirty merging, unknown/backpressured requests, handler re-entry, checked ID exhaustion, lossless order, latest coalescing plus atomic same-key replacement eviction, bounded eviction, mixed/zero/drain lane counters, single/total/dirty-view byte budgets, shared fanout identity, and a paused 100-subscriber/10,000-update storm. The ignored single-thread evidence gate runs 1/5/100 subscribers over a full 4,096 lossless mixed backlog, requires Windows RSS samples, enforces a 50 ms publish-p95 budget, permits only bounded per-inbox metadata allocation, and still rejects payload-size multiplied by fanout. It reports allocation/RSS/queue counters. The static architecture contract is green; managed editor-library and performance gates remain pending and no fixed return is claimed yet.

## Plan Sources

The current design implements Plan 01 M1.1 under the editor-wide layer and ownership rules in Plan 00. It also preserves the incremental dirty-set behavior delivered by editor-layout Plan 09 while applying the engine structure convention's owner-module, thin façade, hard-cutover, and typed-error requirements.

## Open Issues Or Follow-up

The shared bus now belongs to `EditorContext`, and the journal/listener and UI state owners are split. The acceptance decision remains open until the corresponding runtime-text, Frameworks, and editor-layout owners close their recorded failures and the complete editor suite passes.
