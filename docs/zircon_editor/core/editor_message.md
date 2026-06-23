---
related_code:
  - zircon_editor/src/core/editor_message/mod.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/message.rs
  - zircon_editor/src/core/editor_message/refresh_report.rs
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/core/editor_message/subscriber.rs
  - zircon_editor/src/core/editor_event/runtime/editor_event_runtime_state.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
implementation_files:
  - zircon_editor/src/core/editor_message/mod.rs
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/message.rs
  - zircon_editor/src/core/editor_message/refresh_report.rs
  - zircon_editor/src/core/editor_message/topic.rs
  - zircon_editor/src/core/editor_message/view_dirty_set.rs
  - zircon_editor/src/core/editor_message/subscriber.rs
tests:
  - zircon_editor/src/tests/editor_message/bus.rs
  - zircon_editor/src/tests/editor_message/refresh.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
doc_type: module-detail
---

# Editor Message Bus

## Purpose

`core::editor_message` provides the editor-layout message substrate needed before window tabs, detachable drawers, and plugin pages can communicate without forcing global refresh. It owns topic validation, subscriber registration, pub-sub dispatch, request-response dispatch, broadcast dispatch, and a view-level dirty set.

## Behavior Model

Subscribers register a set of `EditorTopic` values. `publish(...)` delivers only to subscribers registered for the exact topic. `broadcast(...)` delivers to every registered subscriber. `request(...)` validates the target subscriber and invokes an `EditorRequestHandler`, while also recording the request delivery in the target inbox.

`EditorMessage` may carry an `EditorViewDirtyMark`. The bus records dirty marks into `ViewDirtySet`, which merges masks per `ViewInstanceId` until `drain_dirty()` is called at the frame boundary. This gives retained-host work a single place to consume view-level invalidation.

`EditorEventRuntime::refresh_view(...)` is the first incremental refresh entry. It marks a single `ViewInstanceId` dirty, drains the pending dirty set, and returns `EditorViewRefreshReport`. The current materialization backend still publishes a full reflection snapshot because `EditorUiControlService` does not yet expose a partial tree/diff publish API.

## Design And Rationale

The module is folder-backed so the core root stays thin. `bus.rs` owns routing and inboxes, `topic.rs` owns structured topic validation, `message.rs` owns transport DTOs, `subscriber.rs` owns subscriber identity, and `view_dirty_set.rs` owns the incremental invalidation data structure.

The core module intentionally does not depend on retained-host private `HostInvalidationMask`. Instead, `EditorViewInvalidationMask` mirrors the same bit semantics at the core boundary. The 09.S2 refresh bridge will convert this core mask into retained-host invalidation work.

09.S2 moved normal event/state mutation paths off direct `refresh_reflection_locked(...)` calls. They now go through `refresh_workbench_for_effects_locked(...)` or `refresh_workbench_locked(...)`, which first records view-level dirty state and then uses the current full snapshot backend as a materialize fallback.

## Edge Cases And Constraints

Unknown request targets return `EditorMessageBusError::UnknownSubscriber`; no request is silently dropped. Empty masks are ignored when marking a view dirty. Topic strings must have at least two non-empty dot-separated segments, with lowercase ASCII, digits, underscore, or hyphen in each segment.

## Test Coverage

`zircon_editor/src/tests/editor_message/bus.rs` covers exact-topic pub-sub delivery, per-view mask merging, request-response target validation, and broadcast delivery. `zircon_editor/src/tests/editor_message/refresh.rs` covers `refresh_view(...)` marking and draining the requested view/mask while reporting that the current backend used full snapshot materialization. On 2026-06-23, `cargo test -p zircon_editor --lib editor_message --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` passed 5/5 focused tests.

## Plan Sources

This document records `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md` S1 and the focused 09.S2 incremental trigger path.

## Open Issues Or Follow-up

The next slice needs a true partial reflection publish path in `EditorUiControlService` / `UiEventManager`. Until that exists, `refresh_view(...)` provides correct view-level dirty collection but still materializes through the full snapshot backend.
