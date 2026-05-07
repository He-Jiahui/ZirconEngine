---
related_code:
  - zircon_editor/src/ui/workbench/layout/activity_window_id.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_host_mode.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/main_host_page_layout.rs
  - zircon_editor/src/ui/workbench/layout/editor_main_frame_layout.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/window_registry/mod.rs
  - zircon_editor/src/ui/workbench/window_registry/editor_window_registry.rs
  - zircon_editor/src/ui/workbench/window_registry/window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/window_kind.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_view_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_binding.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_dock_position.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/native_window_close.rs
  - zircon_editor/src/ui/slint_host/app/close_prompt.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/close_prompt.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/close_prompt.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/tab_drag/resolved_drop.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/src/ui/workbench/model/build/workbench_view_model_build.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/presence.rs
  - zircon_editor/src/ui/workbench/view/activity_window_template_spec.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/asset_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.ui.toml
implementation_files:
  - zircon_editor/src/ui/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/ui/workbench/layout/activity_window_id.rs
  - zircon_editor/src/ui/workbench/layout/main_host_page_layout.rs
  - zircon_editor/src/ui/workbench/layout/editor_main_frame_layout.rs
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/layout/manager/apply.rs
  - zircon_editor/src/ui/workbench/layout/manager/attach.rs
  - zircon_editor/src/ui/workbench/layout/manager/detach.rs
  - zircon_editor/src/ui/workbench/layout/manager/focus.rs
  - zircon_editor/src/ui/workbench/layout/manager/normalize.rs
  - zircon_editor/src/ui/workbench/window_registry/mod.rs
  - zircon_editor/src/ui/workbench/window_registry/editor_window_registry.rs
  - zircon_editor/src/ui/workbench/window_registry/window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/window_kind.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_view_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_window_instance.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_binding.rs
  - zircon_editor/src/ui/workbench/window_registry/drawer_dock_position.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/native_window_close.rs
  - zircon_editor/src/ui/slint_host/app/close_prompt.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/close_prompt.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/close_prompt.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/tab_drop.rs
  - zircon_editor/src/ui/slint_host/tab_drag/resolved_drop.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_chrome_snapshot_build.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/main_page_snapshot.rs
  - zircon_editor/src/ui/workbench/model/build/workbench_view_model_build.rs
  - zircon_editor/src/ui/workbench/autolayout/region/tool_region/presence.rs
  - zircon_editor/src/ui/host/builtin_layout/hybrid_layout.rs
  - zircon_editor/src/ui/host/builtin_layout/workbench_page.rs
  - zircon_editor/src/ui/workbench/view/activity_window_template_spec.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/assets/ui/editor/host/editor_main_frame.ui.toml
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/asset_window.ui.toml
  - zircon_editor/assets/ui/editor/windows/ui_layout_editor_window.ui.toml
plan_sources:
  - user: 2026-04-25 ActivityDrawer should move into ActivityWindow; main frame keeps only task bar and window tabs; no Slint business UI
  - user: 2026-05-06 Drawer/Window/Menu Slate 化推进计划，要求抽屉实例、含抽屉 window、普通 window 由 editor 内单例 registry 管理
  - docs/superpowers/specs/2026-04-25-editor-activity-window-design.md
  - docs/superpowers/plans/2026-04-25-editor-activity-window-restructure.md
tests:
  - zircon_editor/src/tests/workbench/layout/activity_window_layout.rs
  - zircon_editor/src/tests/workbench/layout/editor_main_frame_layout.rs
  - zircon_editor/src/tests/workbench/layout/window_drawer_ownership.rs
  - zircon_editor/src/tests/workbench/registry/window_registry.rs
  - zircon_editor/src/ui/slint_host/app/tests/close_prompt.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/tab_drop.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/visual_screenshot.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/tests/editor_main_frame_template.rs
  - zircon_editor/tests/activity_drawer_window_template.rs
  - zircon_editor/tests/workbench_window_template.rs
  - zircon_editor/tests/asset_window_template.rs
  - zircon_editor/tests/ui_layout_editor_window_template.rs
  - cargo test -p zircon_editor --lib workbench_view_model_uses_only_active_activity_window_drawers --locked --no-run (blocked 2026-04-29 by unrelated zircon_runtime Hybrid GI source-type compile errors)
  - cargo fmt --package zircon_editor -- --check
  - cargo test -p zircon_editor --lib window_registry --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib window_registry_syncs_collapsed_active_drawer_without_selecting_tab --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (2026-05-07: 1 passed, 0 failed, 1104 filtered out)
  - cargo test -p zircon_editor --lib window_registry --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (2026-05-07: 5 passed, 0 failed, 1101 filtered out)
  - cargo test -p zircon_editor --lib tab_drop_dispatch_detaches_drawer_tab_to_independent_drawer_window --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib drawer --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib dirty_floating_window_close_request_shows_cancelable_prompt --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib dirty_floating_window_discard_prompt_closes_all_window_tabs --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib dirty_saveable_floating_window_save_prompt_saves_then_closes_window --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib dirty_main_window_discard_prompt_requests_host_exit --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib capture_close_prompt_visual_artifact --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --ignored --nocapture
  - visual screenshot: target/visual-layout/editor-window-20260507-close-prompt-900x620.png
doc_type: module-detail
---

# Activity Window Drawer Boundary

The editor main frame is now modeled separately from drawer-capable editor windows. `EditorMainFrameLayout` stores only the active window and ordered window tabs. Drawer-capable windows use `ActivityWindowLayout`, which owns `activity_drawers` and a content workspace.

The reusable `.ui.toml` shell is `editor.host.activity_drawer_window`. `WorkbenchWindow`, `AssetWindow`, and `UILayoutEditorWindow` reference that shell and mount their own left, right, bottom, and content regions. Slint remains a host projection layer; these new business window structures are `.ui.toml` assets.

`WorkbenchLayout::activity_windows()` remains the compatibility read surface for old layouts that still only persisted root `drawers`, but current presentation no longer treats the first or root drawer map as active by default. `MainHostPageLayout::WorkbenchPage` now binds the page to an `ActivityWindowId`, and `EditorChromeSnapshot::build(...)` projects drawer snapshots only from the activity window attached to `active_main_page`.

The view model and autolayout path consume that active-window drawer snapshot directly. `WorkbenchViewModel.drawer_ring` is hidden and empty when the active window has no configured drawers, so switching from `WorkbenchWindow` to an `AssetBrowserWindow` with no drawer configuration no longer preserves the previous left/right/bottom edge drawers. Drawer extent calculation also reads the active window drawer snapshots instead of the legacy root `WorkbenchLayout.drawers` map, preventing stale Workbench drawer widths from sizing a different activity window.

Drawer mutation commands (`OpenView` into a drawer, `SetDrawerMode`, `SetDrawerExtent`, and `ActivateDrawerTab`) now resolve through `WorkbenchLayout::active_activity_window_mut()`. Focusing a drawer tab records the owning activity window and re-activates the page bound to that window. This keeps layout state, snapshot projection, and host geometry aligned around the same ownership rule: the current `ActivityWindowLayout` decides whether left, right, and bottom drawers exist and how large they are.

`EditorWindowRegistry` is the runtime data-layer view of the same contract. `EditorUiHost` owns one registry instance, and `workspace_state` rebuilds it from the active `WorkbenchLayout` plus the current `ViewInstance` list when session metadata is recomputed. Ordinary windows are registered as `WindowKind::Ordinary`; windows with `activity_drawers` become `WindowKind::DrawerCapable`; detached drawer windows are tracked separately as `DrawerWindowInstance`.

Drawer registration is explicit. A `DrawerViewInstance` cannot bind to an ordinary window, and `selected_drawer_for_active_window()` returns `None` when the active window has no registered drawer view or when the drawer list is collapsed. Registry sync preserves collapsed drawer semantics by registering retained tabs for lookup while deriving the selected drawer only from each drawer layout's `active_view`; it does not synthesize a selection from the first tab. The public dock-position enum exposes `LeftTop`, `LeftBottom`, `Bottom`, `RightTop`, and `RightBottom`. Legacy `BottomLeft` / `BottomRight` inputs are migrated into the canonical `Bottom` drawer list in layout normalization and projection, so bottom drawers now have one ordering and one selection state.

Dragging a drawer tab out of every dock target now resolves to a detached-window drop route. Drawer-origin drags receive a `drawer-window:` page id, the layout command folds repeated detaches with the same id into one floating tab stack, and registry sync treats only that drawer-window prefix as `DrawerWindowInstance`; ordinary document floating windows are not reclassified as drawers. Removing the selected drawer tab collapses the source drawer list by clearing `active_view`, so a folded rail means no drawer view is open even when tab order is retained.

Native titlebar close requests participate in the same data layer. A floating drawer/document window asks the layout for all instances in that floating workspace, checks dirty `ViewInstance` metadata, and paints a host-owned `Save / Discard / Cancel` prompt into the corresponding native child window. `Cancel` clears only the prompt; `Discard` closes every tab in that floating window; `Save` attempts the registered UI-asset or animation-editor save path before closing. Main-window close uses the same prompt data and requests host exit only after the user chooses `Discard` or a successful `Save`. The 2026-05-07 Save-path regression opens a temporary `.ui.toml` UI asset, detaches it into a child window, marks it dirty, verifies the Save button is enabled, and closes the child window after the save action succeeds; the close-prompt screenshot artifact is recorded under `target/visual-layout/editor-window-20260507-close-prompt-900x620.png`.
