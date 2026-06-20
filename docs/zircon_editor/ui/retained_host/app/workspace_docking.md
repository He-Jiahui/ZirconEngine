---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop/route.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/capture.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/tab_drag.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop/route.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/capture.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app workspace-docking drag/drop and drawer-resize ownership scan
  - app workspace-docking drag-drop route ownership scan
  - app workspace-docking drawer-resize capture/movement ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Workspace Docking Host Actions

`app/workspace_docking.rs` owns retained host pointer entry points for workspace tab dragging and drawer resizing. It normalizes host pointer event kinds, refreshes the committed pointer layout before handling pointer events, and delegates drag/drop and drawer-resize behavior to child modules.

The root should stay as a narrow event boundary. It should not carry tab drop route resolution, runtime dispatch, detached-window target construction, drawer shell-frame sizing, transient preferred-size writes, or resize dispatch side effects.

## Drag Drop

`app/workspace_docking/drag_drop.rs` owns workspace tab drag/drop behavior. It syncs the current drag target group into `UiHostContext`, asks the route child to resolve tab-drop targets, dispatches tab-drop effects into runtime, and updates the status line.

`app/workspace_docking/drag_drop/route.rs` owns drop route resolution. It builds a fresh `WorkbenchViewModel`, reads shell pointer routes, resolves routes against committed workbench layout frames, and constructs detached-window fallback targets.

Keeping drag/drop in child modules isolates target-group sync and runtime `dispatch_tab_drop(...)` side effects from route resolution, workbench model rebuild, pointer route resolution, detached-window id generation, and pointer-event normalization.

## Drawer Resize

`app/workspace_docking/drawer_resize.rs` is the structural drawer resize entry. `drawer_resize/capture.rs` owns resize-region capture from shell pointer routes and starting preferred-size resolution from current workbench layout frames. `drawer_resize/movement.rs` owns transient preferred-size updates during drag, layout dirty marking, final runtime resize dispatch, and presentation invalidation when layout is otherwise clean.

Keeping drawer resize in child modules leaves size capture, shell-frame conversion, transient preference mutation, and resize dispatch policy separate from tab drag/drop routing.

## Boundary Rules

- Keep host drag/resize pointer event kind normalization and committed pointer-layout refresh in `app/workspace_docking.rs`.
- Keep tab drag target group synchronization and `dispatch_tab_drop(...)` handling in `app/workspace_docking/drag_drop.rs`.
- Keep tab drop-route resolution, workbench model rebuild, pointer route lookup, and detached-window fallback target construction in `app/workspace_docking/drag_drop/route.rs`.
- Keep drawer resize module declarations in `app/workspace_docking/drawer_resize.rs`.
- Keep drawer resize capture, shell-frame visibility checks, shell-frame conversion, and starting preferred-size resolution in `app/workspace_docking/drawer_resize/capture.rs`.
- Keep drawer resize transient preferred-size updates, runtime resize dispatch, and resize-driven invalidation in `app/workspace_docking/drawer_resize/movement.rs`.
- Keep callback registration in `app/callback_wiring.rs`; callback wiring should call the retained-host workspace docking entry methods only.
- Keep pure tab drop route selection helpers in `ui/retained_host/tab_drag.rs`; app docking modules should consume those helpers instead of duplicating route math.

## Validation Notes

The 2026-06-18 drag/drop and drawer-resize split reduced `workspace_docking.rs` from 256 lines to 61 lines. `workspace_docking/drag_drop.rs` is 103 lines and owns tab drop target synchronization, drop-route resolution, detached-window fallback ids, and runtime tab-drop dispatch. `workspace_docking/drawer_resize.rs` is 101 lines and owns drawer resize capture, transient preferred-size writes, resize dispatch, and resize invalidation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app workspace-docking drag/drop and drawer-resize ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 drag-drop route subowner split reduced `workspace_docking/drag_drop.rs` from 103 lines to 50 lines. `workspace_docking/drag_drop/route.rs` is 72 lines and owns workbench model rebuild, pointer route lookup, route resolution against committed layout frames, detached-window fallback route construction, and detached window id sanitization.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app workspace-docking drag-drop route ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 drawer-resize capture/movement subowner split reduced `workspace_docking/drawer_resize.rs` from 101 lines to a 2-line structural entry. `drawer_resize/capture.rs` is 55 lines and owns resize-region capture, shell-frame visibility checks, shell-frame conversion, and starting preferred-size resolution. `drawer_resize/movement.rs` is 61 lines and owns transient preferred-size updates, final resize dispatch, dispatch-effect application, resize-driven invalidation, and pointer layout refresh.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app workspace-docking drawer-resize capture/movement ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
