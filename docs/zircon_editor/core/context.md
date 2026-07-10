---
related_code:
  - zircon_editor/src/core/context/mod.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/editor_event/service/editor_event_service.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
  - zircon_editor/src/tests/support.rs
implementation_files:
  - zircon_editor/src/core/context/mod.rs
  - zircon_editor/src/core/context/builder.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/tests/support.rs
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/tests/ui/boundary/editor_event_cutover.rs
  - zircon_editor/src/tests/editor_event/service.rs
  - zircon_editor/src/tests/support.rs::env_lock_tests::shared_test_environment_lock_recovers_after_poison
doc_type: module-detail
---

# Editor Context

`EditorContext` is the L1 aggregate for editor services that are valid without a live workbench UI. It replaces the former UI-coupled event-runtime aggregate with explicit service ownership and independent synchronization.

## Ownership

`EditorContextBuilder` creates one shared `SharedEditorMessageBus`, passes the same handle to `EditorEventService`, and returns an `Arc<EditorContext>`. `EditorManager` constructs and retains that context during editor bootstrap. UI hosts receive the manager-owned context instead of manufacturing a second event core.

The context currently owns:

- the typed shared editor message bus;
- the journal/listener `EditorEventService`.

It intentionally does not own workbench layout, transient UI state, UI controls, extension presentation state, undo/redo operation state, play-mode backend state, or gizmo drag state. Those domains have dedicated owners and locks.

## Synchronization Contract

`EditorContext` contains no aggregate mutex. Each child service synchronizes only its own state. Message request callbacks run outside the raw bus lock, and poisoned context-service locks recover the inner state rather than cascading a process-wide failure.

The split prevents a listener callback, message handler, or reflection refresh from holding one global editor lock while re-entering another editor API.

## UI Boundary

`EditorHostEventController` is the UI-host coordinator. It references the manager-owned `EditorContext` and separately owns:

- `WorkbenchShellState` for `EditorState`, manager linkage, transient UI data, UI controls, and presentation extensions;
- `EditorOperationState` for the operation registry and undo/redo stack;
- `EditorPlayBridge` for the runtime play-mode backend;
- `GizmoDragState` for viewport interaction state.

Core context modules do not import `crate::ui`. The controller performs cross-owner orchestration at the UI host boundary.

## Construction

Use `EditorContextBuilder::new().build()` for the standard context. Tests and alternate hosts can inject an existing `SharedEditorMessageBus` with `with_bus` so all services observe the same transport.

## Validation Status

The hard-cutover boundary test verifies that the deleted aggregate types and owner files do not return, and that context/event/play core modules do not import UI modules. The M1 journal-equivalence test records a known event sequence through `EditorEventService` and checks sequence and revision progression.

The shared editor test environment lock recovers and clears standard-mutex poison. This keeps one panic in a configuration-sensitive test from turning the rest of the editor suite into false `PoisonError` failures, while retaining the existing serialized environment contract at every call site.

Recorded M1-focused evidence is message 9/9, event 85/85, and hard-cut 2/2. A later independent single-thread editor-library run executed 2928 tests as 2754 passed / 140 failed / 34 ignored; a subsequent diagnostic run still saw 133 shared layout/text/plugin failures and resource exhaustion, so neither run accepts M1. The concrete glyph, provider lookup, and ZUI/layout failures are handed to Runtime Text 01, Frameworks 02, and Editor Layout 15 in their numbered `2026-07-11-editor-m1-failure-handoff.md` records. M1 remains open until `cargo test -p zircon_editor --lib --locked` passes.
