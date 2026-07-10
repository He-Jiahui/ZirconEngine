---
related_code:
  - zircon_editor/src/core/editor_message/mod.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/core/editor_message/subscriber.rs
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/core/editor_message/refresh_report.rs
  - zircon_editor/src/core/editor_message/ids/mod.rs
  - zircon_editor/src/core/editor_message/ids/document_id.rs
  - zircon_editor/src/core/editor_message/ids/history_context_id.rs
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
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
implementation_files:
  - zircon_editor/src/core/editor_message/mod.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/editor_message/topics.rs
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/core/editor_message/subscriber.rs
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/core/editor_message/refresh_report.rs
  - zircon_editor/src/core/editor_message/ids/mod.rs
  - zircon_editor/src/core/editor_message/ids/document_id.rs
  - zircon_editor/src/core/editor_message/ids/history_context_id.rs
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
plan_sources:
  - user: 2026-07-10 完整实现 editor 架构并硬切旧架构
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
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

Plan 01 M1.1 removes the old `Empty` and `Text` payloads. A message now belongs to the document, transaction, mode, or focus family, or carries a schema-labelled JSON custom payload. Heavy document, history, world, and selection state remains behind its owning query surface.

## Related Files

The module is split by ownership. `bus.rs` owns mutable routing state, `shared.rs` owns synchronization and the request callback boundary, `topics.rs` owns built-in topic names, `ids/` owns stable lightweight identifiers, and `message/` owns one transport declaration per file. `mod.rs` files contain only module wiring and curated re-exports.

## Behavior Model

Subscribers register exact `EditorTopic` values. `publish` targets matching subscribers, `broadcast` targets every subscriber, and `request` targets one subscriber. Each delivery preserves the protocol, topic, and typed payload.

`SharedEditorMessageBus::request` performs three phases:

1. lock the bus, validate the target, record the request delivery, then release the lock;
2. invoke `EditorRequestHandler`, allowing the handler to publish or request through the same shared bus;
3. lock again, revalidate the target, and record response invalidation.

If the handler unregisters the target during phase 2, phase 3 returns `EditorMessageBusError::UnknownSubscriber`. Poisoned bus locks recover their owned state instead of panicking the editor process.

## Protocol And Payload Matrix

Every protocol transports every built-in payload family without conversion:

| Protocol | Document | Transaction | Mode | Focus |
|---|---|---|---|---|
| Publish | exact-topic fan-out | exact-topic fan-out | exact-topic fan-out | exact-topic fan-out |
| Request | one target + response | one target + response | one target + response | one target + response |
| Broadcast | all subscribers | all subscribers | all subscribers | all subscribers |

Built-in topic strings are `editor.document`, `editor.transaction`, `editor.mode`, and `editor.focus`. The topic parser continues to require at least two non-empty lowercase namespace segments.

## Data And Invalidation Flow

`EditorMessage` may carry an `EditorViewDirtyMark`. The mutable bus merges marks in `ViewDirtySet` by `ViewInstanceId` until the frame boundary drains them. `EditorHostEventController::refresh_view` emits a schema-labelled internal custom message through the manager-owned `EditorContext` bus and still falls back to full reflection materialization. M1.2 already removed the bus from the deleted editor-event aggregate; editor-layout owns eventual partial snapshot publication.

## Edge Cases And Constraints

- Unknown and concurrently removed request targets return a typed error.
- Empty invalidation masks are ignored.
- The raw `EditorMessageBus` is crate-private; cross-service consumers use `SharedEditorMessageBus`.
- `serde_json::Value` makes message envelopes `PartialEq`, not `Eq`.
- No `EditorMessage::empty` or `EditorMessage::text` compatibility constructors remain.

## Test Coverage

The Plan 01 M1.1 test code covers four-family exact-topic routing, the 3×4 protocol/payload matrix, broadcast delivery, dirty-mask merging, unknown request targets, handler re-entry through the same shared bus, and target removal during the unlocked callback window. The latest recorded focused evidence is 9/9 for editor messages, while the declared complete editor-library gate remains open. Runtime Text 01, Frameworks 02, and Editor Layout 15 now own the concrete glyph, provider lookup, and ZUI/layout failure handoffs under their numbered `2026-07-11-editor-m1-failure-handoff.md` archives.

## Plan Sources

The current design implements Plan 01 M1.1 under the editor-wide layer and ownership rules in Plan 00. It also preserves the incremental dirty-set behavior delivered by editor-layout Plan 09 while applying the engine structure convention's owner-module, thin façade, hard-cutover, and typed-error requirements.

## Open Issues Or Follow-up

The shared bus now belongs to `EditorContext`, and the journal/listener and UI state owners are split. The acceptance decision remains open until the corresponding runtime-text, Frameworks, and editor-layout owners close their recorded failures and the complete editor suite passes.
