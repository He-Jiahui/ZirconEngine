---
title: Editor60 native tab and resize pointer owner
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: programmatic_hide_static_review_passed_managed_validation_pending
performance_status: product_gate_pending
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/entry/sequence/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/drag.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/focus.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/constants.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/drag_resize.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters/callbacks.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/60/2026-09-27-hierarchy-native-input-owner.md
tests:
  - zircon_editor/src/tests/host/retained_window/native_host_contract/native_capture_pointer_owner.rs
  - zircon_editor/src/ui/retained_host/app/tests/hierarchy_native_capture.rs
  - zircon_editor/src/ui/retained_host/app/tests/projection_geometry.rs
  - zircon_editor/src/ui/retained_host/app/tests/close_prompt.rs
---

# Editor60 native tab and resize pointer owner

## Source finding and affected path

The native event loop and its test harness carried `UiPointerId` to the Workbench observer, then dropped it before native button and move dispatch. A tab arm stored only tab and coordinates; a resize arm stored only active state and coordinates. Their move and Primary Up handlers accepted every physical pointer. A foreign pointer could overwrite a pending capture, start or move another pointer's drag, or consume the owner's release. Editor1025's real floating hierarchy test exposed that last failure: a foreign tab capture consumed the hierarchy owner's Up and the fallback canceled a valid Reparent.

The path is `UiHostWindowEventLoop::handle_pointer_button/moved` → native button/move dispatch → tab/resize arm, move, finish → `UiHostContext` state and callbacks. The fix passes the translated physical ID through that path. Button input uses `UiPointerId::default()` for normal translated mouse events with no explicit ID. Native capture gets `None` for untranslated or touch-like moves, so neither can advance a mouse capture. Each tab/resize capture stores its owner ID. A pending native capture prevents another press from replacing or switching it, and move/finish check the owner before state mutation, callbacks, or release consumption. An owner release still follows the existing redraw and callback path; after it ends, another pointer may start and finish its own gesture. Each `UiHostWindow` retains separate capture state.

The hierarchy success regression now requires one real Reparent, changed history, exactly one terminal, preserved foreign tab capture until its own Up, and no duplicate Reparent on a late owner Up. Root/child `UiHostWindow` tests exercise tab and resize independently, foreign press/move/up, cross-kind capture attempts, touch-like and untranslated moves, true-owner completion, and a later independent foreign gesture. The prior default-pointer tab/resize tests remain intact. These are source-level regression additions; managed Rust execution is pending.

The local Unreal reference is `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateUser.cpp` (`FSlateUser::SetPointerCaptor`, lines 251–273), which indexes pointer captors by `PointerIndex`; `SlateApplication.cpp` lines 3575–3582 passes the pointer index when setting capture. Zircon's single native tab/resize capture per host is a narrower local policy, and this reference is not a claim of equivalent platform capture or measured performance.

## Validation and remaining gates

V2 handles the missing terminal delivered by `WindowEvent::Focused(false)`: it retires native tab/resize capture and sends an explicit Cancel callback. Drag Cancel does not dispatch a drop. Resize Cancel releases the shell pointer surface capture, removes the active resize and its transient preferred extent, and invalidates window metrics when that extent existed. It never calls `dispatch_resize_to_group`. A late Up cannot commit a canceled gesture, and a new pointer can arm afterward. Root/child host tests cover both capture kinds, late Up, independent windows, and restart; a Workbench host test checks transient extent rollback, unchanged journal, and restart.

`PointerLeft` remains nonterminal for native tab/resize capture. Mouse leave can precede the owning Up outside the window, which `resolve_drag_drop_route_from_pointer` maps to `DetachToWindow` when no target is under the pointer. A touch leave carries its separate finger identity through runtime translation and must not cancel a mouse capture. Tests route translated mouse and touch leave through the host event handler, then verify capture preservation and owner Up. This cross-window drag policy still depends on actual platform Up delivery.

The CloseRequested successor waits for the close callback's response. Only `HideWindow` retires tab/resize capture through the same noncommitting Cancel path as FocusLost, then marks the host hidden before event-loop exit. `KeepWindowShown` leaves capture intact while a save prompt remains visible. Root/child native host regressions cover cancellation, independent windows, late Up, and restart; Workbench regressions cover clean child close without a tab drop, dirty child prompt preservation, and root drawer resize transient rollback with an unchanged journal. These tests call the production response handler; actual OS event-loop exit is not dynamically exercised.

The Editor1026 programmatic-hide successor handles presenter retirement while `RetainedEditorHost` is already mutably borrowed. `UiHostWindow::hide()` clears only that window's native drag/resize capture through `UiHostContext`, without invoking an app callback. For an active resize owned by a child removed from the runtime floating layout, `recompute_if_dirty` clears that child's native resize capture and cancels the app resize before invalidation and geometry snapshots. This discards the transient extent during the same recompute and emits no `SetDrawerRegionExtent`; root-owned and other-window captures are not cleared. Child callback wiring stores the fixed `MainPageId` once and borrows it for each resize event. Host tests cover direct child hide under an app mutable borrow, late Up, restart, per-window native capture isolation, child layout removal during resize, root/foreign source rejection, same-recompute geometry rollback, and no resize journal record. The existing dirty child prompt test continues to cover `KeepWindowShown` preserving native tab capture.

V1, v2, CloseRequested, and programmatic-hide independent static reviews are complete. The programmatic-hide review receipt is `.codex/state/session-coordinator/async-validation-batches/2026-09-28-editor1026-programmatic-hide-resize-successor-independent-static-review.json` (SHA-256 `eceec8ed4b2f5244712d7566d3b59cfa35fd0f5f616220f4b99e3206410ebced`). Managed Editor `--lib` execution remains pending. Programmatic hide uses no app callback, avoiding reentry into the mutable host borrow. No direct Cargo run, OS pointer capture test, visual acceptance, or measured performance is claimed; ED60-G02/G03/G36/G38–G40 remain open as described below.

V1 and v2 static reviews covered exact preimages, owner paths, test wiring, scoped rustfmt, source hashes, and inverses. The CloseRequested successor has separate preimages and static evidence for review. No Cargo, OS pointer capture test, Editor window pixel test, or measured performance run is claimed. Managed Editor `--lib` behavior tests remain pending. ED60-G02/G03 still require platform capture acknowledgement, loss, platform drag metrics, and terminal delivery; G36 requires native multi-window/multi-document product isolation. G38–G40 require stage/latency metrics, true Editor visual feedback, and reproducible same-workload reference benchmarks. The new ID comparisons are constant-time source logic; event latency and allocations have not been measured, and no threshold is invented.
