---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/chrome.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/drag_resize.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/menu.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/runtime.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/content.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/controls.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/details.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/mesh_import.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/references.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/tree.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/component_showcase.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/console.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/hierarchy.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/inspector.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/pane_controls.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/ui_asset.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/welcome.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/chrome.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/drag_resize.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/menu.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/runtime.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/content.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/controls.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/details.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/mesh_import.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/references.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/assets/tree.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/component_showcase.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/console.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/hierarchy.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/inspector.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/pane_controls.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/ui_asset.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/welcome.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app callback-wiring pane-surface ownership scan
  - app callback-wiring viewport/UI Asset callback ownership scan
  - app callback-wiring Component Showcase callback ownership scan
  - app callback-wiring host-shell callback ownership scan
  - app callback-wiring host-shell subowner ownership scan
  - app callback-wiring pane-surface region callback ownership scan
  - app callback-wiring asset callback ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Callback Wiring

`app/callback_wiring.rs` owns the top-level retained host callback installation entry. It keeps `dispatch_with_callback_source(...)`, the shared helper that preserves the source host window while dispatching callbacks into `RetainedEditorHost`, and delegates callback registration to host-shell and pane-surface children.

## Host Shell Callbacks

`app/callback_wiring/host_shell.rs` is the structural `UiHostContext` callback entry. It resolves the global once and delegates callback groups to runtime, menu, chrome, and drag/resize child owners.

`host_shell/runtime.rs` owns frame ticks, unhandled keyboard input, and close prompt actions. `host_shell/menu.rs` owns menu pointer events and activity rail clicks. `host_shell/chrome.rs` owns host page, document tab, document tab close, and drawer header pointer callbacks. `host_shell/drag_resize.rs` owns floating-window header clicks, host drag capture, and resize capture.

Host-shell callbacks are already tied to the native host window, so they do not need `dispatch_with_callback_source(...)`. Pane-surface callbacks continue to use the shared helper when callback source-window attribution matters.

## Pane Surface Callbacks

`app/callback_wiring/pane_surface.rs` owns the structural `PaneSurfaceHostContext` callback registration entry. It resolves the global once, then delegates Welcome, Hierarchy, Console, Inspector, pane surface controls, asset, Component Showcase, viewport, and UI Asset editor callback groups to child modules.

The child module uses the parent `dispatch_with_callback_source(...)` helper for callbacks that need source-window attribution. Direct host-shell callbacks that are already scoped to the root window stay in `callback_wiring/host_shell.rs`.

`app/callback_wiring/pane_surface/welcome.rs` owns Welcome recent-project pointer callbacks and Welcome surface control callbacks. `hierarchy.rs` owns Hierarchy pointer click/move/scroll/full-event callbacks. `console.rs` owns Console scroll callback registration. `inspector.rs` owns Inspector scroll, reference pointer, and control callbacks. `pane_controls.rs` owns generic pane surface control click/edit callbacks plus Workbench context-menu requests.

## Asset Callbacks

`app/callback_wiring/pane_surface/assets.rs` owns the structural asset callback registration entry. It delegates mesh import path editing, asset control changes/clicks, asset tree pointer events, asset content pointer events, asset reference list pointer events, and browser asset details scrolling to asset child modules.

`app/callback_wiring/pane_surface/assets/mesh_import.rs` owns mesh import path edit callback registration. `controls.rs` owns asset control change/click callbacks. `tree.rs` owns asset tree pointer callbacks. `content.rs` owns asset content pointer callbacks. `references.rs` owns asset reference and used-by pointer callbacks. `details.rs` owns browser asset detail scroll callback registration.

## Component Showcase Callbacks

`app/callback_wiring/pane_surface/component_showcase.rs` owns Component Showcase pane-surface callback registration. It wires activate, drag-delta, edit, context request, and option selection callbacks into the retained host Component Showcase action owner while preserving callback source-window attribution.

## Viewport Callbacks

`app/callback_wiring/pane_surface/viewport.rs` owns viewport pane-surface callback registration. It wires viewport pointer events and viewport toolbar pointer clicks into the retained host viewport action owners while preserving callback source-window attribution.

## UI Asset Callbacks

`app/callback_wiring/pane_surface/ui_asset.rs` owns UI Asset editor pane-surface callback registration. It wires UI Asset top-level actions, detail events, and collection events into the retained host UI Asset editor action owners while preserving callback source-window attribution.

## Boundary Rules

- Keep `app/callback_wiring.rs` as the structural callback installation entry and owner of `dispatch_with_callback_source(...)`.
- Keep `app/callback_wiring/host_shell.rs` as the structural `UiHostContext` callback entry.
- Keep frame tick, keyboard, and close-prompt callback registration in `app/callback_wiring/host_shell/runtime.rs`.
- Keep menu pointer and activity rail callback registration in `app/callback_wiring/host_shell/menu.rs`.
- Keep host page, document tab, document-tab close, and drawer header callback registration in `app/callback_wiring/host_shell/chrome.rs`.
- Keep floating-window header, host drag, and host resize callback registration in `app/callback_wiring/host_shell/drag_resize.rs`.
- Keep `app/callback_wiring/pane_surface.rs` as the structural `PaneSurfaceHostContext` callback entry.
- Keep Welcome callback registration in `app/callback_wiring/pane_surface/welcome.rs`.
- Keep Hierarchy callback registration in `app/callback_wiring/pane_surface/hierarchy.rs`.
- Keep Console callback registration in `app/callback_wiring/pane_surface/console.rs`.
- Keep Inspector callback registration in `app/callback_wiring/pane_surface/inspector.rs`.
- Keep generic pane surface control and Workbench context-menu callback registration in `app/callback_wiring/pane_surface/pane_controls.rs`.
- Keep `app/callback_wiring/pane_surface/assets.rs` as the structural asset callback entry.
- Keep mesh import callback registration in `app/callback_wiring/pane_surface/assets/mesh_import.rs`.
- Keep asset control callback registration in `app/callback_wiring/pane_surface/assets/controls.rs`.
- Keep asset tree callback registration in `app/callback_wiring/pane_surface/assets/tree.rs`.
- Keep asset content callback registration in `app/callback_wiring/pane_surface/assets/content.rs`.
- Keep asset reference callback registration in `app/callback_wiring/pane_surface/assets/references.rs`.
- Keep browser asset detail callback registration in `app/callback_wiring/pane_surface/assets/details.rs`.
- Keep Component Showcase callback registration in `app/callback_wiring/pane_surface/component_showcase.rs`.
- Keep viewport pointer and toolbar callback registration in `app/callback_wiring/pane_surface/viewport.rs`.
- Keep UI Asset editor action/detail/collection callback registration in `app/callback_wiring/pane_surface/ui_asset.rs`.
- Keep actual pane/action execution in feature modules such as `pane_surface_actions`, asset handlers, viewport handlers, and UI Asset editor handlers; callback wiring should only adapt callback arguments and forward into those owners.
- Keep source-window attribution explicit for callbacks coming from pane surfaces.

## Validation Notes

The 2026-06-18 pane-surface callback split reduced `callback_wiring.rs` from 669 lines to 153 lines. `callback_wiring/pane_surface.rs` is 527 lines and owns the pane-surface callback registration set.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring pane-surface ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 asset callback split reduced `callback_wiring/pane_surface.rs` from 527 lines to 324 lines. `callback_wiring/pane_surface/assets.rs` is 216 lines and owns the mesh import, asset control, asset tree/content/reference, and browser asset details callback registration set. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring asset callback ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 viewport/UI Asset callback split reduced `callback_wiring/pane_surface.rs` from 324 lines to 245 lines. `callback_wiring/pane_surface/viewport.rs` is 43 lines and owns viewport pointer/toolbar callback registration; `callback_wiring/pane_surface/ui_asset.rs` is 59 lines and owns UI Asset action/detail/collection callback registration. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring viewport/UI Asset callback ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-18 Component Showcase callback split reduced `callback_wiring/pane_surface.rs` from 245 lines to 177 lines. `callback_wiring/pane_surface/component_showcase.rs` is 79 lines and owns Component Showcase activation, drag, edit, context, and option callback registration. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring Component Showcase callback ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only.

The 2026-06-19 host-shell callback split reduced `callback_wiring.rs` from 136 lines to 18 lines. `callback_wiring/host_shell.rs` is 126 lines and owns `UiHostContext` callback registration for frame ticks, keyboard input, close prompt actions, menu pointers, Workbench chrome pointers, drag capture, and resize capture. The root keeps `dispatch_with_callback_source(...)` and delegates to host-shell and pane-surface callback groups.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring host-shell callback ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 pane-surface region callback split reduced `callback_wiring/pane_surface.rs` from 177 lines to 30 lines. New child modules own the formerly inline region groups: `welcome.rs` is 61 lines, `hierarchy.rs` is 40 lines, `console.rs` is 16 lines, `inspector.rs` is 45 lines, and `pane_controls.rs` is 41 lines. Existing asset, Component Showcase, viewport, and UI Asset callback children keep their prior ownership.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring pane-surface region callback ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` was attempted but timed out while concurrent `zircon_runtime` cargo jobs were active in separate target directories, so this slice does not claim a fresh full compile pass. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 asset callback child split reduced `callback_wiring/pane_surface/assets.rs` from 216 lines to 22 lines. New child modules own the formerly inline asset callback groups: `content.rs` is 64 lines, `controls.rs` is 33 lines, `details.rs` is 18 lines, `mesh_import.rs` is 18 lines, `references.rs` is 87 lines, and `tree.rs` is 39 lines.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring asset callback ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. A fresh full `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` was not claimed for this slice because concurrent `zircon_runtime` Cargo jobs were still active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 host-shell subowner split reduced `callback_wiring/host_shell.rs` from 126 lines to a 15-line structural entry. New child owners are `host_shell/runtime.rs` (29 lines) for frame/keyboard/close-prompt callbacks, `host_shell/menu.rs` (35 lines) for menu and activity rail callbacks, `host_shell/chrome.rs` (64 lines) for page/tab/drawer header callbacks, and `host_shell/drag_resize.rs` (28 lines) for floating header, drag, and resize callbacks.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app callback-wiring host-shell subowner ownership scan, and scoped `git diff --check`. A fresh `cargo check` was not rerun for this slice because the current focused editor check is blocked before editor code by `zircon_runtime` duplicate method definitions in `scene/dynamic_scene/session/path_capture.rs`; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The follow-up 2026-06-19 owner-split batch compile validation reran `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
