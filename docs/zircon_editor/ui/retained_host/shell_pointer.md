---
related_code:
  - zircon_editor/src/ui/retained_host/shell_pointer.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/common.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/drag_frames.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/effects.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/node_ids.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/resize_surface.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/route.rs
  - zircon_editor/src/ui/retained_host/route_intent/map.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drag_drop/route.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/capture.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/shell_pointer.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/resize_surface.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/route.rs
  - zircon_editor/src/ui/retained_host/route_intent/map.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - rustfmt --edition 2021 --check zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs zircon_editor/src/ui/retained_host/shell_pointer/drag_surface.rs zircon_editor/src/ui/retained_host/shell_pointer/resize_surface.rs zircon_editor/src/ui/retained_host/shell_pointer/route.rs
  - cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s1-check-0623 --message-format short --color never
  - cargo test -p zircon_editor --lib shell_pointer --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-m5s1-shell-pointer-offline-0623 --message-format short --color never -- --test-threads=1 --nocapture
doc_type: module-detail
---

# Shell Pointer Bridge

`shell_pointer/` owns the retained-host Workbench shell pointer surfaces for tab drag/drop, floating-window attach targets, document-edge routes, and drawer resize splitters. The surfaces are manual runtime `UiSurface` trees because they represent editor chrome geometry rather than compiled pane templates.

## Route Intent Migration

Each interactive drag or resize node now receives a stable synthetic `UiRouteId` at surface construction time. `drag_surface.rs` and `resize_surface.rs` bind those ids into `EditorRouteIntentMap` as `EditorRouteIntent::ShellPointer(...)`. The bridge dispatches pointer input through `UiSurface::dispatch_input_event(...)`, then resolves the route from reply effect targets, reply handler, or runtime route target through the route-intent map.

This removes the old bridge-local node-id business mapping helpers. `route.rs` only declares `HostShellPointerRoute`; it no longer owns `drag_route_from_node(...)` or `resize_group_from_dispatch(...)`.

## Capture Model

Drawer resize capture is driven by runtime pointer dispatch effects. `resize_surface.rs` returns `UiPointerDispatchEffect::capture()` on primary down. `bridge.rs` consumes the resulting `UiDispatchEffect::CapturePointer` / `ReleasePointerCapture` in the `UiDispatchReply` path before falling back to handler or route target. Move and up events continue to route to the captured splitter until the release clears capture.

## Boundary Rules

- Keep geometry and target frames in `drag_surface.rs`, `resize_surface.rs`, and `effects.rs`.
- Keep route enum definitions in `route.rs`; do not add raw node-id match helpers there.
- Keep route id to shell route binding in the surface builders and route consumption in `bridge.rs`.
- Keep workspace docking behavior in `app/workspace_docking/**`; shell pointer should only report `HostShellPointerRoute`.

## Validation Notes

The 2026-06-23 M5.S1 slice passed offline `cargo check -p zircon_editor --lib` and offline `cargo test -p zircon_editor --lib shell_pointer` with 13 focused tests. Source scans confirmed no `drag_route_from_node`, `resize_group_from_dispatch`, or direct `dispatch_pointer_event(` remained in `shell_pointer/`.
