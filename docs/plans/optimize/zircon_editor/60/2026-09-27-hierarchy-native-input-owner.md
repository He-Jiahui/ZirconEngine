---
title: Editor60 hierarchy native input owner
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: product_gate_pending
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks/host.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/runtime.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/input_owner.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-27-hierarchy-drag-document-identity.md
tests:
  - zircon_editor/src/ui/retained_host/app/tests/hierarchy_native_capture.rs
---

# Editor60 hierarchy native input owner

## Source finding and call chain

`UiHostWindowEventLoop::handle_pointer_button` translates each native event and calls the Workbench pointer observer before native pane dispatch. The original `handle_pointer_moved` called its observer only after native pane routing, where `hierarchy_pointer_moved` could cross the 4px threshold without knowing the moving pointer ID. Native pane routing sends a hierarchy callback only for points in the hierarchy pane; a Primary Up in another `UiHostWindow`, or outside the pane in the source window, can therefore leave the hierarchy bridge armed. A later Up routed inside the source hierarchy pane can submit a stale Reparent. Escape, source-window focus loss, and close also need to retire the same logical gesture.

The native input metadata now uses the root `UiWindowId("editor.main")` or the child presentation's floating-window ID. The editor callback authority remains `MainPageId`; the two types remain separate. The host-shell callback captures a weak handle to its actual source `UiHostWindow` and resolves its live child presentation ID when observing raw input. A strong handle here would create a reference cycle because callbacks live inside the host state. The new `HierarchyInputOwner` records source window, physical pointer ID, Primary button, and a checked local gesture generation. A raw Move first passes a narrow hierarchy-owner observer, then native pane dispatch, then the existing tooltip observer with its computed target. Foreign pointer Move still updates hover but cannot advance the owner's threshold. Touch-like and untranslated native Move also reach pane routing, so the observer marks them ineligible before dispatch; neither can advance a mouse owner's threshold. The observer uses the existing native pane route to detect outside movement/release. A same-pointer event in another window, outside movement/release, pointer Cancel, or Secondary Down retires the owner before a later hierarchy callback. Escape, source-window focus loss, and close use that same terminal function. The callback Down/Up paths reject a different pointer; an owner Up can submit once. After native Primary Up dispatch, a narrow fallback retires an owner Up that a competing native chrome capture consumed before the hierarchy pane callback. World, document, history, and callback-window authority from Editor1022 remains required at commit.

`UiHostWindow::dispatch_native_*_for_test` now follows the production observer and native routing order. This test support change is needed because directly invoking a hierarchy callback would skip the missing observer boundary.

## Real-host regression matrix

| Test | Required observation |
| --- | --- |
| `child_hierarchy_owner_release_in_root_cancels_once_before_late_child_up` | Child A Down/Move, root B Up, then late A Up: unchanged parents/history, no Reparent event, one terminal. |
| `child_hierarchy_outside_release_cancels_before_late_inside_up` | Same-window Up outside its hierarchy pane cancels before a later inside Up. |
| `child_hierarchy_outside_move_cancels_before_late_inside_up` | Leaving the hierarchy route cancels before a late inside Up. |
| `native_pointer_cancel_terminalizes_child_hierarchy_once` | Cancel and late Up produce one cleanup. |
| `different_pointer_in_root_does_not_steal_child_hierarchy_owner` | Another pointer's root Up leaves the child gesture active; owner Up commits once. |
| `different_pointer_down_and_up_inside_child_does_not_steal_hierarchy_owner` | Another pointer's Down/Up inside the same pane cannot supersede or submit; owner Up commits once. |
| `different_pointer_move_inside_child_cannot_activate_owner_reparent` | Owner Down without moving, foreign Move beyond 4px, owner Up: no Reparent or World/history change. |
| `touch_like_native_move_cannot_activate_mouse_hierarchy_owner` and `untranslated_native_move_cannot_activate_mouse_hierarchy_owner` | Touch-like or untranslated native Move still routes to the pane but cannot advance a mouse owner's threshold. |
| `foreign_tab_capture_cannot_consume_hierarchy_owner_release` | Another pointer's real floating tab press arms native tab capture; the owner's Up is consumed by that capture but still ends the hierarchy gesture once, leaving World/history unchanged. |
| `unrelated_root_focus_loss_does_not_cancel_child_hierarchy_owner` | Focus loss in another window leaves the child owner intact. |
| `escape_secondary_press_and_focus_loss_each_cancel_one_native_hierarchy_gesture` | Each terminal path clears payload/identity without changing World/history; late Up is inert. |
| `child_window_close_request_cancels_native_hierarchy_gesture_once` and `main_window_close_request_cancels_child_hierarchy_gesture_once` | Both close boundaries cancel before a late Up. |
| `ordinary_hierarchy_refresh_keeps_child_window_press_authority` | Callback-free refresh does not treat the main window as a competing callback source. |
| `native_hierarchy_source_does_not_keep_its_host_window_alive` | The callback source handle resolves a live child presentation but cannot keep its host state alive after the window is dropped. |

These tests use `ChildWindowHostHarness` to create two real `UiHostWindow` instances, each with its own native host dispatch and callback wiring, plus the real World/history/journal. They do not create two operating-system windows. Managed Windows execution and native platform acceptance are pending; source-only review and static checks are recorded under the T Editor1025 artifact prefix.

The weak source-handle test proves only that this new callback handle does not itself retain the host state. Existing pane-surface hierarchy callbacks still capture strong `UiHostWindow` handles, so full wired-window destruction is not claimed by this slice.

## Remaining acceptance

This slice provides host-level logical ownership and exactly-once terminal behavior for the observed events. Native tab/resize capture still lacks physical pointer identity: a different pointer can arm tab capture, consume the hierarchy owner's Up, and cause this slice's fallback to cancel an otherwise valid Reparent. The real floating-tab regression proves cleanup and unchanged World/history for that case, not independent success of both simultaneous gestures. Pointer-aware native capture needs a separate lower-layer owner across button/move dispatch, tab/resize state, and release handling. ED60-G02/G03 still require operating-system capture acknowledgement/loss delivery and platform drag metrics. G36 native multi-window isolation remains a product gate. G30 bounded large Reparent, G34 100K and 1% churn, G35 1M hierarchy, and G38–G40 latency/native feedback/cross-engine benchmarks remain pending. No fixed millisecond threshold or performance result is inferred.
