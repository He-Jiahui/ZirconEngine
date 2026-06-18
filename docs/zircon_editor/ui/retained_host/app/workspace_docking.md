---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/tab_drag.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app workspace-docking drag/drop and drawer-resize ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Workspace Docking Host Actions

`app/workspace_docking.rs` owns retained host pointer entry points for workspace tab dragging and drawer resizing. It normalizes host pointer event kinds, refreshes the committed pointer layout before handling pointer events, and delegates drag/drop and drawer-resize behavior to child modules.

The root should stay as a narrow event boundary. It should not carry tab drop route resolution, runtime dispatch, detached-window target construction, drawer shell-frame sizing, transient preferred-size writes, or resize dispatch side effects.

## Drag Drop

`app/workspace_docking/drag_drop.rs` owns workspace tab drag/drop behavior. It syncs the current drag target group into `UiHostContext`, builds a fresh `WorkbenchViewModel`, resolves the tab drop route against committed workbench layout frames, handles detached-window fallback targets, dispatches tab-drop effects into runtime, and updates the status line.

Keeping drag/drop in a child module isolates workbench model rebuild, pointer route resolution, detached-window id generation, and runtime `dispatch_tab_drop(...)` side effects from the pointer-event normalization layer.

## Drawer Resize

`app/workspace_docking/drawer_resize.rs` owns drawer resize capture. It resolves resize shell regions from the shell pointer bridge, computes the starting preferred size from current workbench layout frames, updates transient preferred sizes during drag, marks layout dirty, dispatches the final resize to runtime, and keeps presentation invalidation aligned when layout is otherwise clean.

Keeping drawer resize in a child module leaves size capture, shell-frame conversion, transient preference mutation, and resize dispatch policy separate from tab drag/drop routing.

## Boundary Rules

- Keep host drag/resize pointer event kind normalization and committed pointer-layout refresh in `app/workspace_docking.rs`.
- Keep tab drag target group synchronization, drop-route resolution, detached-window fallback target construction, and `dispatch_tab_drop(...)` handling in `app/workspace_docking/drag_drop.rs`.
- Keep drawer resize capture, transient preferred-size updates, shell-frame visibility checks, runtime resize dispatch, and resize-driven invalidation in `app/workspace_docking/drawer_resize.rs`.
- Keep callback registration in `app/callback_wiring.rs`; callback wiring should call the retained-host workspace docking entry methods only.
- Keep pure tab drop route selection helpers in `ui/retained_host/tab_drag.rs`; app docking modules should consume those helpers instead of duplicating route math.

## Validation Notes

The 2026-06-18 drag/drop and drawer-resize split reduced `workspace_docking.rs` from 256 lines to 61 lines. `workspace_docking/drag_drop.rs` is 103 lines and owns tab drop target synchronization, drop-route resolution, detached-window fallback ids, and runtime tab-drop dispatch. `workspace_docking/drawer_resize.rs` is 101 lines and owns drawer resize capture, transient preferred-size writes, resize dispatch, and resize invalidation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app workspace-docking drag/drop and drawer-resize ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
