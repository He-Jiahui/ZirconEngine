---
related_code:
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/template/mod.rs
  - zircon_editor/src/ui/template/catalog.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template/adapter.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/src/tests/ui/binding/mod.rs
  - zircon_editor/src/tests/ui/binding/animation.rs
  - zircon_editor/src/tests/ui/binding/asset_selection.rs
  - zircon_editor/src/tests/ui/binding/dock_and_welcome.rs
  - zircon_editor/src/tests/ui/binding/inspector_and_draft.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/ui/template/mod.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_ui/src/template/mod.rs
  - zircon_ui/src/template/document.rs
  - zircon_ui/src/template/instance.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/host_nodes.rs
  - zircon_editor/src/ui/template_runtime/model.rs
  - zircon_editor/src/ui/template_runtime/runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/harness.rs
  - zircon_editor/src/core/editor_event/runtime.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/projection_support.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/source_frames.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/surface.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/host_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/slint_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/slint_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/ui/slint_host/app/assets.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/app/native_windows.rs
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/tab_drop.rs
- zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/host_page_pointer/mod.rs
- zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs
- zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/ui/slint_host/detail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/scroll_surface_host.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_popup_state.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/ui/floating_windows.rs
  - zircon_editor/src/core/host/manager/layout_commands.rs
  - zircon_editor/src/core/host/manager/window_host_manager.rs
  - zircon_editor/src/core/host/manager/workspace_state.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/workbench/layout/mod.rs
  - zircon_editor/src/ui/workbench/layout/layout_manager.rs
  - zircon_editor/src/ui/workbench/layout/manager/mod.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/pane_fields.slint
  - zircon_editor/ui/workbench/ui_asset_editor_pane.slint
  - zircon_editor/ui/workbench/ui_asset_editor_data.slint
  - zircon_editor/ui/workbench/ui_asset_editor_components.slint
  - zircon_editor/ui/workbench/ui_asset_editor_center_column.slint
  - zircon_editor/ui/workbench/ui_asset_editor_inspector_panel.slint
  - zircon_editor/ui/workbench/ui_asset_editor_stylesheet_panel.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/viewport_toolbar.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/support.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/workbench/template_bridge.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/mod.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/viewport/toolbar_dispatch.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/asset/template_bridge.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/inspector/template_parity.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/inspector/batch_apply.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/welcome/project_name.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/welcome/open_recent.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/pane/trigger_action.rs
  - zircon_editor/src/tests/host/slint_activity_rail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_host_page_pointer/mod.rs
- zircon_editor/src/tests/host/slint_document_tab_pointer/
  - zircon_editor/src/tests/host/slint_drawer_header_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_detail_pointer/mod.rs
- zircon_editor/src/tests/host/slint_list_pointer/
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/src/tests/host/slint_drawer_resize/mod.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/tests/workbench_drag_targets.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/tests/workbench/view_model/document_workspace.rs
  - zircon_editor/src/tests/workbench/view_model/welcome_page.rs
  - zircon_editor/src/tests/workbench/view_model/support.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
implementation_files:
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/template/mod.rs
  - zircon_editor/src/ui/template/catalog.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template/adapter.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/host_nodes.rs
  - zircon_editor/src/ui/template_runtime/model.rs
  - zircon_editor/src/ui/template_runtime/runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/harness.rs
  - zircon_editor/src/core/editor_event/runtime.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/projection_support.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/error.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/host_projection.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/slint_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/slint_host/app/welcome_recent_pointer.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/ui/slint_host/app/assets.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/floating_window_projection.rs
  - zircon_editor/src/ui/slint_host/app/native_windows.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs
- zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/host_page_pointer/mod.rs
- zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs
- zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/detail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/scroll_surface_host.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_popup_state.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/core/host/manager/layout_commands.rs
  - zircon_editor/src/core/host/manager/window_host_manager.rs
  - zircon_editor/src/core/host/manager/workspace_state.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/workbench/layout/mod.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/pane_fields.slint
  - zircon_editor/ui/workbench/ui_asset_editor_pane.slint
  - zircon_editor/ui/workbench/ui_asset_editor_data.slint
  - zircon_editor/ui/workbench/ui_asset_editor_components.slint
  - zircon_editor/ui/workbench/ui_asset_editor_center_column.slint
  - zircon_editor/ui/workbench/ui_asset_editor_inspector_panel.slint
  - zircon_editor/ui/workbench/ui_asset_editor_stylesheet_panel.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
plan_sources:
  - user: 2026-04-15 按自定义 TOML 描述文件运行时构建 Slint 树并严格服从 Shared Layout 契约
  - user: 2026-04-15 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-16 把非-menu 的 popup/dialog/tree/list scroll 输入继续迁到同一套 shared pointer dispatcher
  - user: 2026-04-16 继续下一刀，把 secondary native window presenter 接到真实 slint_host
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/编辑器 UI 兼容迁移计划（自定义描述文件运行时构建 Slint 树，严格服从 Shared Layout 契约）.md
tests:
  - zircon_editor/src/tests/ui/binding/animation.rs
  - zircon_editor/src/tests/ui/binding/asset_selection.rs
  - zircon_editor/src/tests/ui/binding/dock_and_welcome.rs
  - zircon_editor/src/tests/ui/binding/inspector_and_draft.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_editor/src/tests/host/slint_activity_rail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_host_page_pointer/mod.rs
- zircon_editor/src/tests/host/slint_document_tab_pointer/
  - zircon_editor/src/tests/host/slint_drawer_header_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_detail_pointer/mod.rs
- zircon_editor/src/tests/host/slint_list_pointer/
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/slint_event_bridge/mod.rs
- zircon_editor/src/tests/host/slint_window/
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/tests/workbench_drag_targets.rs
  - zircon_editor/tests/native_window_hosts.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/tests/workbench/view_model/document_workspace.rs
  - zircon_editor/src/tests/workbench/view_model/welcome_page.rs
  - zircon_editor/src/tests/workbench/view_model/support.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - cargo test -p zircon_editor template -- --nocapture
  - cargo test -p zircon_editor --lib binding -- --nocapture
  - cargo test -p zircon_editor --lib -- --nocapture
  - cargo test -p zircon_editor --lib --locked -- --nocapture
  - cargo test -p zircon_editor --lib binding_dispatch -- --nocapture
  - cargo test -p zircon_editor --lib template_runtime -- --nocapture
  - cargo test -p zircon_editor --lib template_runtime --locked -- --nocapture
  - cargo test -p zircon_editor --lib builtin_host_runtime_exposes_only_generic_host_window_document_id --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib catalog_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib slint_callback_dispatch -- --nocapture
  - cargo test -p zircon_editor --lib slint_callback_dispatch --locked -- --nocapture
  - cargo test -p zircon_editor --lib shared_viewport_surface_replaces_legacy_direct_pointer_callback_abi --locked -- --nocapture
  - cargo test -p zircon_editor --lib slint_event_bridge -- --nocapture
  - cargo test -p zircon_editor --lib editor_event::runtime -- --nocapture
  - cargo test -p zircon_editor --lib slint_tab_drag --locked -- --nocapture
  - cargo test -p zircon_editor --test workbench_autolayout floating_window --locked -- --nocapture
  - cargo test -p zircon_editor --test workbench_drag_targets --locked -- --nocapture
  - cargo test -p zircon_editor --test native_window_hosts --locked -- --nocapture
  - cargo test -p zircon_editor --lib slint_window --locked -- --nocapture
  - cargo test -p zircon_editor --lib viewport_toolbar -- --nocapture
  - cargo test -p zircon_editor --lib slint_viewport_toolbar_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib viewport_ --offline -- --nocapture
  - cargo test -p zircon_editor --lib workbench_view_model_exposes_floating_windows_as_workspace_tabs --locked -- --nocapture
  - cargo test -p zircon_editor --lib host::slint_host::ui::tests --locked -- --nocapture
  - cargo test -p zircon_editor --test workbench_slint_shell -- --nocapture
  - cargo test -p zircon_editor --lib slint_detail_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_activity_rail_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_host_page_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_document_tab_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_drawer_header_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_ --offline -- --nocapture
  - cargo check -p zircon_editor --lib --offline
  - cargo test -p zircon_editor --test workbench_slint_shell --offline -- --nocapture
  - cargo test -p zircon_editor --lib --locked -- --nocapture
  - cargo test -p zircon_editor --lib resolve_floating_window_focus_instance_ --locked -- --nocapture
  - cargo check -p zircon_graphics --lib --locked
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib uses_region_frame_fallback_in_real_host --locked -- --nocapture
doc_type: module-detail
---

# Editor Template Compatibility Migration

## Purpose

这份文档记录“保留 Slint 宿主，但把 editor 壳体模板真源前移到 TOML + shared template runtime”的第一段兼容迁移实现。

当前已经落地的 editor-only 边界是：

- `zircon_ui` 负责模板文档、slot 语义、运行时实例展开，以及 shared `UiTree` / `UiSurface` 的第一段模板桥接
- `zircon_editor::ui` 负责 editor composite catalog、模板文档注册，以及稳定 binding id 到 typed `EditorUiBinding` 的解析
- `zircon_editor` 和 Slint host 还没有在这一轮被强制改写，只保留后续接线入口

这样做的目的不是拖慢迁移，而是避免在 `slint_host` 主链正活跃时又把一半模板逻辑散落进 editor host。

## Latest Compatibility Cutover

最新一刀已经把 viewport 外层原始输入 ABI 也收口到 shared pointer seam：

- `workbench.slint` 不再向 host 暴露 `viewport_pointer_moved` / `viewport_left_pressed` / `viewport_scrolled` 这类 7 个分散回调
- Slint 现在只上报统一 `viewport_pointer_event(kind, button, x, y, delta)` 事实
- `callback_wiring.rs` 和 `app/viewport.rs` 不再把这条链直接旁路到 `InputManager`
- 真实宿主改为复用 `callback_dispatch.rs` 里的 `SharedViewportPointerBridge + dispatch_viewport_pointer_event(...)`

这一步的意义不是“少几个回调名”，而是把 editor host 最后那段 viewport raw pointer authority 从 Slint/宿主局部逻辑，继续拉回 shared `UiSurface + UiPointerDispatcher` 语义里，和 menu/list/toolbar/tree/scroll 的迁移方向一致。

这一轮又把 root document viewport toolbar 的 surface-size authority 继续拉回 shared projection：

- [`app/viewport.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/viewport.rs) 现在会在 root document host 上用 `resolve_root_viewport_content_frame(..., root_shell_frames, true)` 计算 toolbar surface 宽度，而不是只读 legacy `shell_geometry.document width`
- 这意味着 `viewport_toolbar_pointer_clicked(...)` 在 `control rect == 0` 时，不只是 action frame 会回退到 builtin projection；连 projection 本身的 recompute 宽度也会跟着 shared `PaneSurfaceRoot` / viewport frame 走
- [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 新增 `root_viewport_toolbar_pointer_click_prefers_shared_projection_surface_width_over_stale_document_geometry`，直接锁住“document geometry 过窄但 shared root projection 仍正确”时的真实宿主 fallback 行为

同一阶段，真实宿主又补上了一条 projection-backed pane frame fallback，而不是继续默认“callback 上报的宽高必然可信”：

- [`BuiltinWorkbenchTemplateBridge::control_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在把 builtin workbench template 的 shared frame 查询暴露给真实 `slint_host`
- [`PANE_SURFACE_CONTROL_ID`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/constants.rs) 固定指向 `PaneSurfaceRoot`
- [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 里的 welcome recent sync 现在会在 Slint `width/height == 0` 时优先回退到 `PaneSurfaceRoot` 的 shared frame，再继续 retained pointer layout / hit-test / scroll state 同步
- [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 的 `root_welcome_recent_pointer_click_uses_projection_fallback_in_real_host` 和 [`template_bridge/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs) 共同锁住这条 seam

这说明兼容迁移现在不只是“模板 runtime 能产出一棵 shared tree”，而是已经开始反向给真实宿主 pointer 输入提供 canonical pane frame authority；`.slint` 壳层继续只上传 pointer fact，不再对 pane 几何拥有独占解释权。

同一条 authority seam 这轮又继续往 pane-local scroll surface 推进：

- [`resolve_callback_surface_size_for_kind(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs) 现在继续服务 root-shell `Hierarchy` / `Console` / `Inspector`，以及 exclusive `AssetBrowser` details rail
- `browser_asset_details_pointer_scrolled(...)` 不再保留“只要 Slint callback 给 0x0 就停在本地零尺寸 surface”的旧行为；它已经和其它 pane-local scroll 一样回退到 shared host frame，再进入 `ScrollSurfaceHostState`
- 新增的 real-host regression 现在锁定四条 root-shell 0-size callback 路径：`Hierarchy`、`Console`、`Inspector`、`AssetBrowser details`
- 这一轮同一条 seam 又继续推到 asset workspace retained pointer surface：[`resolve_callback_surface_size_for_asset_surface(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs) 现在把 `surface_mode -> ViewContentKind::{Assets, AssetBrowser}` 固定成宿主共享规则
- [`asset_tree_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/asset_tree_pointer.rs)、[`asset_content_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs) 和 [`asset_reference_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/asset_reference_pointer.rs) 不再把 callback `width/height` 直接写成 viewport size；当 Slint 继续上传 `0x0` 时，它们会先回退到 cached size，再回退到 shared host frame
- 新的 real-host regression 又把这条兼容契约补到 6 个 asset pane 用例：`activity/browser` 两条 surface 各自覆盖 `tree/content/reference`，确保 template/runtime authority 不会在真实宿主里因为零尺寸 callback 被局部 Slint 几何重新接管
- `welcome_recent_pointer.rs` 这轮也去掉了剩余的直接 `UiSize::new(width, height)` 宿主写入；现在 `clicked/moved/scrolled` 三条链都会先经过 `resolve_callback_surface_size_for_kind(..., ViewContentKind::Welcome)`
- 新增的 real-host regressions 进一步锁住了 Welcome recent 的 cached-size 优先级：当 recent list 已经缓存了有效 viewport size，而 callback 继续上报 `0x0` 时，兼容链必须保留 cached size，而不是直接跳回 projection frame

这一步的重要性在于：兼容迁移已经从“模板 tree / binding / route 的 shared authority”继续推进到“真实宿主 pane 尺寸恢复策略”本身，shared template bridge 和 shared shell geometry 开始共同决定 pointer/scroll surface 的有效尺寸。

同一条兼容链这轮又第一次把 shared projection frame 真正推入 root host presentation，而不是只停在 callback fallback：

- [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在把 builtin workbench root shell control frame 作为稳定宿主结构导出
- [`apply_presentation(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) 新增 `shared_root_frames`，开始让 root shell 的 `center_band_frame` / `status_bar_frame` 直接消费 shared projection；`document_region_frame` 则在 drawer 折叠时对齐 `DocumentHostRoot`，drawer 可见时仍保留 geometry authority
  - `viewport_content_frame` 现在也服从同一套 mixed authority：drawer 折叠时从 `PaneSurfaceRoot` 推导，`Scene` 额外扣掉 toolbar 高度，drawer 可见时仍回退 geometry
  - [`host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 会把 `template_bridge.root_shell_frames()` 接入 root recompute；child native window presenter 路径显式传 `None`，因此 secondary host 仍保留当前窗口几何边界，不会被这次 root-shell cutover 误伤
  - root host 现在还会把这份 resolved viewport frame 同时喂给 `viewport_size` 与 `SharedViewportPointerBridge`，因此真实 WGPU viewport 尺寸、shared pointer bounds 与 Slint presentation 不再各自持有一份独立 frame
  - visible drawer shell/header 这轮也开始吃同一份 root-frame bundle：[`resolve_root_left_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs)、[`resolve_root_right_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 与 [`resolve_root_bottom_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 现在会在 drawer 可见时用 shared `shell_frame + workbench_body_frame` 重建 root drawer shell 的 `x/y`，但继续保留 legacy geometry 的 `width/height`
  - [`build_workbench_drawer_header_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs) 与 [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 现在复用同一套 resolved frame，因此 visible drawer shell 的 Slint presentation 与 drawer header retained pointer layout 已经重新回到同一条 shared authority 链
  - focused regressions `apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_positions` 和 `shared_drawer_header_pointer_layout_prefers_shared_root_projection_for_visible_drawer_regions` 已经 green；当前这条兼容链不再以 `legacy_menu_button_frames(...)` 或 host-page metric strip 为主要 boundary，而是转向 dynamic floating-window shell 仍通过 editor-only layout geometry 生成 outer frame，再由统一 projection helper 分发给各个 consumer
  - 同一份 root-shell frame bundle 这轮又继续向 root main `document tab` pointer surface 扩张：
  - [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在额外导出 `DocumentTabsRoot`
  - [`build_workbench_document_tab_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs) 现在会在 `Left/Right/Bottom` drawer region 全部折叠时优先吃 shared `DocumentTabsRoot` frame，而不是继续只从 `geometry.region_frame(Document)` 派生 main strip
  - 这把 root-shell `document tab` 命中 authority 拉回到和 root presentation / viewport toolbar 相同的 shared projection 契约上，避免真实宿主在 document geometry 滞后时把 tab surface 根框截窄
  - 同一份 frame bundle 这轮也第一次进入 root 左侧 `activity rail` 的真实宿主命中主链：[`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在额外导出 `ActivityRailRoot`，而 [`resolve_root_activity_rail_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) / [`build_workbench_activity_rail_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/activity_rail_pointer/build_workbench_activity_rail_pointer_layout.rs) / [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 会在 drawer 折叠时优先吃 shared `ActivityRailRoot` frame
  - 这条 seam 的兼容约束和 `document tab` 一样保持 mixed authority：只要还有可见 drawer region，左 rail 仍回退 legacy left-region geometry；当 `Left/Right/Bottom` 都折叠时，真实宿主必须继续服从 shared root projection。对应 real-host regression `root_activity_rail_pointer_click_prefers_shared_projection_surface_when_left_region_geometry_is_stale` 与 pure regression `shared_activity_rail_pointer_layout_prefers_shared_root_projection_when_left_region_geometry_is_stale` 已一起锁住
  - 同一份 frame bundle 这轮也终于接到了 root `host page` strip：`root_shell_frames()` 现在额外导出 `WorkbenchShellRoot`，而 [`build_workbench_host_page_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_page_pointer/build_workbench_host_page_pointer_layout.rs) / [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 会优先吃 shared shell width，再叠加原有 `top_bar_height/host_bar_height` 契约，因此 host-page 根 strip 不会再被 `TAB_MIN_WIDTH` 的 legacy metric estimate 截窄
  - 这条兼容 seam 的 real-host regression `root_host_page_pointer_click_prefers_shared_projection_shell_width_over_metric_strip_estimate` 已经 focused green；更宽的 pure builder / template-bridge 复跑则暂时被邻接 `workbench.slint` 的 `UiAssetCanvasNodeData` / preview-surface Slint build-script 漂移阻塞，所以当前仍按“主链兼容 green、广义重跑有无关 blocker”记账
- [`ui/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/tests.rs) 和 [`template_bridge/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs) 共同锁住了这条新 seam：root presentation 何时必须吃 shared projection、何时仍要保留 geometry，以及 builtin control-frame 映射本身
- [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 现在再锁 `root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed`；它在生产改动前给出的真实 red 是 `1600x876` 对 `1544x884`
- 这条兼容回归现在已经重新跑到 green：两条 `apply_presentation_*`、`builtin_workbench_template_bridge_recomputes_surface_backed_frames_with_shell_size`、`root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed`，以及 `cargo check -p zircon_editor --lib --locked` 都已通过；因此 root-shell frame authority 这条兼容 seam 不再只是“red evidence + compile green”，而是有直接的 focused green rerun
- 新增这轮 focused green 还包括：`cargo test -p zircon_editor --lib apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_positions --locked -- --nocapture` 与 `cargo test -p zircon_editor --lib shared_drawer_header_pointer_layout_prefers_shared_root_projection_for_visible_drawer_regions --locked -- --nocapture`
- 同一条 seam 这轮又继续推到了 drop 解释层，而不是只停在 presentation / drag overlay：[`resolve_workbench_tab_drop_route_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/route_resolution.rs) 和 [`resolve_tab_drop_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/drop_resolution.rs) 现在会把 builtin `root_shell_frames()` 一起传到 precise tab-strip hitbox；[`strip_hitbox.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs) 也会改用 resolved `WorkbenchBody` / `DocumentHostRoot` frame，而不是在最后一步把 attach anchor 又交回旧 `WorkbenchShellGeometry`
- [`workspace_docking.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/workspace_docking.rs) 因此不再只是“拿 shared pointer route 决定 group key”；真实宿主 drop dispatch 现在也会把同一份 root-shell frame authority 带进 attach 解析，避免 `pointer route` 与 `precise attach anchor` 来自两套不同的 `x/y` 事实
- 新的 regression [`root_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs) 里的 `resolve_workbench_tab_drop_route_uses_shared_root_projection_tab_strip_when_drawers_are_collapsed` 直接锁定这个断层：shared pointer route 已经命中右侧 tool stack，但旧逻辑会因为 tab strip 仍按旧 center-band `y` 计算而退化成 `Drawer(RightTop) + anchor None`
- 同一轮又把 pure helper/parity 入口补齐到了同一份 authority：[`resolve_workbench_drag_target_group_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/bridge.rs) 现在允许纯函数 drag-target 解析显式接收 builtin `root_shell_frames()`；focused regression `resolve_workbench_drag_target_group_with_root_frames_uses_shared_root_projection_document_bounds_when_drawers_are_collapsed` 已经 green，因此 shared root projection 不再只存在于真实宿主 path
- 当前验证状态更新为：`cargo check -p zircon_editor --lib --locked` 已重新通过；但更宽的 `cargo test -p zircon_editor --lib slint_tab_drag --locked -- --nocapture` / exact `--lib` rerun 仍会被邻接 `src/tests/editing/ui_asset/**` 与 `src/tests/host/manager/**` 的 `UI Asset Editor` 漂移抢先拦住，所以这里记为“focused green, broader lib-test blocked by unrelated ui_asset drift”

这意味着兼容迁移已经不只是“template runtime 能解释 callback / route / fallback frame”，而是开始让真实 Slint host 的 presentation 也服从 shared projection 契约。后续 clip/z-order 和更多 pane shell 继续 cutover 时，不需要再重新定义一套新的 presentation authority。

## Editor-Only Types

### `EditorComponentCatalog`

这一层记录 editor composite 的最小描述：

- `component_id`
- `document_id`
- `binding_namespace`

它的职责不是存放 layout/frame 结果，而是明确：

- 哪些 composite 属于 editor-only 目录
- 哪份模板文档拥有这个 composite
- 它对外稳定暴露的 binding 命名空间是什么

当前这能直接覆盖诸如：

- `WorkbenchShell`
- `MenuBar`
- 后续的 `ActivityRail`
- `DocumentTabs`
- `StatusBar`

### `EditorTemplateRegistry`

registry 负责托管已加载的 `UiTemplateDocument`，并对外提供：

- `register_document(...)`
- `document(...)`
- `instantiate(...)`

这让 editor 迁移后可以先把模板文档作为 build/embed 资源注册进来，再由运行时按需实例化，而不是重新让 `.slint` 业务壳承担模板装配权。

### `EditorTemplateAdapter`

adapter 当前只做一件事：把 shared `UiBindingRef` 解析为 typed `EditorUiBinding`。

它内部维护的是：

- `binding_id -> EditorUiBinding`

解析时会：

- 检查 stable binding id 是否存在
- 检查 `UiBindingRef.event` 和已注册 typed binding 的事件种类是否一致
- 返回稳定顺序的 `EditorUiBinding` 列表，供后续 host/runtime 接线

这一步非常关键，因为它把模板文档里的稳定命名空间，和 editor-only 的 typed command payload 严格分层了。

## Stable Binding Contract

当前迁移切片默认采用以下约束：

- TOML 里只写稳定 `UiBindingRef`
- `UiBindingRef.id` 使用稳定 shell 命名空间
- `UiBindingRef.route` 只是 route key，不直接等于宿主 callback
- 真正的 typed payload 仍然由 `EditorUiBinding` / `EditorUiBindingPayload` 权威定义

因此像 `WorkbenchMenuBar/SaveProject` 这样的 binding id，在模板里只是稳定引用；具体它最终映射到：

- `MenuAction("SaveProject")`
- `DockCommand::*`
- `ViewportCommand::*`

仍然由 editor-only adapter/runtime 决定。

## Repository Template Asset

当前仓库已经放入第一份真实 editor template 资产：

- [workbench_shell.ui.toml](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)

它现在承担的是“复合装配层 baseline + shared layout 契约入口”，不是最终的手写 `.slint` 业务树。当前覆盖的 composite 有：

- `WorkbenchShell`
- `MenuBar`
- `ActivityRail`
- `DocumentHost`
- `StatusBar`

除了这些 editor composite，本轮还在模板里显式引入了通用 shared container 节点与 layout 描述，例如：

- `WorkbenchShell` 根节点通过 `attributes.layout.container = VerticalBox` 下沉 shared 布局语义
- `WorkbenchBody` 使用通用 `HorizontalBox` 把 `ActivityRail` 和 `DocumentHost` 组合成主体区
- toolbar / activity rail / tabs / status bar / pane surface 都通过 `attributes.layout` 显式给出宽高、stretch、gap、clip 和滚动语义

这一步的意义是把模板接口从测试字符串推进到真实仓库源文件，同时让 editor shell 的 shared frame 结果终于能从模板契约直接求出来，而不是再回退到手写 `.slint` 公式。

## Editor Host Runtime Scaffold

这一轮先把下一层安全骨架补到了 `zircon_editor`；在本文后半段，又继续把最小的 `slint_host` callback 主链接缝补上了。

新增的宿主侧接口是：

- `EditorUiHostRuntime`
- `SlintUiProjection`
- `SlintUiNodeProjection`
- `SlintUiBindingProjection`
- `EditorUiCompatibilityHarness`

### `EditorUiHostRuntime`

它当前负责三件事：

1. 加载真实模板资产
2. 持有 editor component catalog / template registry / binding adapter
3. 把模板实例和 shared surface 投成 host-neutral `SlintUiProjection` / `SlintUiHostModel`

当前内置入口 `load_builtin_workbench_shell()` 会：

- 加载 [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)
- 注册 `WorkbenchShell`、`MenuBar`、`ActivityRail`、`DocumentHost`、`StatusBar` 这些 editor component descriptor
- 注册第一批稳定 binding id 到 typed `EditorUiBinding` 的默认映射

这意味着后续 editor host 接 Slint 时，至少不需要再手写“怎么从模板资产变成一棵宿主节点树”。

这一轮又补上了一条更靠近 shared core 的运行时入口：

- `build_shared_surface(document_id)`

它会直接把模板文档实例化之后交给 `zircon_ui::template::UiTemplateSurfaceBuilder`，产出带模板元数据的 shared `UiSurface`。因此 editor runtime 现在已经不是只会“模板 -> host projection”，而是已经具备：

- 模板 -> shared surface
- 模板 -> host-neutral projection
- 模板 -> Slint host projection

这对后续真正把 layout、hit-test、focus 和 pointer route 统一下沉到 shared core 很关键，因为 editor 侧终于能在不碰旧 `slint_host` 的情况下直接拿到 shared retained tree。

在这条链路上，本轮又补上了：

- `build_host_model_with_surface(projection, surface)`
- `build_slint_host_projection_with_surface(projection, surface)`

这两个入口会把 shared `UiSurface` 的权威结果直接带进宿主模型，因此 host/slint 投影不再只有 component/control/binding 信息，还开始消费：

- `frame`
- `clip_frame`
- `z_index`

### `SlintUiProjection`

当前 projection 还是刻意收窄的 host-neutral 形态：

- `document_id`
- `root: SlintUiNodeProjection`
- `bindings: Vec<SlintUiBindingProjection>`

节点只先保留：

- `component`
- `control_id`
- `attributes`
- `style_tokens`
- `binding_ids`
- `children`

也就是说，这一层先把模板资产里的稳定结构、control id 和 binding id 收口成宿主可消费的数据模型，但还没有直接映射到现有 `WorkbenchShell.set_*` 那套 Slint 属性投影。

### `SlintUiHostModel`

这一轮继续把 projection 往真正的宿主接线方向推了一层，但仍然停在 `template_runtime` 内，不直接进入当前活跃的 `slint_host` 主链。

新增的 host-neutral 宿主节点模型是：

- `SlintUiHostModel`
- `SlintUiHostNodeProjection`
- `SlintUiHostBindingProjection`

这层模型把 `SlintUiProjection` 进一步压成稳定的 preorder node list，并保留：

- `node_id` / `parent_id`
- `component`
- `control_id`
- `frame`
- `clip_frame`
- `z_index`
- `attributes`
- `style_tokens`
- `binding_id`
- `event_kind`
- `route_id`

这样后续真正替换 `slint_host/ui.rs` 时，就不需要直接从模板实例树重新做一次散装遍历，而是可以消费已经定型的宿主节点列表。

更重要的是，这个 host model 已经显式携带 `route_id`，意味着：

- 模板里的稳定 binding id
- `EditorUiControlService` 注册后的共享 `UiRouteId`
- 未来 Slint host 的统一 dispatcher

现在已经能在同一个中间层里汇合，而不是继续依赖旧壳体里分散的 callback 命名。

### `SlintUiHostAdapter`

这一刀继续把“host-neutral 节点列表”往真正的 Slint 宿主可绑定形态推进了一层，但仍然不直接接入当前手写 `workbench.slint`。

新增接口是：

- `SlintUiHostAdapter`
- `SlintUiHostProjection`
- `SlintUiHostNodeModel`
- `SlintUiHostComponentKind`
- `SlintUiHostRouteProjection`
- `SlintUiHostValue`

这层 adapter 做的事情很克制：

- 根据 `component` 归类通用宿主节点种类
- 从通用 `attributes` 里提炼 `text` / `icon`
- 保留 `control_id`
- 保留 route 绑定结果
- 把 TOML 值转换成更稳定的宿主属性值模型

当前已经覆盖的 kind 包括：

- `Root`
- `Toolbar`
- `IconButton`
- `ActivityRail`
- `DocumentHost`
- `HorizontalBox`
- `TabStrip`
- `PaneSurface`
- `StatusBar`
- `VerticalBox`
- `Label`

这意味着后续真正做通用 Slint host 组件库时，不需要再从 `toml::Value` 原始表里现场猜字段，而可以直接绑定：

- `kind`
- `frame`
- `clip_frame`
- `text`
- `icon`
- `routes`
- `style_tokens`

### Runtime Convenience Projection

`EditorUiHostRuntime` 现在除了：

- `project_document(...)`
- `build_host_model(...)`

还新增：

- `build_slint_host_projection(...)`

这样后续 editor host 做并行接线时，可以直接从 runtime 取到“已注册 route 的 Slint 宿主节点投影”，而不需要在调用侧自己重复组合：

1. template projection
2. route registration
3. host model build
4. adapter conversion

### Embedded Template Authority

`EditorUiHostRuntime::load_builtin_workbench_shell()` 现在不再通过运行时文件路径读取模板，而是使用 `include_str!` 嵌入 [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)。

这一步让当前链路更符合迁移计划里的“仓库模板源文件 + build/embed 作为发布权威”的要求：

- 发布态不再依赖运行时文件 IO 才能拿到 builtin workbench template
- 模板源文件仍然保留在仓库里，便于后续 debug hot-reload 再补回开发态覆盖
- 现有测试直接覆盖嵌入版本，不需要额外的宿主路径探测逻辑

### Route Stub Registration

`EditorUiHostRuntime::register_projection_routes(...)` 现在可以把 projection 里的 typed binding 注册成 `EditorUiControlService` route stub。

这一步的意义是：

- projection 里的稳定 binding id
- editor-only 的 typed `EditorUiBinding`
- 远控 / reflection / host 共享的 `UiRouteId`

三者开始在同一条宿主模板运行时骨架上汇合，而不是继续只靠旧手写壳体上的分散 callback。

## Real Slint Host Callback Bridge

这一刀第一次把 shared template runtime 真正接进了 `slint_host` 的实际 callback 路径，但仍然避开了高冲突的 `workbench.slint` 业务树重写。

新增的实际宿主桥是：

- `BuiltinWorkbenchTemplateBridge`

2026-04-16 之后，[`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 只保留结构入口；`BuiltinWorkbenchTemplateBridge` 的 owner 现在位于 folder-backed [`callback_dispatch/template_bridge/workbench/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/mod.rs)，其中 bridge 行为在 [`workbench/bridge.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs)，root-shell frame 声明在 [`workbench/root_shell_frames.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs)，builtin host projection 布局在 [`workbench/host_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/host_projection.rs)；workbench 菜单分发位于 [`callback_dispatch/workbench/menu_action.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs)，layout 行为位于 [`callback_dispatch/layout/`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/layout) 子树。

`BuiltinWorkbenchTemplateBridge` 负责：

1. 加载 builtin [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)
2. 构建 `SlintUiProjection`
3. 通过 shared `UiSurface::compute_layout(...)` 生成随 shell 尺寸变化的 shared-surface-backed `SlintUiHostProjection`
4. 从宿主投影里按 `control_id + event_kind` 查回稳定 binding id
5. 再把该 binding 作为 typed `EditorUiBinding` 交给 `EditorEventRuntime`

这条链路的关键点是：实际宿主不再需要先把 `"ResetLayout"`、`"SaveProject"` 这类字符串硬编码翻译成 editor event，而是可以先从共享模板投影解析“当前这个 control 在 shared core 权威下绑定了什么 typed payload”。

### `slint_host/app.rs` 中的最小 cutover

[`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 现在持有一份 builtin template bridge，并在宿主重排时同步重算它的 shared frame：

- shell 初始化时创建 bridge
- 每次 `recompute_if_dirty()` 时按当前 shell 尺寸更新 bridge 的 shared host projection
- `handle_menu_action(...)` 对已经进入 TOML 模板的菜单 control，优先走 template bridge
- 未模板化或仍保留宿主执行权的路径继续回退旧 callback 分发

当前保留的特殊分支仍然包括：

- 还未模板化的 legacy menu action

其中 `OpenProject` 这条菜单语义现在已经不再由 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 本地 special-case 拦截，而是变成：

- `WorkbenchMenuBar/OpenProject`
  -> builtin template binding
  -> `MenuAction::OpenProject`
  -> `EditorEventEffect::PresentWelcomeRequested`
  -> 宿主消费 effect 后展示 welcome surface

`SavePreset.*` / `LoadPreset.*` 这组动态菜单 id 这轮也已经不再由 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 本地前缀分支拦截，而是下沉到 [`callback_dispatch/workbench/menu_action.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs)：

- `dispatch_menu_action(...)`
  -> `slint_menu_action("SavePreset.<name>" | "LoadPreset.<name>")`
  -> canonical `EditorEvent::Layout(core::editor_event::LayoutCommand::SavePreset/LoadPreset)`
  -> `SlintDispatchEffects.active_layout_preset_name`
  -> 宿主只消费 effect，更新当前 preset 选择与状态行

scene 空态里的 `Open Scene` / `Create Scene` 按钮这轮也已经从 label fallback 改成真实 typed menu binding：

- [`workbench/model/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/model/mod.rs) 现在直接为 scene empty-state 生成 `MenuAction::OpenScene` / `MenuAction::CreateScene`
- [`ui.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui.rs) 不再需要用按钮文案反推 action id
- [`runtime.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/editor_event/runtime.rs) 统一产出当前占位状态 `"Scene open/create workflow is not wired yet"`

因此当前 `handle_menu_action(...)` 保留下来的本地分支只剩未模板化 legacy menu action；startup/welcome 的执行态仍然在宿主，但已经接入 runtime 的 menu 入口都回到了统一 template/adapter authority。

这条链路这轮又补了一条更硬的兼容约束：`WorkbenchMenuBar/ResetLayout` 继续保持 `MenuAction::ResetLayout`，而不是在模板侧提前改写成 `DockCommand::ResetToDefault`。真正的 layout reset 仍然在 runtime 内部执行 `core::editor_event::LayoutCommand::ResetToDefault`；这里锁住的是 canonical event log 真源，而 `MenuAction` / `LayoutCommand` 的 DTO owner 也已经收口到 [`core/editor_event/workbench`](/E:/Git/ZirconEngine/zircon_editor/src/core/editor_event/workbench/mod.rs)。这样旧手写 Slint 菜单入口和 builtin template 菜单入口在同一 fixture 上仍然会产出同构 `EditorEventRecord`，不会因为 cutover 路径不同而把同一个菜单动作记成两类上层事件。

### 当前已接入的真实收益

这一刀带来的不是“Slint 壳已经替换完成”，而是三件更基础但更关键的事情：

- `slint_host` 终于有了一个真实 consumer，会消费 shared `UiSurface` 求出的 frame/route，而不是只有测试在消费
- builtin TOML 模板里的稳定 binding id 开始进入真实 menu/dock callback 链路
- 后续继续迁移 activity rail、toolbar、dock target、viewport chrome 时，可以复用同一条 `control_id -> shared host projection -> typed binding` 解析链，而不是再复制一套字符串 callback 对照表

### Workbench structural tab cutover

这一轮把 editor shell 里四类最核心的“结构性标签输入”同时收口到了 shared `UiSurface + UiPointerDispatcher`，不再让手写 `.slint` callback 名称或 `(slot, id)` 字符串组合作为点击真源：

- `ActivityRail`
- host page strip
- document / floating document tabs
- drawer header tabs

新增的宿主桥接分别是：

- [`activity_rail_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs)
- [`host_page_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_page_pointer/mod.rs)
- [`document_tab_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs)
- [`drawer_header_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs)

它们都遵守同一条权威链：

Slint 上传 strip-local pointer 事实与必要的几何输入
-> host bridge rebuild focused shared `UiSurface`
-> `UiPointerDispatcher` 解析 top-most route
-> [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 把 route 投影到 builtin template binding 或兼容 fallback
-> `EditorEventRuntime`

当前 `workbench.slint` / `chrome.slint` 的职责已经进一步收窄成“表现层 + 原始 pointer 坐标上传层”：

- `RailButton.pointer_pressed(...)` 只上传 rail 内本地坐标
- `DockTabButton.pointer_pressed(...)` 只上传 host page strip 内本地坐标
- `TabChip.pointer_clicked(...)` / `close_pointer_clicked(...)` 只上传 document tab 的点击事实
- `DockTabButton.pointer_clicked(...)` 只上传 drawer header tab 的点击事实

对应地，真实宿主已经不再注册这些旧直连业务入口：

- `ui.on_activate_host_page(...)`
- `ui.on_activate_document_tab(...)`
- `ui.on_close_tab(...)`
- `ui.on_toggle_drawer_tab(...)`

现在统一替换成：

- `ui.on_activity_rail_pointer_clicked(...)`
- `ui.on_host_page_pointer_clicked(...)`
- `ui.on_document_tab_pointer_clicked(...)`
- `ui.on_document_tab_close_pointer_clicked(...)`
- `ui.on_drawer_header_pointer_clicked(...)`

这里有两个关键实现约束：

- host page 与 activity rail 使用 `pointer_pressed`，因为它们本身不承担拖拽语义，按下即可让 shared surface 做命中和 typed route 解析
- document tab 与 drawer header tab 使用 click-level pointer hook，而不是 `down`，因为它们和 drag/drop 共享同一颗 visual tab；只有沿用 Slint 现有 click suppression，才能避免拖拽开始时误发激活/关闭/折叠事件

这让 `BuiltinWorkbenchTemplateBridge` 在真实宿主里首次同时覆盖四类 shell structural route：

- `ActivityRail/*`
- `WorkbenchShell/ActivateMainPage`
- `DocumentTabs/ActivateTab`
- `DocumentTabs/CloseTab`

其中 drawer header toggle 仍然保留 editor shell 的既有兼容语义，但解释权已经从 Slint callback 参数本身转到了 shared route + template binding：

- route 落到当前激活且已展开的 drawer tab 时，仍会折叠该 drawer
- route 落到已折叠 drawer 的目标 tab 时，仍会重新激活并恢复可见
- 语义判断现在由 `dispatch_builtin_workbench_drawer_toggle(...)` 统一完成，而不是回到 `app.rs` 手写分支

因此当前真实宿主里的结构性标签输入已经不再是“模板能解释一部分，但 Slint 仍保留第二真源”的过渡态，而是共享 pointer authority 真正进入主链：

- 命中顺序由 shared retained surface 决定
- 是否触发激活/关闭/折叠由 shared route 决定
- typed payload 仍由 builtin template binding 决定
- Slint 只保留视觉、drag suppression 和局部坐标上传职责

### Asset surface semantic cutover

这一轮继续把 `assets.slint` 的语义入口也收到了 builtin template authority 下，但仍然不去改现有 Slint 视觉树。

新增的 builtin 文档是：

- [`asset_surface_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml)

它固定了稳定命名空间：

- `AssetSurface/SelectFolder`
- `AssetSurface/SelectItem`
- `AssetSurface/SearchEdited`
- `AssetSurface/SetKindFilter`
- `AssetSurface/SetViewMode`
- `AssetSurface/SetUtilityTab`
- `AssetSurface/ActivateReference`
- `AssetSurface/OpenAssetBrowser`
- `AssetSurface/LocateSelectedAsset`

配套改动是：

- [`zircon_editor/src/ui/binding/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/binding/mod.rs) 把 `AssetCommand` 扩成 typed surface command 集合
- [`binding_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/binding_dispatch/mod.rs) 通过 `asset/` 子模块集中解析 `surface/view_mode/utility_tab`
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 新增 `BuiltinAssetSurfaceTemplateBridge`
- [`app/assets.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/assets.rs) 现在只保留 `control_id -> UiEventKind + template arguments` 的翻译，而不再保留 `update_asset_search(...)` / `open_asset_browser()` 这类业务 helper
- [`app/callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 的 asset 宿主注册现在只剩 `pane_surface_host.on_asset_control_changed(...)` 和 `pane_surface_host.on_asset_control_clicked(...)`，asset generic callback 不再回挂 root `UiHostWindow`

因此现在真实宿主链路已经从：

- Slint callback
  -> `app.rs` 本地字符串解析
  -> 直接改 asset workspace / layout

变成：

- Slint callback
  -> builtin asset surface projection
  -> stable `AssetSurface/*`
  -> typed `AssetCommand`
  -> `EditorEventRuntime`

但现有 asset 视觉壳体已经进一步按 owner 拆开：[`asset_panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/asset_panes.slint) 现在承接 Project/Assets/AssetBrowser 三块顶层 pane，而 [`assets.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint) 只再保留复用 leaf surface 和 DTO；这一刀仍然只替换语义/目录边界，不改视觉行为本身。

这一轮又把 asset surface header / utility / import 这层的 ABI 继续收紧成“稳定 control id，而不是业务 callback 名称”：

- [`asset_panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/asset_panes.slint) 的顶层 asset pane 现在统一只暴露：
  - `control_changed(control_id, value)`
  - `control_clicked(control_id)`
- [`ui_asset_editor_pane.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_pane.slint) 现在独立承接 `UiAssetEditorPane + UiAssetEditorPaneData + palette/source/style/binding authoring DTO`，[`pane_fields.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_fields.slint) 则把 `CompactField/AxisField` 从旧的巨型 `panes.slint` 里抽成共享字段件 owner；[`panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/panes.slint) 因而只再保留 generic pane surface
- `SearchEdited` / `SetKindFilter` / `SetViewMode` / `SetUtilityTab` / `OpenAssetBrowser` / `LocateSelectedAsset` / `ImportModel` 都通过同一套 generic control route 上传
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 顶层只保留：
  - `asset_control_changed(source, control_id, value)`
  - `asset_control_clicked(source, control_id)`
- `ProjectOverviewPane` 的 “Asset Browser” 按钮现在也复用 `asset_control_clicked("project", "OpenAssetBrowser")`，而不是保留单独 `open_asset_browser()` ABI
- `mesh_import_path_edited(...)` 仍然保留在 draft/binding 主链上，因为它还没有进入 `asset_surface_controls.ui.toml` 这份 builtin template 文档

因此 asset 宿主的非 pointer 语义边界现在也开始和 menu / welcome 一样收口成：

- Slint 只上传稳定 `control_id`
- host 只负责把 `control_id` 翻译成 template arguments
- typed payload 继续由 builtin template binding 和 `EditorEventRuntime` 权威决定

这一轮又把 asset surface 的非菜单列表输入继续往 shared core 收口了一刀，而且不再只停在 folder tree / content list：

- [`asset_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/asset_pointer.rs) 现在除了 `AssetFolderTreePointerBridge` 和 `AssetContentListPointerBridge` 之外，还新增 `AssetReferenceListPointerBridge`
- `AssetSurfacePointerState` 现在同时持有：
  - folder tree shared pointer state
  - content list shared pointer state
  - `references` shared pointer state
  - `used_by` shared pointer state
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 新增 `dispatch_shared_asset_reference_pointer_click(...)`
- reference list click 不再由 Slint row `TouchArea` 直接触发 `activate_reference(uuid)`，而是走：
  坐标
  -> shared `UiSurface`
  -> shared `UiPointerDispatcher`
  -> `AssetPointerReferenceRoute`
  -> stable `AssetSurface/ActivateReference`
  -> typed `AssetCommand::ActivateReference`
  -> `EditorEventRuntime`

对应地，[`assets.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint) 里的 `ReferenceListView` 现在也不再把 `ScrollView` 和 per-row `TouchArea` 当成事件真源，而是改成和 welcome recent list、hierarchy list、asset tree/content 一样的模式：

- Slint 只消费 host 投影下来的 `hovered_index` / `scroll_px`
- Slint 只上报 viewport-local pointer 坐标和滚轮 delta
- activity/browser 两套 asset surface 的 `references` / `used_by` 列都复用同一套 shared pointer dispatcher 语义

因此 asset 面板当前已经形成一条更完整的 shared-core-first 输入链：

- folder tree
- content list
- references list
- used-by list

四类列表都不再以手写 Slint row 命中作为权威。

### Scroll-only pane surfaces now stay host-authoritative

这一轮继续把不需要 editor-only row route、但仍然需要 shared wheel/scroll authority 的 pane surface 往同一条主链上收口：

- [`detail_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/detail_pointer/mod.rs) 现在不再只服务 console 和 asset details rail，还新增了 inspector 对应的 shared scroll layout helper；根入口只保留结构导出，`ScrollSurfacePointerBridge` 声明、layout helper、route/dispatch/state、scroll 行为实现已经拆到独立脚本
- `ScrollSurfacePointerBridge` 继续作为统一的 scroll-only 宿主桥：
  - host 同步 `pane_size + content_extent`
  - bridge 在 shared `UiSurface` 里建立单一 `ScrollableBox`
  - wheel 事件先经过 `UiPointerDispatcher`
  - 最终只把 `scroll_offset` 回灌给 Slint 的 `scroll_px`
- 宿主侧的 `bridge + state + size` 也已经从 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 抽到 [`scroll_surface_host.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/scroll_surface_host.rs)，后续 popup/dialog-like overlay 不需要再复制同一套状态样板代码
- [`panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/panes.slint) 里的 `InspectorPane` 已经不再把 Slint `ScrollView` 当成滚动真源，而是改成和 hierarchy/tree/list 一样的“clip + host-driven content offset”模式
- 这条改法刻意保留了 inspector 里的 `CompactField`、`AxisField` 和 `ShellButton` 现有 callback 语义；shared core 只接管 wheel / scroll state，不重新定义 inspector draft/apply/delete 协议

因此当前进入 shared scroll authority 的非-menu pane surface 已经包括：

- `ConsolePane`
- `SelectionDetailsRail`
- `InspectorPane`

### Welcome surface semantic cutover

这轮继续把 startup/welcome 的 callback 语义入口也迁到了 builtin template authority 上，但刻意把切口停在“typed host event”，没有把整个 startup session 生命周期强行推进 `EditorEventRuntime`。

新增的 builtin 文档是：

- [`startup_welcome_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml)

它定义了稳定命名空间：

- `WelcomeSurface/ProjectNameEdited`
- `WelcomeSurface/LocationEdited`
- `WelcomeSurface/CreateProject`
- `WelcomeSurface/OpenExistingProject`
- `WelcomeSurface/OpenRecentProject`
- `WelcomeSurface/RemoveRecentProject`

对应的 typed 层现在拆成两段：

- [`zircon_editor/src/ui/binding/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/binding/mod.rs) 新增 `WelcomeCommand`
- [`binding_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/binding_dispatch/mod.rs) 通过 `welcome/` 子模块新增 `WelcomeHostEvent`

真实宿主里：

- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 新增 `BuiltinWelcomeSurfaceTemplateBridge`
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 的 `on_welcome_*` 回调不再直接把 callback 名称当语义，而是先解析稳定 binding，再执行已有的 startup session 逻辑

当前刻意保留的宿主 seam 是：

- `EditorStartupSessionDocument`
- recent project 验证与移除
- `create_project_and_open / open_project_and_remember`
- exclusive welcome page 的 show/dismiss 生命周期

也就是说，welcome 现在已经不再是“本地回调名就是协议”，但它还没有被错误地伪装成 runtime 已经完全接管。模板 authority 负责 control 语义，startup session 仍然由宿主持有。

### Legacy List ABI Retirement

这一轮又补了一层 authority hygiene，把 list surface 上最后一批直连 Slint callback ABI 彻底切掉了，而不是只满足“shared pointer 已经能工作”：

- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint)、[`assets.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint)、[`panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/panes.slint)、[`welcome.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/welcome.slint) 不再声明或转发：
  - `hierarchy_select`
  - `asset_select_folder`
  - `asset_select_item`
  - `asset_activate_reference`
  - `welcome_open_recent_project`
  - `welcome_remove_recent_project`
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 也同步移除了对应 `ui.on_*` 注册和 helper 方法，不再保留“shared pointer route 之外还可以直接打进 runtime”的第二条旁路
- list surface 现在只保留 viewport-local `pointer_clicked / moved / scrolled` 上报，以及 host 回灌的 `hovered_index / scroll_px`

这意味着 hierarchy、asset tree/content/reference、welcome recent projects 这些列表面已经统一遵守同一条主链：

坐标/滚轮
-> shared `UiSurface`
-> shared `UiPointerDispatcher`
-> stable binding / route id
-> typed `EditorEventRuntime`

对应回归由 [`surface_contract.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_list_pointer/surface_contract.rs) 的 `shared_list_surfaces_do_not_expose_legacy_direct_callback_routes` 锁定，避免后续 cutover 时把旧 callback 壳层悄悄接回来。

welcome recent real-host fallback 又把这条主链往前推进了一步：即使 `.slint` 回调暂时没有带回有效 pane 尺寸，shared template projection 仍然能把 `PaneSurfaceRoot` frame 作为 pointer 主链的几何真源，避免重新引入宿主本地尺寸猜测。

### Inspector drafts and asset import cutover

这一轮又补上了一条之前还留在宿主直写状态里的兼容缺口：inspector 实时草稿编辑和 asset pane 的 mesh import path 编辑，不再直接在 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 里调用 `runtime.update_*` / `runtime.set_mesh_import_path(...)`。

现在它们统一变成：

- [`zircon_editor/src/ui/binding/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/binding/mod.rs) 新增 `DraftCommand`
  - `DraftCommand::SetInspectorField { subject_path, field_id, value }`
  - `DraftCommand::SetMeshImportPath { value }`
- [`zircon_editor/src/core/editor_event/types.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/editor_event/types.rs) 新增 `EditorDraftEvent`
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 新增：
  - `dispatch_inspector_draft_field(...)`
  - `dispatch_mesh_import_path_edit(...)`
- [`runtime.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/editor_event/runtime.rs) 把这些 typed binding 归一化成 canonical `EditorEvent::Draft(...)`

因此当前真实主链已经变成：

- Slint callback
  -> typed `DraftCommand`
  -> `EditorEvent::Draft(EditorDraftEvent::...)`
  -> runtime 更新 live snapshot
  -> `SlintDispatchEffects::presentation_dirty`

这一步的约束非常明确：

- inspector 的实时输入仍然只是草稿态，不会提前触发 `ApplyInspectorChanges`
- mesh import path 的实时输入仍然只是 presentation/runtime 草稿，不会提前触发模型导入或 asset sync
- `Import Model` 提交动作现在已经进入 typed asset command authority：
  - `AssetCommand::ImportModel`
  - `EditorEvent::Asset(EditorAssetEvent::ImportModel)`
  - `EditorEventEffect::ImportModelRequested`
- 真正的文件复制、asset import、resource resolve 和 scene mesh 注入仍然保留在宿主 `import_model_into_project()`；本轮 cutover 的目标是把“控件语义解释权”移出 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs)，不是把文件系统 side effect 伪装成纯 runtime 逻辑

同一轮里，reflection/action surface 也同步补齐了远控入口，而不是只改桌面 callback：

- inspector activity 现在额外暴露 `edit_field`
- assets activity 现在额外暴露 `set_mesh_import_path`
- assets activity 现在额外暴露 `import_model`

因此 live draft 语义现在已经同时进入：

- desktop Slint callback
- typed binding codec
- runtime journal/event normalization
- reflection `CallAction`
- runtime host-effect request for `ImportModel`

### Shell Pointer Route Normalization

这一轮继续把 editor shell 的 pointer/drop 路由往 shared core 权威收口，但仍然保持 cutover 的范围足够小，不去重写高冲突的 startup/welcome 状态机。

新增的收口点最初有三层：

- `WorkbenchShellPointerRoute` 现在不只表示 splitter resize，还显式带上 `DragTarget(...)`
- [`tab_drag.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag.rs) 新增 `ResolvedWorkbenchTabDropRoute`
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 新增 `dispatch_tab_drop(...)`

这三层让真实宿主中的 tab drop 主链从原先的：

- Slint callback 提供 `target_group` 字符串
- `app.rs` 在局部手写逻辑里混合 pointer hit-test、fallback 和 attach/reopen dispatch

变成现在的：

- `WorkbenchShellPointerBridge` 在 shared `UiSurface` 上给出 `WorkbenchShellPointerRoute::DragTarget`
- `resolve_workbench_tab_drop_route(...)` 用 shared pointer route 先决，再保留旧 `target_group` 作为兼容 fallback
- `dispatch_tab_drop(...)` 统一把 normalized drop route 落成 typed `LayoutCommand::AttachView`
- 如果目标是 collapsed drawer，再由统一 dispatcher 追加 `SetDrawerMode::Pinned`

这一步的收益不是单纯删掉几行字符串判断，而是把真实宿主里的 dock drop 流程也推到了与 menu、activity rail、document tabs 相同的“先拿 shared/template/shared-surface authority，再做 typed dispatch”的方向上。

这一轮又补了一条 route dispatcher 的兼容约束：attach 到 drawer 不再无条件补发 `SetDrawerMode::Pinned`。现在只有目标 drawer 处于 `Collapsed` 时才会 reopen；已经 `Pinned` 或 `AutoHide` 的 drawer 会保留原模式，只记录 `AttachView`。这样 normalized drop route 和旧宿主语义在 journal 上重新对齐，不会因为 shared route cutover 多出一条无意义的 layout event。

更重要的是，当前还保留了迁移期兼容面：

- shared pointer route 缺失时，仍允许回退到旧的 `target_group` 字符串
- 精确 tab anchor 解析仍然复用已有 `resolve_tab_drop(...)`
- `app.rs` 不再直接决定 attach/reopen 次序，只负责拿 route 和设置状态提示

因此当前切片没有把 editor shell 的拖放行为重新定义一遍，而是把“谁是 drop route 真源”重新钉到了 shared shell pointer bridge 上。

### Document edge route host cutover

在这一轮之后，这条链又往前推了一刀，document edge 不再只是 shared pointer 内部语义，而是已经进入真实 Slint 宿主可见的 drag feedback 与 fallback key：

- `WorkbenchShellPointerRoute` 现在显式区分：
  - `DragTarget(...)`
  - `DocumentEdge(DockEdge)`
  - `Resize(...)`
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 的 `update_drag_target(...)` 不再把所有 document edge 都压扁成 `"document"`，而是直接把 shared route 投成稳定宿主 key：
  - `document-left`
  - `document-right`
  - `document-top`
  - `document-bottom`
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 的 drag overlay 现在直接消费这些 key，并把 badge / edge highlight 与 shared route 对齐，而不是继续只显示粗粒度的 `"Dock Document"`
- [`tab_drag.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag.rs) 现在还能把这些 key 当作兼容 fallback 重新解释成 `ResolvedWorkbenchTabDropTarget::Split`，因此即便 pointer route 在 drop 时缺失，宿主仍然会保留 `CreateSplit` 语义，而不是回退到普通 attach

这意味着 document split 的 cutover 已经不再只有“dispatcher 内部能分辨 edge”，而是已经形成一条完整的 shared-core-first 主链：

- shared pointer hit-test 决定 edge
- host overlay 显示与该 edge 一致的反馈
- normalized route 落到 `CreateSplit`
- 宿主 fallback key 与 shared route 保持同一套命名，而不是再发明一套 Slint-only 字符串

### Floating window route normalization and view-model exposure

这一刀继续往 shared authority 链路推进，但刻意只补“projection + route contract”，没有直接跳去重写浮窗宿主视觉。

[`WorkbenchViewModel`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/model/mod.rs) 现在新增 `floating_windows`，并把浮窗里的 tab 也收敛成和主 document 一致的 workspace-tab 术语：

- `FloatingWindowModel.window_id`
- `FloatingWindowModel.title`
- `FloatingWindowModel.focused_view`
- `FloatingWindowModel.tabs`

与此同时，`DocumentTabModel` 不再把 host 固定死成 main page，而是显式携带 `WorkspaceTarget`。这让主 document 和 floating workspace 终于可以共用同一套：

- `workspace`
- `workspace_path`
- `instance_id`
- active/closeable/empty-state 投影

在 route 这一层，又补上了两类稳定 fallback key：

- `floating-window/<window_id>`
- `floating-window-edge/<window_id>/<edge>`

[`tab_drag.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag.rs) 现在会把它们统一归一成 typed shared route：

- attach 到 `ViewHost::FloatingWindow(window_id, path)`
- split 到 `WorkspaceTarget::FloatingWindow(window_id)`

其中 `path` 会优先跟随 layout 的 `focused_view`，否则回退到浮窗 workspace 内第一个 active/non-empty tab stack。这样 route 归一层和 workspace 语义先独立定型，之后再把真实 shared pointer surface 命中补到同一条主链上：

- 浮窗 attach
- 浮窗 edge split
- 与 `WorkbenchLayout` / `FloatingWindowSnapshot` 保持一致的 workspace 语义

这一步之后，真实 `slint_host` 也已经开始消费这份投影：

- [`ui.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui.rs) 现在会把 `WorkbenchViewModel.floating_windows` 投成 `FloatingWindowData`
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 新增最小浮窗 overlay host，会真正渲染：
  - 浮窗标题
  - 浮窗 tab strip
  - 复用 `PaneSurface` 的活动 pane 内容
- 浮窗 tab 已经可以复用现有 shared document-tab pointer 桥：
  - 激活 route
  - 关闭 route
  - 拖出到现有 dock/document 目标组

也就是说，当前已经不是“只有模型和路由，没有宿主消费”，而是“已有过渡性的真实宿主 overlay，并且浮窗几何 authority 与 pointer route authority 都已经开始走 shared core”。

这一轮又把浮窗 overlay 的 header click 收口成了一条明确的 runtime dispatch 主链：

- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 的浮窗 card 只暴露 `floating_window_header_pointer_clicked(...)`
- [`callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 只接这条 pointer fact
- [`app/workbench_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/workbench_pointer.rs) 先经由 [`shell_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/shell_pointer.rs) 解析 shared `FloatingWindow/FloatingWindowEdge` route
- [`callback_dispatch/layout/floating_window/dispatch.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs) 再把 route 收口为 typed `LayoutCommand::FocusView`
- [`FloatingWindowModel`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/model/mod.rs) 现在提供唯一的 `focus_target_instance()` / `focus_target_tab()` helper，overlay projection 和焦点 dispatch 不再各自维护一份 fallback 逻辑
- focus fallback 顺序已经固定并由测试锁住：存在于该浮窗 tab 集中的 `focused_view -> active tab -> first tab`

这也把当前 shell 的 non-menu transient overlay inventory 盘清了：

- menu popup 仍然是唯一具备 dismiss overlay 语义的 transient surface
- menu 之外，当前 workbench shell 没有第二个 standalone popup/dialog/modal dismiss surface
- floating-window header 是当前唯一活着的 non-menu transient overlay 命中层，但它属于 persistent workspace host，而不是 dismissible popup/dialog

这一刀之后，这里的 authority 边界已经继续下沉了两层：

- [`layout/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/layout/mod.rs) 的 `FloatingWindowLayout` 现在显式持久化 `frame: ShellFrame`
- [`autolayout/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/autolayout/mod.rs) 的 `WorkbenchShellGeometry` 现在生成 `floating_window_frames`
- 未持久化 frame 的旧 layout 会由 shared geometry 层按中心工作区生成默认浮窗 frame，再统一 clamp 到 `center_band_frame`
- [`ui.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui.rs) 会把这些 frame 和稳定 route key 一起投进 `FloatingWindowData`
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 不再自己决定浮窗位置，只消费 `window.frame.*`

同时，浮窗自身的 shared pointer route 也已经接进真实主链，而不是只剩 fallback key：

- [`shell_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/shell_pointer.rs) 的 drag surface 现在会直接产出：
  - `WorkbenchShellPointerRoute::FloatingWindow(window_id)`
  - `WorkbenchShellPointerRoute::FloatingWindowEdge { window_id, edge }`
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 的 `update_drag_target(...)` 会把这些 route 归一成稳定宿主 key：
  - `floating-window/<window_id>`
  - `floating-window-edge/<window_id>/<edge>`
- [`tab_drag.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag.rs) 会优先消费 shared route，只在 route 缺失时再回退到同名 fallback key

这一刀还顺手修正了一个 shared-core 命中边界：

- document edge split 热区现在只落在真实 `Document` region frame 上
- empty bottom/right dock band 不再被 `document-bottom` / `document-right` split 路由吞掉
- 因此 shared dock target route 与旧 workbench drag-target 契约重新对齐

这一刀仍然故意没有完成的是：

- 真正的多宿主/多原生窗口 cutover
- detach/promote 之外更完整的 window lifecycle / focus / capture 语义
- 让通用 Slint host 组件树直接消费更多 editor shell frame/clip/z-order，而不是继续保留一部分手写 overlay 结构

也就是说，当前完成的已经不只是“模型、typed 路由、以及最小宿主可视消费”，而是“浮窗 frame authority + 浮窗 pointer route authority + 最小宿主消费”都已经接到了 shared contract；还没有结束的是更完整的窗口化宿主 cutover。

### Viewport toolbar callback cutover

这一轮继续沿着“保留 Slint 壳，但把语义权威前移到模板/runtime”推进，不过 cutover 的对象换成了 Scene viewport toolbar。

新增的模板入口是：

- [`scene_viewport_toolbar.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml)

它现在定义了一组稳定 control id 和 binding id：

- `SetTool`
- `SetTransformSpace`
- `SetProjectionMode`
- `AlignView`
- `SetDisplayMode`
- `SetGridMode`
- `SetTranslateSnap`
- `SetRotateSnapDegrees`
- `SetScaleSnap`
- `SetPreviewLighting`
- `SetPreviewSkybox`
- `SetGizmosEnabled`
- `FrameSelection`

对应的稳定 binding 命名空间固定为：

- `ViewportToolbar/*`

真实 cutover 发生在宿主侧：

- [`runtime_host.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs) 现在只保留宿主 façade 与运行时状态；builtin 模板注册在 [`build_session.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/runtime/build_session.rs) 与 [`template_bindings.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs)，projection 逻辑单独放进 [`projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/runtime/projection.rs)
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 新增 `BuiltinViewportToolbarTemplateBridge`
- [`viewport_toolbar_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs) 现在持有 `ViewportToolbarPointerBridge`，用最小 `UiSurface + UiPointerDispatcher` 统一解析 toolbar control hit-test
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 里的 viewport toolbar callback 不再本地手写字符串解析 `tool/space/projection/display/grid`，只上传 `surface_key + control_id + local frame + pointer point`

这让当前主链从原先的：

- Slint callback 上传字符串
- `app.rs` 本地 `parse_*`
- 再拼成 `EditorViewportEvent`

变成现在的：

- Slint callback 上传稳定 `control_id + event_kind + arguments`
- `BuiltinViewportToolbarTemplateBridge` 先从 builtin template projection 找到 `ViewportToolbar/*`
- `ViewportToolbarPointerBridge` 先用 shared route 确认实际命中的 control
- typed binding 再落到 `EditorEventRuntime`

这一步刻意只替换“命令解释权”，不替换现有视觉树：

- `workbench.slint` 仍然渲染 Scene viewport toolbar
- host 仍然保留当前 Scene pane 的 Slint 布局与样式
- 但 toolbar 命中和命令语义都已经不再以 `app.rs` 的局部字符串 parser 或 `.slint` 直连 callback ABI 为权威

因此当前 editor UI 不是“全部替换完成”，而是：

- viewport toolbar 的命令语义已经进入模板 binding 权威链
- viewport toolbar 的 pointer hit-test 已经进入 shared `UiSurface + UiPointerDispatcher`
- viewport toolbar 的视觉结构仍然属于迁移中的 Slint compatibility shell
- `workbench.slint` 已移除 `viewport_set_*` / `viewport_frame_selection` 直连 callback 声明和转发，只保留 `viewport_toolbar_pointer_clicked(...)` 这一条 pointer-fact 宿主边界

这一层目前有两类回归保护：

- [`template_runtime/viewport_toolbar.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/template_runtime/viewport_toolbar.rs) 验证 builtin projection 确实暴露 `ViewportToolbar/*`
- [`slint_viewport_toolbar_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs) 锁定 shared pointer bridge、runtime dispatcher，以及 `.slint` 不再暴露 legacy direct callback ABI
- [`slint_callback_dispatch/viewport/toolbar_dispatch.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_callback_dispatch/viewport/toolbar_dispatch.rs) 验证 `SetTool` 和 `FrameSelection` 已经通过模板桥接落成 typed `EditorViewportEvent`

### Pane surface action callback cutover

这一轮又把还留在兼容壳里的一个显式逃逸口补掉了：pane empty-state 和 project overview 里的“动作按钮”不再从 [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 直接抛 `menu_action(action_id)` 给宿主。

新的权威链路是：

- [`pane_surface_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml) 定义 builtin `PaneSurface/TriggerAction`
- [`callback_dispatch/pane/surface_control.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/pane/surface_control.rs) 通过 `BuiltinPaneSurfaceTemplateBridge` 把 `control_id + action_id` 归一化成 typed `MenuAction`
- [`pane_surface_actions.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs) 成为真实宿主入口
- [`callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 不再注册 `ui.on_menu_action(...)`

对应的 Slint 兼容壳也一起收口：

- `PaneSurface` 内部不再声明 `empty_action(...)`
- workbench root 不再声明 `menu_action(...)`
- scene/game empty-state 的主/次按钮，以及 `ProjectOverviewPane` 的 `Open Assets`，现在统一只上传 `pane_surface_control_clicked(control_id, action_id)` 这一条 generic control seam

这一步的价值不是“再多一层包装”，而是避免 transient shell surface 在 template/runtime 主链之外保留第二条直接字符串业务 ABI。现在这些条件性 surface 和 inspector / welcome / asset surface 一样，已经回到 builtin template binding authority。

同一轮里，viewport toolbar 的 authority hygiene 也继续收紧了一层：

- `SceneViewportToolbar` 组件本身已经删掉残留的 `set_tool` / `set_projection_mode` / `frame_selection` 等 legacy callback 壳声明
- [`slint_viewport_toolbar_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs) 现在把这些声明也视为 ABI 回退

### `EditorUiCompatibilityHarness`

当前 harness 不再只看模板 projection 摘要，还能直接捕获 host model 的稳定快照。

projection 侧仍然记录：

- `components`
- `control_ids`
- `binding_ids`

host model 侧新增记录：

- `host_nodes`
- `route_bindings`
- `frame_entries`
- `attribute_entries`
- `style_token_entries`
- `route_key_entries`

它还不是最终的“旧 Slint 壳 vs 新模板宿主”的完整 parity harness，但已经开始覆盖：

- 哪个宿主节点暴露了哪个 `control_id`
- 哪个 binding 在注册后拿到了哪个 `route_id`
- 哪些 icon/text/token 属性真正落到了宿主节点模型上

这让后续 parity 对照不再只比较“树里有没有这些节点”，而是能比较“宿主真正能消费到的节点与路由数据是否一致”。

这一轮又开始把这份 parity 从“静态 projection/snapshot”推进到“同 fixture 的 runtime event log”。当前已经直接锁住三组 shared/template vs legacy/raw 的等价性：

- `WorkbenchMenuBar/ResetLayout` vs `dispatch_menu_action("ResetLayout")`
- `ViewportToolbar/SetTool` vs `dispatch_viewport_command(ViewportCommand::SetTool(...))`
- `AssetSurface/SearchEdited` vs `dispatch_asset_search(...)`

这些测试现在直接比较 `EditorEventRecord` 和 `SlintDispatchEffects`，而不只比较 binding id、route id 或 frame 摘要。因此 template runtime 一旦把某个 control 误归类成另一种 payload family，就会在 same-fixture parity 上立即暴露出来。

这轮 parity 又往前补了两类入口：

- `WorkbenchShell/ActivateMainPage` template dispatch vs 直接 `LayoutCommand::ActivateMainPage`
- `WelcomeSurface/ProjectNameEdited`、`WelcomeSurface/OpenRecentProject` template dispatch vs 直接 `dispatch_welcome_binding(...)`

同时，dock route dispatcher 现在也有 focused journal parity 保护：

- attach 到 collapsed drawer 仍会产生 `AttachView + SetDrawerMode(Pinned)`
- attach 到已经可见的 drawer 只会产生 `AttachView`
- attach 到 `AutoHide` drawer 会保留 `AutoHide` 模式，不会被 template/shared route cutover 意外改写成 `Pinned`

这一轮又补了一组 Slint host projection 侧的摘要：

- `slint_nodes`
- `frame_entries`
- `text_entries`
- `icon_entries`

因此 parity harness 现在已经能分别对比：

- 模板实例树摘要
- host-neutral 节点摘要
- Slint 可绑定节点摘要
- legacy floating overlay 摘要

虽然还没有真正拿旧手写壳体做双边对照，但对照数据面已经开始成形。

与此同时，harness 现在还能抓 shared surface 摘要，记录：

- `surface_nodes`
- `frame_entries`
- surface 节点上的 `binding_ids`
- surface 节点上的 `attribute_entries`
- surface 节点上的 `style_token_entries`

这让兼容对照的中间层变成了三段：

1. shared surface
2. host-neutral node model
3. Slint-consumable host projection

后续如果某个行为在 shared core 和 Slint host 之间发生了漂移，至少已经能知道偏差是出在：

- 模板到 shared tree
- shared tree 到 host-neutral model
- host-neutral model 到 Slint host projection

而不是再像旧手写壳那样把所有差异都堆在一个 callback 或属性投影层里。

这一轮又把 legacy shell 浮窗 overlay 正式纳入 harness：

- [`EditorUiCompatibilityHarness`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/harness.rs) 新增 `capture_floating_window_overlay_snapshot(...)`
- 同一份 harness 现在又新增：
  - `capture_event_journal_delta_snapshot(...)`
  - `capture_resolved_tab_drop_route_snapshot(...)`
- snapshot 现在会记录：
  - `floating-window/<window_id>=x,y,w,h`
  - `floating-window/<window_id>.attach=<stable-route-key>`
  - `floating-window/<window_id>.<edge>=<stable-edge-route-key>`
  - `floating-window/<window_id>.focus_target_id=<stable-view-instance-id>`
  - `floating-window/<window_id>.active_pane.*`

这不是把 floating window 直接模板化了，而是先把“旧 Slint 壳当前真实消费到的 shared frame / route key”固定成可比较的 parity 数据。
这样后续即使继续推进 multi-window host cutover，也不会再退回到“只有视觉看起来差不多，但不知道 route key 有没有漂”的状态。
现在又进一步把“浮窗 header click 最终应该 focus 到哪个 view”也固定成 parity 数据，因此 route key、active pane 和 runtime focus dispatch 可以在同一 fixture 上一起对照，而不是各自漂移。
同一轮里，dock/drop route normalization 也不再只靠结构体 `assert_eq!` 零散覆盖：

- [`document_routes.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_tab_drag/document_routes.rs) 和 [`floating_routes.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_tab_drag/floating_routes.rs) 现在会把 shared pointer route 和 fallback group key 各自解析出的结果都压成 `route_result_entries`
- 文档边缘 split 和 floating-window attach 都已经有 same-fixture parity，对照 shared pointer authority 与 legacy host group key 是否仍然归一到同一条 normalized route
- `slint_callback_dispatch/layout.rs` 则把 floating-window focus 的 builtin dispatch journal delta 和直接 `LayoutCommand::FocusView` journal delta 并排对照，防止后续 route/event cutover 把 focus target、effect 或 undo 语义悄悄漂移

同一轮里，[`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 的 `FloatingWindowData` 也新增 `focus_target_id`。它不是新的业务 ABI，只是把 shared projection 的 canonical focus target 显式带到宿主壳里，方便 parity 和后续 multi-window host cutover 复用。

与之配套，[`tab_drag.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag.rs) 现在抽出了 `workbench_shell_pointer_route_group_key(...)`，把：

- `DragTarget(...)`
- `DocumentEdge(...)`
- `FloatingWindow(...)`
- `FloatingWindowEdge { .. }`

统一归一成宿主稳定 key。  
[`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 和新 parity 测试现在共用这一份 helper，不再各自手写一套 route -> string 映射。

## Shared Menu Pointer Authority

这一轮又把 editor shell 的 menu / popup / scroll pointer 路由推进到了和 dock / drag / resize 同一层的 shared authority：

- [`menu_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/menu_pointer/mod.rs) 现在持有 `HostMenuPointerBridge`，内部维护独立 `UiSurface + UiPointerDispatcher`
- bridge surface 会显式建出：
  - top menu button 节点
  - popup dismiss overlay 节点
  - popup surface 节点
  - popup item 节点
- `Window` 菜单的 popup surface 不再只是一块 Slint 裁剪区域，而是 shared `ScrollableBox + UiScrollState` 节点；滚轮会先更新 shared scroll state，再回灌 host 的 `window_menu_scroll_px`
- dismiss overlay route 现在只收起 popup，不再在 close path 上清空 `popup_scroll_offset`；shared scroll state 只在下一次 `open_popup(...)` 时重置，这样 template/runtime parity 和真实 host dismiss/reopen 都继续共用同一份 canonical state
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 和 [`chrome.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/chrome.slint) 里的 menu button / item 现在只是表现层，真实命中和 hover/open state 都由 host 下发
- [`build_host_menu_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs) 现在把 builtin `root_shell_frames()` 里的 `WorkbenchMenuBarRoot` / `WorkbenchShellRoot` frame 继续投影成兼容壳仍在使用的六段 top menu button frame；这一步保证 legacy Slint menu 壳还没完全模板化之前，top-level menu strip 也优先吃 shared root projection，而不是继续把 `get_*_menu_button_frame()` 当真源
- [`app/pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 会把这组 frame 反向设置回 `WorkbenchShell` 的 `*_menu_button_frame`；[`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 的 popup anchor 已切到这些 host-projected property，因此 menu hit-test 与 popup presentation 在兼容壳里重新共享同一条 authority 链
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 新增 `dispatch_shared_menu_pointer_click(...)`，把：
  Slint 坐标
  -> shared menu hit-test
  -> shared pointer route
  -> template-aware menu dispatch fallback
  -> `EditorEventRuntime`
  收成一条可测试的统一链路
- 这条兼容 cutover 的 focused regressions 现在包括：
  - [`slint_menu_pointer/layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_menu_pointer/layout.rs): `shared_menu_pointer_layout_prefers_shared_root_menu_bar_projection_over_stale_legacy_frames`
  - [`slint_menu_pointer/surface_contract.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs): `shared_menu_popup_presentation_drops_host_menu_button_frame_setters`
  - [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs): `root_menu_popup_scroll_and_dismiss_flow_through_shared_pointer_bridge_in_real_host`

这意味着 menu/popup 不再只是“shared core 负责命中，`app.rs` 再自己猜 action”的半迁移状态。现在 pointer click 已经能在 shared route 返回后直接进入 runtime dispatcher，并对静态按钮和动态 preset menu item 走同一条入口。

## Viewport Overlay Pointer Authority

viewport 这条链路现在也不再停留在“外框进入 shared bridge，但 gizmo/handle/renderable 仍由本地 picking cache 判断”的中间态：

- outer viewport surface 仍然先经过 [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 里的 `SharedViewportPointerBridge`，统一处理 frame bounds、pointer capture 和 `ViewportCommand` 映射
- 进入 editor runtime 之后，[`SceneViewportController`](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/controller/mod.rs) 现在会把 camera、viewport、handle overlay、scene gizmo 和 renderable 候选同步进 [`ViewportOverlayPointerRouter`](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/pointer/mod.rs)
- overlay router 内部会建立一棵最小 `UiSurface`，把 coarse candidate frame、z-order 和 `UiPointerDispatcher` 放进 shared core，再统一产出 `ViewportPointerRoute::{HandleAxis, SceneGizmo, Renderable}`
- route 优先级固定为 `HandleAxis > SceneGizmo > Renderable`，并由 [`zircon_editor/src/tests/editing/viewport.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/editing/viewport.rs) 锁住
- 旧的 `picking.rs` 已经退出主链，因此 Slint host 现在只负责窗口/纹理/pointer 坐标上传；viewport overlay 命中语义属于 shared surface + editor runtime

这让模板迁移路径不再只覆盖菜单、dock 和列表输入。即便 Scene viewport 还保留现有 Slint 壳和 viewport texture host，它的 overlay hit-test 真源也已经开始遵守 shared contract，而不是继续藏在一份 editor-local pick helper 里。

## Drag/Resize Capture Pointer-Fact Cutover

这一轮把 editor shell 最后那段仍然依赖 handwritten Slint callback ABI 的 drag/resize capture 也收到了 shared contract 下：

- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 不再声明或转发 `drop_tab(...)`、`update_drag_target(...)`、`begin_drawer_resize(...)`、`update_drawer_resize(...)`、`finish_drawer_resize(...)`
- drag overlay 和 splitter/full-screen resize capture 现在统一只上传两类 pointer fact：
  - `workbench_drag_pointer_event(kind, x, y)`
  - `workbench_resize_pointer_event(kind, x, y)`
- [`callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 现在只注册这两条 shared pointer 入口，不再把 drag/drop 或 resize lifecycle 拆成五个业务回调
- [`workspace_docking.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/workspace_docking.rs) 现在把这些 pointer fact 还原成统一宿主流程：
  - drag move/up 先经过 `WorkbenchShellPointerBridge::drag_route_at(...)`
  - resize down/move/up 先经过 `WorkbenchShellPointerBridge::{begin,update,finish}_resize(...)`
  - drop/resize 的 typed runtime dispatch 仍分别走 `resolve_workbench_tab_drop_route(...) + dispatch_tab_drop(...)` 与 `dispatch_resize_to_group(...)`

这一步的关键不是“把几个 callback 合并成两个名字”，而是把 editor shell capture 期间的真实 authority 从 Slint business ABI 退回到 shared `UiSurface + UiPointerDispatcher`：

- Slint 只保留局部拖拽开始、视觉状态和 pointer 坐标上传
- shared shell surface 负责 route 解析、capture 延续和 group key 归一
- runtime dispatcher 继续负责 typed `LayoutCommand` / `DockCommand` 落地

这样一来，drag/drop、splitter resize、document edge split、floating attach/edge 和未来 popup/dialog capture 都可以沿用同一条 host seam，而不是在 `workbench.slint` 里重新扩张 callback ABI。

## ScrollView Authority Cleanup

这一轮又补掉了之前仍然留在 Slint 壳里的“双滚动真源”残留：

- [`ConsolePane`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/panes.slint) 不再使用 `ScrollView.viewport-y`
- asset browser [`SelectionDetailsRail`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint) 也不再使用 `ScrollView.viewport-y`
- 两者现在都改成 `clip + retained content stack + host-driven scroll_px`
- wheel 输入继续由 [`detail_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/detail_pointer/mod.rs) 和 [`scroll_surface_host.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/scroll_surface_host.rs) 这条 shared scroll bridge 处理

这让非-menu pane surface 的滚动 authority 终于一致了：

- Slint 只做表现层裁剪和位移
- shared `UiSurface + UiPointerDispatcher` 才是 scroll offset 的唯一权威
- `InspectorPane`、`ConsolePane`、`SelectionDetailsRail` 不再出现“wheel 进 shared dispatcher，但 viewport-y 仍由 Slint 本地维护”的混合状态

同一轮里，[`callback_dispatch/viewport/snap_cycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs) 的 helper 可见性也已经补齐到 `viewport` 子树内，避免 [`route_mapping.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs) 在真实宿主 shared viewport dispatcher 主链上再次因为模块拆分而失编。

## What This Slice Already Enables

这一轮已经把第一段真实 `slint_host/app.rs` callback 接缝落下来了，并把后续迁移最危险的几件事先定住了：

- editor composite 不需要再依赖手写 `.slint` 业务树作为真源
- binding 稳定命名空间不会因为切换宿主结构而丢失
- shared template runtime 和 typed editor binding 已经进入真实宿主 callback 链路
- inspector live edit、mesh import path live edit、以及 `Import Model` submit 已经不再由宿主直接解释 UI 语义，而是进入 typed draft/asset/runtime authority
- shell pointer route 现在已经能直接驱动 normalized tab drop route 和统一 dispatcher，而不是只给 `app.rs` 提供局部 group 字符串
- viewport overlay hit-test 现在也能通过 shared `UiSurface + UiPointerDispatcher` 收口，而不是保留 editor-local `picking.rs` 例外
- 未来做 parity harness 时，可以直接对比“模板实例 binding 列表”和“旧 Slint 壳回调投影”的等价性

这正是兼容迁移第一阶段需要的基础，而且仍然不必先去重写高冲突的 startup 状态机或 `workbench.slint` 主业务树。

## Manager-side Native Window Seam

为了让下一刀 multi-window cutover 不再建立在“只有 overlay snapshot，没有原生窗口宿主状态”的过渡态上，manager 层这次已经先补齐了一条稳定 seam：

- [`window_host_manager.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/window_host_manager.rs) 现在维护 `NativeWindowHostState { window_id, handle, bounds }`
- [`workspace_state.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/workspace_state.rs) 会在 `recompute_session_metadata()` 里把 `WorkbenchLayout.floating_windows` 全量同步进 `WindowHostManager`
- [`layout_commands.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/layout_commands.rs) 会在 detach 出去的最后一个 view 被重新 attach 回主 workbench 后，通过 `reattach_window(...)` 清掉失效原生窗口记录
- [`EditorManager::native_window_hosts()`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/workspace_state.rs) 现在是公开 API，后续 secondary-window presenter 不需要穿透私有 manager 状态就能读取窗口宿主账本
- [`native_window_hosts.rs`](/E:/Git/ZirconEngine/zircon_editor/tests/native_window_hosts.rs) 已经锁住三件事：
  - detach 最后一个 view 后会生成 host record
  - restore 含 floating window 的 workspace 时会同步 bounds
  - config/bootstrap 默认布局不会伪造原生窗口记录

这条 seam 现在已经不再只停在 manager/bookkeeping：

- [`app/native_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/native_windows.rs) 现在会在真实 `slint_host` 里维护 `NativeWindowPresenterStore`
- presenter target 会先读取 [`EditorManager::native_window_hosts()`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/workspace_state.rs)，再交给 host-bounds-aware [`resolve_floating_window_outer_frame_with_host_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs)；当 manager 账本里的 bounds 还是零时，会直接回退到 shared floating-window outer frame，而不是生成不可见的 1x1 窗口
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 新增 `native_floating_window_mode` / `native_window_title` / `native_window_bounds`，同一套 `WorkbenchShell` 现在既能渲染主壳，也能把指定 floating window 投成 full-window child host
- [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 现在会在 secondary shell 创建时复用 [`wire_callbacks(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs)，把 child host 直接接回同一份 shared callback/runtime dispatcher
- native-mode [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 现在会把 floating tab activate/close、header focus click 和 `PaneSurface` 内部 pointer/scroll/control 回调重新转回 root host；header route 额外叠加 `native_window_bounds`，避免 child native window 因本地坐标丢失 shared shell 几何语义
- [`native_window_targets.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_window/native_window_targets.rs)、[`presenter_store.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_window/presenter_store.rs) 和 [`native_mode.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_window/native_mode.rs) 现在分别锁住 geometry fallback、presenter create/update/hide/creation-hook 生命周期，以及 native-mode callback forwarding 契约

因此 multi-window cutover 已经推进到“真实 secondary native window create/update/hide + shared presentation reuse”，不再只是 overlay-first 过渡态。

这条链在最新一刀又往 shared projection 收了一层：

- [`floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 现在把 native host bounds 直接并入 floating-window `outer/tab/content` frame helper，而不是只在 presenter target collection 里特判
- [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 会在每次 recompute snapshot 一次 `native_window_hosts()`，并把这份状态同时喂给 root `apply_presentation(...)`、floating document tab pointer layout、shell drag surface，以及 secondary child shell presentation
- [`native_window_targets.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_window/native_window_targets.rs)、[`floating_strip_bounds.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_document_tab_pointer/floating_strip_bounds.rs) 和 [`floating_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_tab_drag/floating_pointer.rs) 现在分别锁住 geometry fallback、floating tab strip host-bounds preference，以及 floating attach surface host-bounds preference

当前仍然刻意保留的限制是：

- child-window 的 tab/header/pane 本地 pointer 与 scroll/control 回调已经迁到 shared dispatcher
- native close request 现在只在“该 floating window 内所有 tab 都可关闭”时转成 shared `CloseView` 序列；如果包含 non-closeable tab，则仍会保守拦截
- pointer capture 与更细的 multi-window focus/close UX 还没有完整落地

## Deliberate Non-Goals In This Slice

这一轮故意没有在 `zircon_editor::ui` 里加入：

- docking 拓扑求解
- `WorkbenchLayout` 替代品
- 全量替换手写 Slint callback/属性投影
- reflection/nativeBinding 协议改写
- runtime `UiTree` 自动装配

这些后续都应该建立在当前 catalog/registry/adapter 已经稳定的前提上继续推进。

## Next Integration Steps

按当前边界，后续最合理的接线顺序是：

1. 在已经落地的 secondary native presenter 之上，继续把 child-window 的 pointer capture 与更细的 close/focus UX 迁到 shared dispatcher，而不是让 child shell 永久停在“基础输入已统一，但 capture/close 仍保守”的阶段
2. 把 `PaneSurfaceRoot` projection-backed frame fallback 从 welcome recent 推广到 hierarchy、detail scroll 和其他仍依赖 Slint callback 宽高的 pane-local shared pointer surface
3. 当前 shell inventory 已经没有剩余 standalone non-menu popup/dialog-like overlay surface；下一条 overlay 相关工作应转到 multi-window / detach-promote host，或未来新增 dialog/popup 时直接落在 shared `UiSurface` dispatcher，而不是回退到宿主本地 hit-test
4. 在真实 `slint_host` 的 presentation 侧继续扩大 `SlintUiHostProjection` 消费范围：root shell frame 已经进入主链，下一步是 clip/z-order 以及 drawer 可见状态下更多 document/pane shell frame
5. 为旧手写 Slint 壳和描述文件驱动宿主补真正的同 fixture parity snapshot，对齐 shell structure、route 结果和 event log
6. 继续保留 startup/welcome 的 host-owned session 执行、preset 和其他高冲突特殊语义的兼容 seam，直到 startup session 也进入受控 runtime authority

只要遵守这个顺序，`slint_host` 的重构就不需要再重新定义模板语义或 binding 命名，而是只做宿主接线。

这一轮新增的直接验证包括：

- `cargo test -p zircon_editor --lib host::slint_host::ui::tests --locked -- --nocapture`
- `cargo test -p zircon_editor --lib slint_detail_pointer --offline -- --nocapture`
- `cargo check -p zircon_editor --lib --offline`
- `cargo test -p zircon_editor --test workbench_slint_shell --offline -- --nocapture`
- `cargo test -p zircon_editor --lib slint_menu_pointer --locked -- --nocapture`
- `cargo test -p zircon_editor --lib slint_tab_drag --locked -- --nocapture`
- `cargo test -p zircon_editor --lib shared_drag_capture_surface_replaces_legacy_direct_drop_callback_abi --locked -- --nocapture`
- `cargo test -p zircon_editor --lib slint_drawer_resize --locked -- --nocapture`
- `cargo test -p zircon_editor --lib shared_resize_surface_replaces_legacy_direct_resize_callback_abi --locked -- --nocapture`
- `cargo test -p zircon_editor --lib shared_detail_scroll_surfaces_do_not_leave_slint_scrollview_as_authority --locked -- --nocapture`
- `cargo check -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --lib template_runtime --locked -- --nocapture`
- `cargo test -p zircon_editor draft_command_bindings_parse_into_typed_payloads_instead_of_custom_calls --locked`
- `cargo test -p zircon_editor --lib draft_inspector_binding_normalizes_and_updates_live_snapshot --locked`
- `cargo test -p zircon_editor --lib draft_mesh_import_path_binding_normalizes_and_updates_live_snapshot --locked`
- `cargo test -p zircon_editor --lib inspector_draft_field_dispatch_updates_live_snapshot_without_scene_side_effects --locked`
- `cargo test -p zircon_editor --lib mesh_import_path_edit_dispatch_updates_live_snapshot_without_backend_sync --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_call_action_dispatches_typed_draft_actions --locked`
- `cargo test -p zircon_editor --lib asset_command_binding_roundtrips_for_import_model --locked`
- `cargo test -p zircon_editor --lib asset_import_binding_normalizes_to_runtime_host_request --locked`
- `cargo test -p zircon_editor --lib builtin_asset_surface_import_model_dispatches_host_request_from_template --locked`
- `cargo test -p zircon_editor --lib workbench_reflection_call_action_dispatches_asset_import_action --locked`
- `cargo test -p zircon_editor --lib builtin_pane_surface_trigger_action_matches_legacy_menu_action_dispatch --locked -- --nocapture`
- `cargo test -p zircon_editor --lib pane_surface_actions_use_generic_template_callbacks_instead_of_legacy_menu_action_abi --locked -- --nocapture`
- `cargo test -p zircon_editor --lib shared_viewport_toolbar_surface_replaces_legacy_direct_click_routes --locked -- --nocapture`
- `cargo test -p zircon_editor --locked -- --nocapture`
- `cargo test -p zircon_editor --lib asset_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi --offline -- --nocapture`
- `cargo test -p zircon_editor --lib slint_asset_pointer --offline -- --nocapture`
- `cargo test -p zircon_editor --lib slint_callback_dispatch --offline -- --nocapture`

## Latest Shared Root Projection Slice

这轮兼容迁移继续沿着 “template/runtime/shared core authority 优先” 往前推，而不是在 Slint 壳里增加新的几何特判：

- [`resolve_root_document_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 在 visible drawer 存在时，已经不再把 legacy document region 当成 root shell 的真源
- 实际 authority 现在改成：
  - shared builtin root `WorkbenchBody` 提供 document shell 的位置和总可用区域
  - legacy left/right/bottom drawer frame 只提供剩余 extents
- [`resolve_root_document_tabs_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 同步复用了这条 resolved document frame，因此 template-projected document tabs strip 与 root document shell 又少了一层 mixed-authority 偏差
- [`resolve_root_viewport_content_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 也开始从 resolved document frame 推导 visible-drawer viewport content，因此 root shell 不再为 viewport content 留下一条单独的 legacy geometry seam
- 新增的 [`apply_presentation_prefers_shared_root_projection_for_visible_drawer_document_region`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/tests.rs) 把 document shell 和 viewport frame 的这条 cutover 一起固定成 regression

当前这条迁移链上仍刻意保留的 boundary 是：

- visible drawer shell/header 还没有被 builtin template 直接建模
- 更宽的 `cargo test -p zircon_editor --lib ... --locked` 复跑目前继续被无关的 `zircon_graphics` `wgpu` API 漂移抢先阻断

最新这一刀又把 multi-window 兼容链上的 floating authority 从“shared helper first”推进到了“shared recompute bundle first”：

- [`FloatingWindowProjectionBundle`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 现在在真实 `slint_host` 的 `recompute_if_dirty()` 内一次性构建并缓存；它不只是继续暴露 `outer/tab/content` frame，还把“是否真的存在 native host 记录”和“最终采用的 host_frame”一起固定下来
- [`document_tab_pointer/build_workbench_document_tab_pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs)、[`shell_pointer/drag_surface.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs)、[`ui/floating_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/floating_windows.rs)、[`app/helpers.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs)、[`app/viewport.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/viewport.rs) 与 [`app/native_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/native_windows.rs) 现在优先直接消费这份 bundle，而不是各自重新拼 `WorkbenchShellGeometry + native_window_hosts`
- 新增 real-host regression [`child_window_host_recompute_caches_floating_window_projection_bundle_for_detached_window`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs) 已经把“host 真正缓存 bundle entry，而不是只在 helper 调用点临时计算”固定成兼容迁移约束
- 这一刀的 focused validation 现在包括：
  - `cargo check -p zircon_editor --lib --locked`
  - `cargo test -p zircon_editor --lib floating_window --locked -- --nocapture`
  - `cargo test -p zircon_editor --lib shared_document_tab_pointer_layout_prefers_native_window_host_bounds_for_floating_strip --locked -- --nocapture`
  - `cargo test -p zircon_editor --lib shared_shell_pointer_route_prefers_native_window_host_bounds_for_floating_attach_surface --locked -- --nocapture`

因此 multi-window 这条兼容 cutover 现在的边界已经从“shared helper first”推进到“shared recompute bundle first”。下一步如果继续推进，就应该把 bundle 的 outer producer 再往 builtin template/runtime 或更纯的 shared projection producer 提升，而不是回到更多 helper stitching。

## Latest Floating-Window Dedicated Shared Source

这一刀把 floating-window base outer-frame 的 shared source 再往前推了一层：真实宿主不再从 root-shell builtin projection 的 `DocumentHostRoot + WorkbenchBody` 借位，而是改成一份 dedicated floating-window builtin template/runtime source。

- [`floating_window_source.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml) 新增了独立的 builtin 模板文档，直接建模 `FloatingWindowCenterBandRoot` 和 `FloatingWindowDocumentRoot`
- [`callback_dispatch/template_bridge/floating_window_source/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/mod.rs) 现在只保留 structural wiring；真正 owner 已拆到 `bridge.rs`、`error.rs`、`source_frames.rs`、`surface.rs`
- [`callback_dispatch/template_bridge/floating_window_source/bridge.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs) 负责 `BuiltinFloatingWindowSourceTemplateBridge` 生命周期，[`source_frames.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/source_frames.rs) 负责导出 `BuiltinFloatingWindowSourceFrames`；这套 owner 只输出浮窗默认/钳制所需的中心带和文档区域 frame，不再复用 root shell 业务树节点
- [`floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 的 `resolve_floating_window_projection_shared_source(...)` 现在直接消费这份 dedicated source frame bundle，并继续复用 `default_floating_window_frame(...) + clamp_floating_window_frame(...)`，没有再复制 placement 数学
- [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 在每次 root host 重算时会同步重算 `floating_window_source_bridge`，再用它统一驱动：
  - `EditorManager::sync_native_window_projection_bounds(...)`
  - recompute-time `FloatingWindowProjectionBundle`
- 生产路径里的 authority 顺序现在固定为：
  - native host bounds
  - dedicated shared floating-window source（`requested_frame + FloatingWindowDocumentRoot + FloatingWindowCenterBandRoot`）
  - legacy geometry fallback
- [`app/tests/floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs) 与 [`slint_callback_dispatch/template_bridge/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs) 已经把预期改成 dedicated source bridge，而不是 root-shell-derived source

这轮 red 证据是明确的：

- `cargo test -p zircon_editor --lib builtin_floating_window_source_template_bridge_recomputes_surface_backed_frames_with_shell_size --offline` 首先失败在 `BuiltinFloatingWindowSourceTemplateBridge` 与 `SlintEditorHost::floating_window_source_bridge` 缺失

当前 widened green 仍被无关工作区漂移阻断，不过最新阻塞点已经继续前移到 [`editing/ui_asset`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/mod.rs) 这轮未收敛的模块迁移：

- `cargo check -p zircon_asset --lib --offline` 已经单独通过，说明 earlier `zircon_asset` manager 漂移不再是当前第一阻塞
- `cargo test -p zircon_editor --lib builtin_floating_window_source_template_bridge_recomputes_surface_backed_frames_with_shell_size --offline`
- `cargo test -p zircon_editor --lib child_window_host_recompute_caches_floating_window_projection_bundle_for_detached_window --offline`
- 两条命令现在都会先失败在 `binding_inspector` wrapper include、`session.rs` / `session/mod.rs` 双入口、以及一整串 `ui_asset` wrapper/re-export/type-inference 错误，而不是这条 floating-window authority cutover 本身

## Latest Generic Host Boundary Slice

这一刀先把 builtin root host 的“文档身份”从 workbench 专名收口成 generic host 边界，而不是继续让 runtime/template/host 主入口对外暴露业务名。

- [`zircon_editor/src/ui/template_runtime/builtin/template_documents.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/builtin/template_documents.rs) 现在只把 `ui.host_window` 注册为 builtin root template 的 `document_id`；旧 workbench shell document alias、测试 re-export 和重复 document entry 已删除，同一份 [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) 不再被双身份注册
- [`zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs) 里的 `UiHostWindow/MenuBar/ActivityRail/DocumentHost/StatusBar` descriptor 已切到 `ui.host_window`，因此 component catalog 的默认根入口只依赖 generic host 文档身份
- [`zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs) 只保留 generic `load_builtin_host_templates()` 入口，不再通过旧 workbench shell 命名包装加载 builtin host templates
- [`zircon_editor/src/ui/slint_host/callback_dispatch/constants.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/constants.rs) 与 [`zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 主消费路径已经改用 `BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID` 与 `UI_HOST_WINDOW_CONTROL_ID`，因此真实 Slint host 的 builtin root projection 只按 generic host document id 取文档
- [`zircon_editor/src/tests/host/template_runtime/host_window_document.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/template_runtime/host_window_document.rs) 现在以 `ui.host_window` 作为 builtin projection / shared surface 的唯一预期；`editor_ui_host_runtime_registers_only_generic_host_window_document_id` 会锁住 shared surface tree id，`builtin_host_runtime_exposes_only_generic_host_window_document_id` 会禁止 template runtime 和对应测试恢复旧 alias 常量或旧 literal
- [`zircon_editor/ui/workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 现在把导出的 `UiHostWindow` 真正收口成 generic host window/bootstrap wrapper：root 只保留 window 级属性、对 `WorkbenchHostScaffold` 的属性别名，以及 callback forwarding；原来的 menu/drawer/document/floating 业务树已经落回内部 `WorkbenchHostScaffold`
- 最新这一刀继续把 root/bootstrap 与 pane-surface 目录边界切得更干净：[`zircon_editor/ui/workbench/pane_data.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_data.slint) 现在独立承接 `PaneData + SceneViewportChromeData + ProjectOverviewData`，[`zircon_editor/ui/workbench/asset_panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/asset_panes.slint) 独立承接 `ProjectOverviewPane/AssetsActivityPane/AssetBrowserPane`，[`zircon_editor/ui/workbench/ui_asset_editor_pane.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_pane.slint) 进一步退化成 `UiAssetEditorPane` orchestration shell，而 `UiAssetEditorPaneData`、shared widgets、center column、inspector panel 和 stylesheet panel 已分别拆到 [`ui_asset_editor_data.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_data.slint)、[`ui_asset_editor_components.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_components.slint)、[`ui_asset_editor_center_column.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_center_column.slint)、[`ui_asset_editor_inspector_panel.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_inspector_panel.slint) 和 [`ui_asset_editor_stylesheet_panel.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_stylesheet_panel.slint)；[`zircon_editor/ui/workbench/pane_fields.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_fields.slint) 继续只承接 `CompactField/AxisField`，而 [`zircon_editor/ui/workbench/host_workbench_surfaces.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint) 则独立承接 `HostSideDockSurface/HostDocumentDockSurface/HostBottomDockSurface/HostFloatingWindowLayer/HostNativeFloatingWindowSurface`
- 对应地，[`zircon_editor/ui/workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 与 [`zircon_editor/ui/workbench/host_components.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 都不再直接 import `pane_surface.slint`；`PaneSurfaceHostContext` 现在从 `host_workbench_surfaces.slint` 转出，`PaneData/ProjectOverviewData/SceneNodeData` 从 `pane_data.slint` 转出
- [`zircon_editor/tests/workbench_slint_shell.rs`](/E:/Git/ZirconEngine/zircon_editor/tests/workbench_slint_shell.rs) 新增 `ui_host_window_root_delegates_to_internal_scaffold_only`，把“exported root 只能委托内部 scaffold，不能重新直接拥有 `top_bar` / floating overlay / main content zone” 固定成 source regression

这次不再保留旧 alias：

- 当前审计没有发现 production caller 需要旧 builtin host document identity；剩余命中只来自 alias 自身、对应测试和旧文档说明
- `workbench_shell.ui.toml` 仍然是当前 root host asset 文件名，但它只通过 generic `ui.host_window` 文档身份进入 runtime registry
- Generic host boundary 后续仍要继续迁出 menu/drawer/document/floating 子结构；这些业务结构不再作为保留重复 builtin document identity 的理由

这轮 focused validation 结果：

- `cargo check -p zircon_editor --lib --locked`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked`
- `cargo test -p zircon_editor --locked --quiet --test workbench_slint_shell --test workbench_slint_ui_asset_authoring_shell --test workbench_slint_ui_asset_theme_shell`
- `cargo test -p zircon_editor --lib template_runtime --locked -- --nocapture`
- `cargo test -p zircon_editor workbench_surface_components_own_pane_surface_seam_instead_of_host_components --locked --quiet`
- `cargo test -p zircon_editor shared_viewport_surface_replaces_legacy_direct_pointer_callback_abi --locked --quiet`
- `cargo test -p zircon_editor pane_surface_actions_use_generic_template_callbacks_instead_of_legacy_menu_action_abi --locked --quiet`
- `cargo test -p zircon_editor native_floating_window_mode_forwards_tabs_header_and_pane_callbacks_to_root --locked --quiet`

2026-05-01 继续清理旧 builtin host document alias 后，focused validation 已重新覆盖当前 active Rust path：

- `cargo test -p zircon_editor --lib builtin_host_runtime_exposes_only_generic_host_window_document_id --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture`，1 passed / 0 failed / 847 filtered out
- `cargo test -p zircon_editor --lib template_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture`，36 passed / 0 failed / 812 filtered out
- `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture`，8 passed / 0 failed / 840 filtered out
- `cargo test -p zircon_editor --lib catalog_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture`，3 passed / 0 failed / 845 filtered out
- `rustfmt --edition 2021 --check` 覆盖本轮触及 Rust 文件，通过
- exact source sweeps confirmed the old builtin host document literal and `LEGACY_HOST_WINDOW_DOCUMENT_ID` are gone from active `zircon_editor/src` and docs; only unrelated `editor.workbench.shell_pointer.*` routing ids remain under shell pointer surfaces
- touched-file `git diff --check` only reported Windows LF-to-CRLF warnings

当前更宽的 `cargo test -p zircon_editor --lib builtin_workbench_template_bridge --locked -- --nocapture` 仍会先失败在相邻 `editing/ui_asset/session.rs` source-cursor refactor 遗留的缺失 helper（例如 `remap_source_byte_offset(...)`）上，而不是这次 `ui.host_window` cutover 本身。


