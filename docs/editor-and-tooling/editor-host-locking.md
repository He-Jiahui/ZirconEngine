---
related_code:
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/view_registry.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_listener_control.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_manager_minimal_host.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/startup/welcome_page.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/hydration.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/open.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/preview_refresh.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/binding.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/inspector.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/navigation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/node_ops.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/palette.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/source.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/style.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/broadcast.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/subscribe_editor_asset_changes.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/effects.rs
implementation_files:
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/view_registry.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_listener_control.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_manager_minimal_host.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/startup/welcome_page.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/editing.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/hydration.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/open.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/preview_refresh.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/save.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/sync.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/binding.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/inspector.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/navigation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/node_ops.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/palette.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/source.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/editing/style.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/broadcast.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/subscribe_editor_asset_changes.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/effects.rs
plan_sources:
  - user: 2026-05-03 修复其他 editor 生产路径 lock().unwrap() 热点
tests:
  - production scan: zircon_editor/src excluding tests has no lock().unwrap() matches
  - production scan: zircon_editor/src excluding tests has no lock().expect(...) matches
  - rustfmt --edition 2021 --check on edited Rust files
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-viewport-lock-gap --message-format short --color never
doc_type: module-detail
---

# Editor Host Locking

## Purpose

`zircon_editor::ui::host` owns shared workbench state for view registry, layout/session metadata, native windows, animation editor sessions, UI asset editor sessions, workspace watcher state, subsystem reports, and capability snapshots.

Production host code should not call `Mutex::lock().unwrap()` or `Mutex::lock().expect(...)` directly. A poisoned lock should be recovered through the owner-local helper so one panic while a host lock is held does not make later editor snapshots, layout commands, descriptor reads, asset editor refreshes, or shell pointer effects panic on lock acquisition.

## Owner Helpers

`EditorUiHost` exposes owner-local lock helpers in `editor_ui_host.rs`: `lock_session`, `lock_view_registry`, `lock_window_host_manager`, `lock_animation_editor_sessions`, `lock_ui_asset_sessions`, `lock_ui_asset_workspace_watcher`, `lock_subsystem_report`, and `lock_capability_snapshot`.

The helper policy is intentionally narrow: recover poisoned locks with `into_inner()` and keep the original operation result path unchanged. This round does not introduce a new editor error variant for poisoning, because the selected hot path was direct panic removal rather than semantic state reconciliation after a panic.

`DefaultEditorAssetManager` has a local `lock_change_subscribers` helper for subscriber broadcast state. `slint_host::shell_pointer::effects` uses a local drag-frame lock helper because that state is owned by the Slint shell pointer surface rather than by `EditorUiHost`.

## Validation Scope

The source scan covers production Rust files under `zircon_editor/src` and excludes test directories. It checks both direct `lock().unwrap()` and direct `lock().expect(...)`, including multi-line call chains.

This policy does not claim that all editor panic sites are gone. It only closes the production lock-poison panic surface covered by the 2026-05-03 host/session/startup/shell-pointer pass.
