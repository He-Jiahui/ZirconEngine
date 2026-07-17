---
related_code:
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editor_event/dispatcher.rs
  - zircon_editor/src/core/editor_event/service/mod.rs
  - zircon_editor/src/core/editor_event/service/editor_event_service.rs
  - zircon_editor/src/core/editor_event/service/listener_control.rs
  - zircon_editor/src/core/editor_event/service/state.rs
  - zircon_editor/src/core/editor_event/service/stamp.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/play/bridge.rs
  - zircon_editor/src/scene/viewport/interaction/gizmo_drag_state.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
implementation_files:
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editor_event/dispatcher.rs
  - zircon_editor/src/core/editor_event/service/mod.rs
  - zircon_editor/src/core/editor_event/service/editor_event_service.rs
  - zircon_editor/src/core/editor_event/service/listener_control.rs
  - zircon_editor/src/core/editor_event/service/state.rs
  - zircon_editor/src/core/editor_event/service/stamp.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/play/bridge.rs
  - zircon_editor/src/scene/viewport/interaction/gizmo_drag_state.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/core/editor_event/listener.rs::tests::listener_acceptance_does_not_normalize_prefixes_per_record
  - zircon_editor/src/core/editor_event/listener.rs::tests::listener_filter_normalizes_operation_prefixes_once
  - zircon_editor/src/tests/editor_event/service.rs
  - zircon_editor/src/tests/editor_event/runtime/
  - zircon_editor/src/tests/editor_event/animation_runtime/
  - zircon_editor/src/tests/ui/boundary/editor_event_cutover.rs
  - tests/acceptance/editor-architecture-plan-01-m1.md
doc_type: module-detail
---

# Editor Event Service And Host Dispatch

The event architecture separates durable event bookkeeping from UI execution. `EditorEventService` is the headless journal/listener owner. `EditorHostEventController` coordinates workbench execution across narrowly scoped state owners.

## Event Service

`EditorEventService` owns only:

- event id and delivery sequence allocation;
- editor revision progression;
- the event journal;
- named listener registration, filtering, delivery, acknowledgement, and status;
- the shared typed editor message bus handle.

`begin_event` advances the revision. `begin_observation` allocates an ordered record without advancing it. `record` appends the immutable record and notifies listeners while holding only the event-service lock. Lock poisoning recovers the owned state.

## Host Dispatch

The UI dispatch path allocates an event stamp from the context service, executes the event against the appropriate owner, records operation history through `EditorOperationState`, refreshes reflection after releasing state locks, and finally records the event through `EditorEventService`.

There is no public method exposing the former aggregate state. Event leaf functions accept the concrete `WorkbenchShellStateData` they need. Play and gizmo events receive the host controller only where cross-owner access is required.

## Hard Cutover

The old aggregate implementation and compatibility surface were deleted:

- `core/editor_event/runtime.rs` and `core/editor_event/runtime/`;
- the former runtime-state, dispatcher, play-backend, bootstrap, and listener-control owner files;
- the aggregate state accessor and every production `lock_inner` call;
- the `EditorEventRuntime` type name.

No alias, re-export, compatibility module, or forwarding wrapper preserves those paths. The play backend moved to `core/play/bridge.rs`; operation state, workbench state, and gizmo state moved to their real owners.

## Test Structure

The former 3590-line event test and 1169-line animation test were replaced by folder-backed, responsibility-based modules. Every resulting test file is below the repository's 800-line soft limit. The existing test-only harness name remains local to tests; no production alias or compatibility surface preserves the removed aggregate type.

`service.rs` asserts the journal sequence/revision contract. The boundary test asserts old symbol and owner-file absence. Existing operation, listener, extension, replay, UI binding, animation, and play-mode cases remain attached to their original test functions after the mechanical module split.

## Validation Status

Recorded focused evidence is 85/85 for the split event suite and 2/2 for the hard-cut guards; the stale play-backend test imports discovered by the complete build target `core::play`. The complete editor-library gate remains non-green across shared Runtime Text, Frameworks, and Editor Layout owners. Their concrete failures now live in the corresponding numbered `2026-07-11-editor-m1-failure-handoff.md` plan records, and Plan 01 M1 remains open until the declared full command passes.

## Performance review status

Listener operation-path prefixes are normalized once when a descriptor/filter is created or
updated. Per-record matching now performs only borrowed prefix and dot-boundary checks; its source
guards completed RED-to-GREEN.

The journal and listener delivery stores remain unbounded, payloads are cloned into journal and
per-listener deliveries, and `record` performs fanout while holding the service state lock. Editor02
owns the linked retention-class/shared-payload/lock-free-dispatch failure. Storm tests, memory
budgets, and current-source Cargo remain pending, so the event module stays outside performance
acceptance.
