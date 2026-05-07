---
related_code:
  - zircon_editor/src/ui/workbench/window_registry/mod.rs
  - zircon_editor/src/ui/workbench/window_registry/editor_window_registry.rs
  - zircon_editor/src/ui/workbench/window_registry/window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/window_kind.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_view_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_binding.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_dock_position.rs
  - zircon_editor/src/ui/workbench/window_registry/menu_overflow_mode.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/attach.rs
  - zircon_editor/src/ui/workbench/layout/manager/detach.rs
  - zircon_editor/src/ui/workbench/layout/manager/focus.rs
  - zircon_editor/src/ui/workbench/layout/manager/normalize.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/native_window_close.rs
  - zircon_editor/src/ui/slint_host/app/close_prompt.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/native_window_close.rs
  - zircon_editor/src/ui/slint_host/app/close_prompt.rs
implementation_files:
  - zircon_editor/src/ui/workbench/window_registry/mod.rs
  - zircon_editor/src/ui/workbench/window_registry/editor_window_registry.rs
  - zircon_editor/src/ui/workbench/window_registry/window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/window_kind.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_view_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_binding.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_dock_position.rs
  - zircon_editor/src/ui/workbench/window_registry/menu_overflow_mode.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/workspace_state.rs
plan_sources:
  - user: 2026-05-06 Drawer/Window/Menu Slate 化推进计划，要求抽屉实例、含抽屉 window、普通 window 由 editor 内单例 registry 管理
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
tests:
  - zircon_editor/src/tests/workbench/registry/window_registry.rs
  - zircon_editor/src/ui/slint_host/app/tests/close_prompt.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/tab_drop.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/visual_screenshot.rs
  - cargo test -p zircon_editor --lib window_registry --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib window_registry_syncs_collapsed_active_drawer_without_selecting_tab --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (2026-05-07: 1 passed, 0 failed, 1104 filtered out)
  - cargo test -p zircon_editor --lib window_registry --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (2026-05-07: 5 passed, 0 failed, 1101 filtered out)
  - cargo test -p zircon_editor --lib window_registry_syncs_detached_drawer_window_without_reclassifying_plain_floating_windows --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib tab_drop_dispatch_detaches_drawer_tab_to_independent_drawer_window --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib dirty_floating_window_discard_prompt_closes_all_window_tabs --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib close_prompt --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib capture_close_prompt_visual_artifact --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --ignored --nocapture
  - visual screenshot: target/visual-layout/editor-window-20260507-close-prompt-900x620.png
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never
  - cargo test -p zircon_editor --lib menu_overflow_preference --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib window_registry --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Editor Window Registry

`EditorWindowRegistry` is the editor-owned registry for activity windows and drawer ownership. It is deliberately scoped to the editor host/runtime instance and is held by `EditorUiHost`; it is not a global mutable static. The registry gives the host a single query surface for `get_window`, `get_drawer_view`, and `get_drawer_window` style access while the persisted layout remains the durable source for actual workbench placement.

The registry tracks three instance groups:

- `WindowInstance` records ordinary activity windows and drawer-capable activity windows.
- `DrawerViewInstance` records drawer views that are owned by a registered drawer-capable window.
- `DrawerWindowInstance` records detached drawer windows that use their own page/window id but still preserve the focused drawer view and title.

`WindowKind` is the gate that prevents every window from accidentally receiving drawer UI. `register_drawer_view` first resolves the owner `ActivityWindowId`, then rejects the registration unless the window is drawer-capable. This matches the editor behavior that a normal window only receives top/page/content chrome, while a drawer-capable window may show left/top, left/bottom, bottom, right/top, and right/bottom drawer lists.

`DrawerDockPosition` is the public dock-position contract used by registration and binding. The current UI exposes `LeftTop`, `LeftBottom`, `Bottom`, `RightTop`, and `RightBottom`; legacy `BottomLeft` and `BottomRight` persisted slots are normalized to the single public `Bottom` position. `primary_slot()` still maps `Bottom` back to a layout slot so the existing `ActivityDrawerLayout` storage can be reused without introducing a second persisted drawer model.

`sync_from_layout` rebuilds a registry snapshot from `WorkbenchLayout.activity_windows()` and the current `ViewInstance` list. Windows with `activity_drawers` become `DrawerCapable`; windows without drawers become `Ordinary`. Floating windows are projected as `DrawerWindowInstance` only when their page id uses the `drawer-window:` prefix created by drawer detach routing; plain document floating windows remain ordinary floating workspaces and are not exposed as drawer windows. During sync, drawer tabs are still registered for typed lookup, but the owning window's `selected_drawer` is copied from drawer `active_view` only; a collapsed drawer with retained tabs and `active_view = None` therefore stays unselected instead of reopening the first tab. The same sync copies each window's `menu_overflow_mode`, so host-side registry queries observe the persisted menu popup preference. `workspace_state` calls this sync while recomputing session metadata so the host observes the same active-window drawer and menu facts that layout, projection, and hit testing use.

`ActivityWindowLayout.menu_overflow_mode` is the durable per-window setting for menu popup overflow. Its serde default is `Auto`, which keeps older project/global layouts compatible. `EditorChromeSnapshot::build(...)` reads the active activity window's setting and exposes it as `EditorChromeSnapshot.menu_overflow_mode`; the Slint menu pointer builder then projects that value into `HostMenuPointerLayout`. This gives `MenuOverflowMode::MultiColumn` a production layout/config path instead of leaving it as a test-only layout override.

Runtime mutation still flows through layout commands. `SetDrawerMode`, `SetDrawerExtent`, `ActivateDrawerTab`, drawer attach, drawer detach, and drawer focus now resolve through the active `ActivityWindowLayout` rather than legacy root drawers. Collapsing a drawer list means the tabs/order can remain, but no drawer view is selected/open. Switching to a window without a registered drawer-capable layout therefore leaves `selected_drawer_for_active_window()` empty and prevents the previous window's drawer from leaking into the new window.

Drawer detach is modeled as a real layout operation, not a painter-only overlay. The drag bridge emits a detach route when a tab is released outside any dock target; repeated detaches into the same floating page id append to that floating tab stack. The native close flow then queries the same floating workspace to build dirty close prompts, so `Discard` can close every tab in the detached drawer window and `Cancel` leaves both the registry and layout untouched.

This registry is a data-layer contract for the Slate-like host model. It is intentionally not a painter, not a hit-test table, and not a `.ui.toml` parser. The host still routes UI through `UiSurfaceFrame`, projected chrome frames, and the native pointer bridges; the registry only answers which windows and drawer views exist, which one is active, and whether a drawer can legally attach to a target window.
