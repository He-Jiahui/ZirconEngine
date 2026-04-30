---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
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
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs
  - zircon_editor/src/ui/slint_host/detail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/scroll_surface_host.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_popup_state.rs
- zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/host_page_pointer/mod.rs
- zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs
- zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
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
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/action_control.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/drawer_toggle.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/document_tab.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/main_page.rs
  - zircon_editor/src/ui/template_runtime/runtime.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
- zircon_editor/src/ui/slint_host/viewport/mod.rs
- zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/core/host/manager.rs
  - zircon_editor/src/core/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/mod.rs
  - zircon_editor/src/core/host/manager/layout_commands.rs
  - zircon_editor/src/core/host/manager/window_host_manager.rs
  - zircon_editor/src/core/host/manager/workspace_state.rs
  - zircon_editor/src/core/host/manager/startup/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/layout/mod.rs
  - zircon_editor/src/ui/workbench/layout/layout_manager.rs
  - zircon_editor/src/ui/workbench/layout/manager/mod.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/workbench/fixture/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/ui/workbench/startup/mod.rs
  - zircon_editor/src/ui/workbench/view/mod.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_surface_contract.slint
  - zircon_editor/ui/workbench/host_scene.slint
  - zircon_editor/ui/workbench/host_components.slint
  - zircon_editor/ui/workbench/host_workbench_surfaces.slint
  - zircon_editor/ui/workbench/pane_data.slint
  - zircon_editor/ui/workbench/host_context.slint
  - zircon_editor/ui/workbench/pane_surface.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/pane_fields.slint
  - zircon_editor/ui/workbench/ui_asset_editor_pane.slint
  - zircon_editor/ui/workbench/ui_asset_editor_data.slint
  - zircon_editor/ui/workbench/ui_asset_editor_components.slint
  - zircon_editor/ui/workbench/ui_asset_editor_center_column.slint
  - zircon_editor/ui/workbench/ui_asset_editor_inspector_panel.slint
  - zircon_editor/ui/workbench/ui_asset_editor_stylesheet_panel.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/src/tests/host/slint_detail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_list_pointer/
  - zircon_editor/assets/icons/ionicons/folder-open-outline.svg
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/fixtures/workbench/default-layout.json
  - zircon_editor/fixtures/workbench/view-descriptors.json
  - zircon_editor/fixtures/workbench/view-instances.json
  - zircon_editor/src/core/editing/ui_asset/session.rs
  - zircon_editor/src/core/editing/ui_asset/source_sync.rs
  - zircon_editor/src/core/editing/ui_asset/tree_editing.rs
  - zircon_editor/src/core/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/core/editing/ui_asset/inspector_semantics.rs
  - zircon_editor/src/core/editing/ui_asset/command.rs
  - zircon_editor/src/core/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/src/tests/editing/ui_asset/
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/host/slint_window/
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
implementation_files:
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/core/editing/state/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_editor/src/ui/slint_host/app/pointer_layout.rs
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
  - zircon_editor/src/ui/slint_host/app/workspace_docking.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/template_runtime/runtime.rs
  - zircon_editor/src/core/host/manager/startup/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/detail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/scroll_surface_host.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs
  - zircon_editor/src/ui/slint_host/menu_pointer/host_menu_pointer_bridge_popup_state.rs
- zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/host_page_pointer/mod.rs
- zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
- zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs
- zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs
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
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/action_control.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
- zircon_editor/src/ui/slint_host/viewport/mod.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/startup/mod.rs
  - zircon_editor/src/ui/workbench/fixture/mod.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_surface_contract.slint
  - zircon_editor/ui/workbench/host_scene.slint
  - zircon_editor/ui/workbench/host_components.slint
  - zircon_editor/ui/workbench/host_workbench_surfaces.slint
  - zircon_editor/ui/workbench/pane_data.slint
  - zircon_editor/ui/workbench/host_context.slint
  - zircon_editor/ui/workbench/pane_surface.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/ui/workbench/pane_fields.slint
  - zircon_editor/ui/workbench/ui_asset_editor_pane.slint
  - zircon_editor/ui/workbench/ui_asset_editor_data.slint
  - zircon_editor/ui/workbench/ui_asset_editor_components.slint
  - zircon_editor/ui/workbench/ui_asset_editor_center_column.slint
  - zircon_editor/ui/workbench/ui_asset_editor_inspector_panel.slint
  - zircon_editor/ui/workbench/ui_asset_editor_stylesheet_panel.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/assets/icons/ionicons/folder-open-outline.svg
  - zircon_editor/src/tests/host/slint_list_pointer/
  - zircon_editor/src/core/host/manager.rs
  - zircon_editor/src/core/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/mod.rs
  - zircon_editor/src/core/host/manager/layout_commands.rs
  - zircon_editor/src/core/host/manager/window_host_manager.rs
  - zircon_editor/src/core/host/manager/workspace_state.rs
  - zircon_editor/src/ui/workbench/layout/mod.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/fixtures/workbench/default-layout.json
  - zircon_editor/fixtures/workbench/view-descriptors.json
  - zircon_editor/fixtures/workbench/view-instances.json
  - zircon_editor/src/core/editing/ui_asset/session.rs
  - zircon_editor/src/core/editing/ui_asset/source_sync.rs
  - zircon_editor/src/core/editing/ui_asset/tree_editing.rs
  - zircon_editor/src/core/editing/ui_asset/binding_inspector.rs
  - zircon_editor/src/core/editing/ui_asset/inspector_semantics.rs
  - zircon_editor/src/core/editing/ui_asset/command.rs
  - zircon_editor/src/core/editing/ui_asset/undo_stack.rs
  - zircon_editor/src/core/host/manager/ui_asset_sessions.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - docs/editor-and-tooling/prototypes/editor-workbench-hybrid-shell.html
plan_sources:
  - user: 2026-04-13 JetBrains Hybrid Workbench Shell Spec + Implementation Plan
  - user: 2026-04-14 Slint Workbench 响应式 AutoLayout 与约束求解计划
  - user: 2026-04-14 实现运行时/编辑器共享 UI 布局与事件系统架构计划的首个共享 core 切片
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - user: 2026-04-14 编辑器启动最近工程与 Welcome 新建工程计划
  - user: 2026-04-14 编辑器 Builtin 资产归位与 Revision 稳定化计划
  - .codex/plans/Zircon UI Editor UI Binding & Reflection Architecture.md
  - user: 2026-04-15 Scene Viewport Gizmos/Handle/Overlay 规范化方案
  - user: 2026-04-15 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-16 把非-menu 的 popup/dialog/tree/list scroll 输入继续迁到同一套 shared pointer dispatcher
  - user: 2026-04-16 继续下一刀，把 secondary native window presenter 接到真实 slint_host
  - user: 2026-04-17 Source/Hierarchy/Canvas 的更强选中同步和 source roundtrip 体验
  - user: 2026-04-17 parent-specific slot/layout inspector，补 Overlay/Grid/Flow/ScrollableBox 语义
  - user: 2026-04-17 designer canvas 的可视化 authoring：插入、重排、reparent、wrap/unwrap
  - user: 2026-04-17 Bindings Inspector 的下一版：事件枚举选择、action/payload 结构化编辑
  - user: 2026-04-17 Palette 到真实节点/引用节点创建的落地
  - user: 2026-04-17 结构化 undo/redo，从当前 source-text 级别继续往 tree-command 演进
tests:
  - zircon_ui/src/tests/shared_core.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/tests/workbench_drag_targets.rs
  - zircon_editor/tests/native_window_hosts.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/host/slint_drawer_resize/mod.rs
  - zircon_editor/src/tests/host/slint_detail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_editor/src/tests/host/slint_activity_rail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_host_page_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_document_tab_pointer/
  - zircon_editor/src/tests/host/slint_drawer_header_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_viewport_toolbar_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/layout.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/surface_contract.rs
- zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/mod.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/workbench/template_bridge.rs
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/slint_event_bridge/mod.rs
  - zircon_editor/src/tests/host/slint_window/
  - zircon_editor/src/tests/host/slint_asset_refresh/mod.rs
  - zircon_editor/src/tests/host/slint_builtin_assets.rs
  - zircon_editor/src/tests/editing/ui_asset/
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/tests/workbench_window_resize.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/tests/workbench/fixture/default_preview.rs
  - zircon_editor/src/tests/workbench/fixture/view_model_projection.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/tests/workbench/view_model/document_workspace.rs
  - zircon_editor/src/tests/workbench/view_model/welcome_page.rs
  - zircon_editor/src/tests/workbench/view_model/support.rs
  - zircon_editor/src/tests/workbench/reflection/model_projection.rs
  - zircon_editor/src/tests/workbench/reflection/remote_routes.rs
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
  - cargo test -p zircon_editor --lib slint_tab_drag --locked -- --nocapture
  - cargo test -p zircon_editor --lib workbench_view_model_exposes_floating_windows_as_workspace_tabs --locked -- --nocapture
  - cargo test -p zircon_editor --lib scene_document_pane_projects_viewport_toolbar_state -- --nocapture
  - cargo test -p zircon_editor --lib viewport_toolbar -- --nocapture
  - cargo test -p zircon_editor --lib slint_viewport_toolbar_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib viewport_ --offline -- --nocapture
  - cargo test -p zircon_editor --lib template_runtime -- --nocapture
  - cargo test -p zircon_editor --lib slint_callback_dispatch -- --nocapture
  - cargo test -p zircon_editor --lib resolve_floating_window_focus_instance_ --locked -- --nocapture
  - cargo test -p zircon_editor --lib floating_window --locked -- --nocapture
  - cargo test -p zircon_editor --lib shared_document_tab_pointer_layout_prefers_native_window_host_bounds_for_floating_strip --locked -- --nocapture
  - cargo test -p zircon_editor --lib shared_shell_pointer_route_prefers_native_window_host_bounds_for_floating_attach_surface --locked -- --nocapture
  - cargo test -p zircon_editor --lib shared_viewport_surface_replaces_legacy_direct_pointer_callback_abi --locked -- --nocapture
  - cargo test -p zircon_editor --locked
  - cargo test -p zircon_editor slint_drawer_resize -- --nocapture
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_ui shared_core -- --nocapture
  - cargo test -p zircon_editor --test workbench_autolayout -- --nocapture
  - cargo test -p zircon_editor --test workbench_drag_targets --locked -- --nocapture
  - cargo test -p zircon_editor --test native_window_hosts --locked -- --nocapture
  - cargo test -p zircon_editor --lib slint_window --locked -- --nocapture
  - cargo check -p zircon_graphics --lib --locked
  - cargo check -p zircon_graphics --lib --offline
  - cargo check -p zircon_editor --lib --locked
  - cargo check -p zircon_editor --lib --offline
  - cargo test -p zircon_asset -p zircon_manager --locked
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
  - cargo check -p zircon_editor --lib --offline
  - cargo test -p zircon_editor --lib slint_detail_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib asset_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_asset_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_callback_dispatch --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_activity_rail_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_host_page_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_document_tab_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_drawer_header_pointer --offline -- --nocapture
  - cargo test -p zircon_editor --lib slint_ --offline -- --nocapture
  - cargo test -p zircon_editor --test workbench_slint_shell --offline -- --nocapture
  - cargo test -p zircon_editor --test workbench_window_resize --offline -- --nocapture
  - cargo test -p zircon_editor --lib uses_region_frame_fallback_in_real_host --locked -- --nocapture
  - cargo test -p zircon_editor --lib --locked ui_asset_editor_
  - cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_
  - cargo test -p zircon_editor --lib --locked editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo
  - cargo test -p zircon_editor --lib --locked tests::host::slint_window::child_window_callback_wiring_tracks_source_window_for_pane_interactions
  - cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_
doc_type: module-detail
---

# Editor Workbench Hybrid Shell

## Purpose

这一版 workbench shell 的目标不是继续堆一个“功能都在，但视觉和行为都很散”的宿主，而是把 editor 壳体收束成一套明确的 Hybrid Shell 规范：

- 视觉方向固定为 JetBrains Rider 对齐的 workbench shell
- 默认布局固定为 editor-first，大中台 viewport 占主导
- Pane 语义和空状态固定属于 pane 自己，不属于某个位置
- 布局仍完全由 `WorkbenchLayout` 驱动，用户可以把 pane 挪到任何 drawer/document host
- HTML 原型、`WorkbenchViewModel`、反射树和未来 Slint 宿主必须读同一套语义，而不是各做各的

## Current Root Bootstrap Boundary

最新一轮 cutover 已经把 root Slint 文件切成真正的 bootstrap 边界：

- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 现在只导出通用 `UiHostWindow`，负责窗口属性、重新导出 [`WorkbenchHostContext`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_context.slint) 与 `PaneSurfaceHostContext` 两条 generic/global seam，并单点挂载 `WorkbenchHostScaffold`；`PaneSurfaceHostContext` 不再直接从 [`pane_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_surface.slint) 暴露，而是经由新的 [`host_workbench_surfaces.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint) 统一承接
- [`host_scaffold.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_scaffold.slint) 这一轮又继续变薄：它现在只剩 root host property 承接、grouped [`HostWindowBootstrapData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 产出，以及把窗口 surface 整包委托给高层 wrapper
- [`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 也继续压薄成真正的 orchestrator：它现在只负责接入 [`host_surface_contract.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface_contract.slint) 并在 main/native path 之间选用对应 surface wrapper
- 新增 [`pane_data.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_data.slint) 后，`SceneViewportChromeData`、`PaneData` 和 `ProjectOverviewData` 这几块 pane-local DTO 不再继续粘在 [`pane_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_surface.slint) 或 [`assets.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint)；`host_components.slint`、asset pane owner 与 root export 现在都直接依赖这层数据 owner，而不是继续把 pane catalog 文件当作数据入口
- [`host_components.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 现在只保留较低层的 menu chrome、host page bar、status bar、splitter/resize layer、tab drag overlay 与 grouped DTO；真正会实例化 `PaneSurface` 的 `side/document/bottom/floating/native-floating` 五块 pane-backed surface 已经迁到新的 [`host_workbench_surfaces.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint)
- 最新这轮又把 root/scaffold/surface 之间原先散装透传的 shell chrome + layout geometry ABI 收成两个 grouped DTO：[`HostWindowShellData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 和 [`HostWindowLayoutData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)
- 再下一刀把 tab/pane/floating-window collection 也并进 [`HostWindowSurfaceData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)：[`host_scaffold.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_scaffold.slint) 不再把 `host_tabs/left_tabs/document_pane/floating_windows` 逐项 fan-out 给 child surface，而是整包下传 `host_surface_data`
- 最新这一刀继续把 host interaction/layout seam 再收成 grouped DTO：[`HostMenuStateData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)、[`HostDragStateData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 和 [`HostTabDragOverlayData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 取代了原来散落在 [`WorkbenchHostContext`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_context.slint) 与 `host_surface.slint` 里的 menu hover/open/scroll、drag session、drop overlay scalar property
- [`HostMenuChrome`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 与 [`HostTabDragOverlay`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 继续直接消费这些 grouped contract；pane-backed [`HostSideDockSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint)、[`HostDocumentDockSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint)、[`HostBottomDockSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint) 和 [`HostFloatingWindowLayer`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint) 则迁到了新的 surface owner；`host_surface.slint` 不再逐项 fan-out `open_menu_index/drag_tab_id/drag_pointer_x/left_drop_width/...`
- 最新继续把高层 shell chrome / status / splitter capture 的 surface ABI 也压成 grouped DTO：[`HostMenuChromeData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)、[`HostPageChromeData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)、[`HostStatusBarData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)、[`HostResizeStateData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 和 [`HostResizeLayerData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 现在把 menu chrome、host page、status bar、resize overlay 这几块从一串 scalar/property alias 收成 surface-local contract
- 最新这一刀又把 `host_surface.slint` 自己继续压薄了一层：新增 [`host_scene.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_scene.slint) 统一承接 menu/page/document/side/bottom/status/resize/drag/floating 这些主壳 scene 组合；`host_surface.slint` 现在只保留 grouped DTO 组装与 main/native variant 选择，不再直接内联 leaf host catalog
- [`WorkbenchHostContext`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_context.slint) 现已同时持有 `menu_state`、`drag_state`、`resize_state` 三个 grouped interaction state；[`HostResizeLayer`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 不再在 `host_surface.slint` 本地保留 `resize_active/resize_group` 两个散装状态
- [`app/pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 现在一次性写入 `WorkbenchHostContext.menu_state`，[`app/workspace_docking.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/workspace_docking.rs) 则统一读写 `WorkbenchHostContext.drag_state`；drag-target 更新和 drop dispatch 不再依赖散装 `set_active_drag_target_group()/get_drag_tab_id()` 这一组标量 ABI
- 顺手清掉了一条完全未消费的主壳残留 payload：`breadcrumbs` 已从 root/scaffold/presentation/host component DTO 退出，不再占着 bootstrap ABI
- 最新这轮又继续把一批 pane-local pure proxy property 从 scaffold ABI 上拿掉：`welcome/hierarchy/assets/inspector/console/viewport image/mesh import` 这类只服务 `PaneSurfaceHostContext` 的状态不再先挂在 `WorkbenchHostScaffold`
- 再下一刀把这些 pane-local state 连 root `UiHostWindow` 上的 alias property 也一并删掉：[`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 现在只保留 host window/bootstrap 所需的 grouped host payload，Rust 侧改为直接写 `PaneSurfaceHostContext` global
- root 还进一步删掉了 `AssetBrowserPane` / `WelcomePane` / `DockTabButton` / `TabChip` / `ToolbarButton` 这类 probe-only 叶子导入，以及 `shared_pointer_hook_probes`、`shared_menu_anchor_probes`、`shared_welcome_control_probe` 这些只为旧 ABI 守卫服务的隐藏壳；`UiHostWindow` 不再保留 `asset_control_*`、`welcome_control_*`、`viewport_toolbar_pointer_clicked` 这组三类 pane-specific callback seam
- [`callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 里对应的宿主注册也同步下沉到了 `PaneSurfaceHostContext`：asset/welcome/viewport-toolbar 这组 generic pane callback 现在只走 `pane_surface_host.on_*`，不再回挂 root `UiHostWindow`
- [`apply_presentation.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/apply_presentation.rs)、[`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 与 [`host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 也同步切到 `ui.global::<PaneSurfaceHostContext>()`：presentation、hover/scroll state、viewport image 不再依赖 root setter
- menu popup 的按钮锚点 frame 也已经从 root/scaffold 代理面退出，改由 [`HostMenuChrome`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 在 host component catalog 内部直接持有；root/scaffold 不再暴露 `*_menu_button_frame` 这类 control-specific host property
- [`pane_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_surface.slint) 现在单独承接 `PaneSurface` catalog 本体；`PaneSurfaceHostContext` 的 root seam 已经改由 [`host_workbench_surfaces.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint) 转出，`PaneData` 与 `ProjectOverviewData` 则由 [`pane_data.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_data.slint) 统一提供
- [`pane_content.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_content.slint) 现在明确承接 Welcome、Project、Assets、Hierarchy、Inspector、Console、UiAssetEditor 和 Animation editor 这些业务 pane catalog；其中 Project/Assets/AssetBrowser 三块顶层资产 pane 已进一步迁到 [`asset_panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/asset_panes.slint)，[`assets.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint) 只再保留 asset DTO 和 leaf surface；[`PaneSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_surface.slint) 只保留 viewport shell、scene/game 特例和通用 chrome 包装，非 Scene/Game 内容统一委托给 `PaneContent`
- [`workbench_slint_shell.rs`](/E:/Git/ZirconEngine/zircon_editor/tests/workbench_slint_shell.rs) 与 [`template_assets.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/ui/boundary/template_assets.rs) 现在把这条边界锁成源码守卫：root 与 `host_components.slint` 不得再直接 import `pane_surface/assets/panes/welcome/animation_editor_pane`，而 `host_workbench_surfaces.slint` 成为唯一允许承接 pane-surface seam 的 workbench surface owner

因此，当前“`workbench.slint` 仍是业务真源”的说法对这个工作区已经不成立。更大面的 generic host boundary 任务仍然存在，但残留位置已经从 root bootstrap 收缩到 `host_surface.slint` 的高层 property orchestration / business mapping 面，以及更深一层的 pane-local schema。

## Latest Host Component Catalog Cut

generic host boundary 最近几刀继续把主壳分层成 `host_scaffold.slint -> host_surface.slint -> host_scene.slint -> host_components.slint + host_workbench_surfaces.slint`：

- menu chrome、document host、side/bottom drawer、floating-window layer 与 native floating window surface 已先从 scaffold 拆成独立 host component
- 最新又补上 `HostPageChrome`、`HostStatusBar`、`HostTabDragOverlay` 和 `HostResizeLayer`
- root 这边也同步把 pane-local context state 改成直接绑定 `PaneSurfaceHostContext`，所以 `host_scaffold.slint` 的顶层 property 面已经不再混着大批“自己不消费、只是代传给 global”的伪 ABI
- 最新又把原来还留在 scaffold 里的主窗口 / native window 高层组合抽成 [`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint)：scaffold 不再直接实例化 `HostMenuChrome`、`HostPageChrome`、`HostDocumentDockSurface`、`HostResizeLayer`、`HostFloatingWindowLayer` 或 `HostTabDragOverlay`
- 这轮继续把 root/scaffold/surface 三层之间的大批散装宿主 property 收敛成 `HostWindowShellData + HostWindowLayoutData + HostWindowSurfaceData`：[`apply_presentation.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) 现在直接设置 grouped host struct，child native presenter 也改成回写同一份 `host_shell`
- [`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 也不再重新声明 `host_tabs/left_pane/...` 这些 collection/pane passthrough property；高层 orchestration 直接从 `host_surface_data.*` 取值，`PaneData` / `TabData` 不再作为这层 root ABI 的单独输入
- 同一条线上，`host_surface.slint` 现在也不再直接给 menu/page/status/resize layer 逐项喂 `top_bar_height/project_path/status_secondary/left_splitter_frame/resize_group` 这类细碎 ABI，而是统一组装 `menu_chrome_data/page_chrome_data/status_bar_data/resize_layer_data`
- 最新这轮继续把 `side/document/bottom/floating` 四块 host surface 的细碎布局输入收成 [`HostSideDockSurfaceData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)、[`HostDocumentDockSurfaceData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)、[`HostBottomDockSurfaceData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 和 [`HostFloatingWindowLayerData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)；[`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 现在只组装 `left/right/document/bottom/floating` 五份 surface DTO，再交给 [`HostSideDockSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)、[`HostDocumentDockSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)、[`HostBottomDockSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 和 [`HostFloatingWindowLayer`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)
- 这一刀继续往 `host_surface.slint` 内部压 root-level orchestration 常量和 native floating scalar ABI：固定 outer/top-bar/header 尺寸现在先收成 [`HostWorkbenchSurfaceMetricsData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)，left/right/bottom/document 的派生布局与 tab origin 收成 [`HostWorkbenchSurfaceOrchestrationData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)，native floating child shell 则统一吃 [`HostNativeFloatingWindowSurfaceData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)
- 最新又把这整块 derived surface state 从 [`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 下推进新增的 [`host_surface_contract.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface_contract.slint)：`surface_metrics`、`surface_orchestration_data`、`menu/page/status/resize/drag` DTO、`left/right/document/bottom/floating` surface DTO、`workbench_scene_data` 和 `native_floating_surface_data` 现在都在 contract 层统一产出
- [`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 本体因此进一步变成薄 orchestrator：它只保留 [`HostWorkbenchWindowSurfaceHost`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 这层 wrapper、接入 [`HostWorkbenchWindowSurfaceContract`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface_contract.slint)，然后在 main/native path 之间选择 [`HostWorkbenchWindowSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 或 [`HostNativeWorkbenchWindowSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint)
- [`HostWorkbenchWindowSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 现在只消费 [`HostWorkbenchWindowSceneData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_scene.slint)，[`HostNativeWorkbenchWindowSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 只消费 [`HostNativeFloatingWindowSurfaceData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)；wrapper 已不再直接把 `floating_windows/native_floating_window_id/native_window_bounds/header_height` 这类 scalar/collection 再展开一次
- [`host_scaffold.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_scaffold.slint) 也不再保留 main/native 分支和 `shell_width_px/viewport_width` 这组标量输出，而是只产出 grouped [`HostWindowBootstrapData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 并整包挂接 [`HostWorkbenchWindowSurfaceHost`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint)
- [`workbench_slint_shell.rs`](/E:/Git/ZirconEngine/zircon_editor/tests/workbench_slint_shell.rs) 也同步把这条边界锁成源码守卫：scaffold 不能再把 `DockTabButton` host page loop、`status_bar_zone`、`left/right/bottom splitter`、`resize_active` touch capture 或原始 drag overlay block 拉回本体
- 同一份 [`workbench_slint_shell.rs`](/E:/Git/ZirconEngine/zircon_editor/tests/workbench_slint_shell.rs) 现已继续要求 `host_surface.slint` 以 `surface_data/layer_data` 绑定上述 grouped DTO，防止后续把 `region_frame/tabs/pane/header_height/floating_windows` 这类逐字段 fan-out 拉回 orchestration 层
- 同一份源码守卫现在也额外要求 `host_surface.slint` 只能通过 `surface_metrics/surface_orchestration_data/native_floating_surface_data` 这三份 grouped payload 组装 orchestration/native path，防止把 `outer_margin/top_bar_height/left_stack_width/native_window_bounds` 这类 root scalar ABI 再引回去

当前这条 slice 的直接结果是：

- `workbench.slint` 已经进一步逼近纯 root bootstrap，不再充当 pane-state setter/getter 中转层
- `host_scaffold.slint` 已经进一步收成 bootstrap DTO 层
- `host_surface.slint` 现在只保留薄 wrapper/orchestrator，真正的派生 surface state 已经下推到 `host_surface_contract.slint`
- `host_components.slint` 更明确地成为较低层 generic host catalog
- root/scaffold/surface 之间的主壳状态、窗口 bootstrap 与派生 surface state 已经收成 grouped host DTO，而不是几十个分散 setter/getter

但 cutover 还没完成；剩下的主要不是这些显式壳块，而是：

- `host_scaffold.slint` / `host_surface.slint` 之上仍有一层 workbench-specific window ABI 命名和 wrapper 层次，generic host surface 还没完全抽象成更统一的 surface bundle
- 更大面的 business pane kind 分支仍然存在，但 owner 已经收敛到 [`PaneContent`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_content.slint)；[`host_workbench_surfaces.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_workbench_surfaces.slint) 只负责 pane-backed surface ownership，不再承接 `PaneSurfaceHostContext` plumbing
- [`pane_content.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_content.slint) 现在直接从 [`ui_asset_editor_pane.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_pane.slint) 引入 `UiAssetEditorPane`；[`panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/panes.slint) 已收回成 `ToolWindowEmptyState/Hierarchy/Inspector/Console/Fallback` 这些泛化 pane owner，而 [`pane_fields.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_fields.slint) 则承接 `CompactField/AxisField` 这种共享字段件

## Viewport Raw Pointer Authority

workbench shell 现在不再把 scene/game viewport 当成一个“可以单独直连宿主回调”的例外区域。

最新收口后的规则是：

- `PaneSurface` 只上传统一 `viewport_pointer_event(kind, button, x, y, delta)` pointer fact
- `callback_wiring.rs` 只保留一个 viewport pointer 注册入口
- `app/viewport.rs` 通过 `SharedViewportPointerBridge` 把事实交给 shared dispatcher，再映射到 `EditorViewportEvent`
- editor host 不再把 viewport move/down/up/wheel 旁路写入 `InputManager`

因此，workbench shell 当前的交互 authority 分层已经变成：

- Slint: 提供宿主窗口、原始 pointer fact、视觉承载
- shared pointer bridge: 做 hit-test、capture、route 和事件归一化
- editor runtime: 执行 viewport 命令并产出 presentation/render side effects

## Latest Real-Host Pane Frame Fallback

最新这轮 workbench shell 收口不再只覆盖顶层 menu/list/viewport route，而是把几条真实宿主里仍可能收到零尺寸 callback 的 pane surface 继续拉回 shared geometry authority：

- `ChildWindowHostHarness::activate_workbench_page()` 现在会显式设脏 host layout，再切离 `Welcome` 页面；这让 root-shell pane pointer 回归真正运行在 workbench shell 几何上，而不是误用 welcome-page frame
- `Hierarchy`、`Console`、`Inspector` 三条 root-shell pointer/scroll callback 都已经锁定 `region frame fallback`，对应的 real-host 回归测试统一跑在 `uses_region_frame_fallback_in_real_host`
- `AssetBrowser` exclusive page 的 details rail 也不再只吃 Slint callback 局部宽高；当 callback 上传 `0x0` 时，真实宿主会回退到 shared host frame，再继续驱动 `ScrollSurfaceHostState`
- `asset_tree_pointer_*`、`asset_content_pointer_*` 和 `asset_reference_pointer_*` 现在也已经全部改成同一条 host fallback：`surface_mode == "activity"` 走 `ViewContentKind::Assets`，`surface_mode == "browser"` 走 `ViewContentKind::AssetBrowser`
- 这意味着左上 `Assets` drawer 和 exclusive `Asset Browser` page 的 tree/content/reference 三块 surface，在 root shell 上都不再依赖 Slint callback 必须上传有效宽高；shared `WorkbenchShellGeometry` / template frame 重新成为 pane viewport size 的真源
- real-host 回归现在额外锁住 6 个 asset pane 用例：`activity/browser` 两条链各自覆盖 `tree/content/reference` 的零尺寸 callback 恢复
- `welcome_recent_pointer_clicked/moved/scrolled` 这轮也和其它 pane-local surface 对齐了：零尺寸 callback 时先保留 cached size，再回退 projection-backed `PaneSurfaceRoot` frame，而不是把已知有效的 recent-list viewport size 覆盖掉

这让 workbench shell 当前的真实输入链又更接近计划里的目标：Slint 负责承载和上传 pointer fact，pane 尺寸与 scroll authority 则回到 shared `UiSurface + UiPointerDispatcher + WorkbenchShellGeometry`。

同一轮又继续缩小了 root-shell 自己的 mixed-authority 面积：

- [`menu_pointer/build_host_menu_pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs) 已经删掉 legacy menu button frame 输入；顶层 menu button row 现在只从 shared `menu_bar_frame` 或 shared `shell_frame` 推导
- [`app/pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 不再回读任何 Slint `get_*_menu_button_frame()` getter，root menu popup 与 shared menu pointer bridge 只剩一份 projection-backed button-frame authority
- [`BuiltinWorkbenchRootShellFrames`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs) 现在额外导出 `Left/Right/BottomDrawerContentRoot`，所以 drawer-backed callback-size 不必再把整个 region shell 当成内容 viewport
- [`resolve_callback_surface_size_for_kind(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs) 现在优先吃这些 shared drawer content frame；`Hierarchy`、`Console` 以及同路径的 drawer asset tree/list/reference scroll surface 都先服从 shared content size，再回退 legacy `geometry.region_frame(...)`
- 新增的 real-host focused regressions 已经把这条边界锁住：
  - `root_hierarchy_pointer_move_prefers_shared_drawer_content_projection_over_stale_left_region_geometry`
  - `root_console_pointer_scroll_prefers_shared_drawer_content_projection_over_stale_bottom_region_geometry`

结果是：当前 root-shell 剩余的 boundary 已经不再包括 `legacy_menu_button_frames(...)` 或 host-page/document strip 这类专门桥接；主要收敛到 shared frame 缺席时的通用 geometry fallback，以及 dynamic floating-window 壳层仍通过统一 projection helper 从 editor-only layout geometry 派生 outer/tab/content frame。

## Root Shell Presentation Starts Consuming Shared Projection Frames

真实宿主这一轮又向前走了一步：shared template projection 不再只在 callback fallback 和 parity harness 里“旁路存在”，root shell 的 presentation 主链已经开始直接消费 builtin workbench 的 shared frame。

- [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在统一导出 `WorkbenchMenuBarRoot`、`WorkbenchBody`、`DocumentHostRoot`、`PaneSurfaceRoot`、`StatusBarRoot` 对应的 shared frame
- [`apply_presentation(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) 新增 `shared_root_frames` 输入；root shell 的 `center_band_frame` 和 `status_bar_frame` 已经直接取 shared projection，而不是继续完全依赖 `WorkbenchShellGeometry`
- `document_region_frame` 现在刻意保留分层 mixed authority：当 `Left/Right/Bottom` drawer region 全部折叠时，document zone 直接对齐 `DocumentHostRoot`；只要有可见 drawer region，就改为由 shared `WorkbenchBody` 提供 `x/y` 与总可用跨度，再由 legacy drawer extents 扣出 document zone，避免当前 transitional shell 在 drawer 打开时继续信任 stale `geometry.region_frame(Document)`
- visible drawer shell/header 这一层现在已经直接模板化并进入 shared root projection：
  - [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) 新增 `Left/Right/BottomDrawerShellRoot` 以及对应 `*DrawerHeaderRoot`
  - [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在会在 host recompute 时根据 chrome drawer extent 直接导出 visible drawer shell/header frame，不再只导出 body/menu/document/status 这组基础 root control
  - [`resolve_root_left_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs)、[`resolve_root_right_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 与 [`resolve_root_bottom_region_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 现在优先消费这些 shared drawer shell frame；visible drawer `document_region_frame` 也会从 shared left/right/bottom shell frame 扣减，而不是继续读 legacy main-axis extent
  - [`build_workbench_drawer_header_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs) 现在在 shared `*DrawerHeaderRoot` 存在时直接复用 header frame，所以 visible drawer header pointer surface 与 root shell presentation 真正合并到同一份 authority
  - focused regressions 现在直接锁住这条 cutover：`builtin_workbench_template_bridge_exports_visible_drawer_shell_and_header_frames_from_chrome`、`apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_extents`、`shared_drawer_header_pointer_layout_prefers_shared_root_projection_for_visible_drawer_regions`，以及 real-host `root_host_recomputes_builtin_template_bridge_with_visible_drawer_shell_and_header_frames`
- `viewport_content_frame` 也进入同一套 mixed authority：当 `Left/Right/Bottom` drawer region 全部折叠时，root shell 直接从 `PaneSurfaceRoot` 推导 viewport frame；`Scene` 会额外扣掉 toolbar 高度，`Game` 则直接使用 pane surface frame；只要 drawer 可见，viewport 就复用 resolved document frame，而不再保留一套独立的 legacy viewport geometry authority
- [`host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 的 root recompute 现在会把 `template_bridge.root_shell_frames()` 接进 `apply_presentation(...)`；同一函数在 child native window presenter 路径上显式传 `None`，因此 secondary native window 继续沿用 `configure_native_floating_window_presentation(...)` 的专用窗口边界
  - 同一条 recompute 主链现在会复用这份 resolved viewport frame 更新 `viewport_size` 与 `SharedViewportPointerBridge` 的 bounds，让 WGPU viewport 尺寸、shared pointer 命中范围和最终 presentation 看到的是同一份 frame
  - root main `document tab` strip 现在也开始沿用这套 mixed authority，而不是继续完全信任 `geometry.region_frame(Document)`：
  - [`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在额外导出 `DocumentTabsRoot`
  - [`build_workbench_document_tab_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs) 现在在 drawer region 全部折叠时优先消费 shared `DocumentTabsRoot` frame；只有 drawer 可见或 projection 缺席时才回退到 legacy geometry header strip
  - 这让 root-shell `document tab` pointer surface 与 `document_region_frame` / `viewport_content_frame` 保持同一份根级 authority，不再出现 presentation 已经按 shared projection 展示、但 `document tab` 命中根框仍被过窄 document geometry 截断的情况
  - root 左侧 `activity rail` pointer surface 这轮也接到了同一份 root-frame authority：[`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在额外导出 `ActivityRailRoot`，而 [`resolve_root_activity_rail_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) / [`build_workbench_activity_rail_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/activity_rail_pointer/build_workbench_activity_rail_pointer_layout.rs) / [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 会在 drawer 折叠时优先消费 shared rail frame、drawer 可见时继续回退 legacy left-region geometry
  - 对应的 real-host regression [`root_activity_rail_pointer_click_prefers_shared_projection_surface_when_left_region_geometry_is_stale`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 和 pure builder regression [`shared_activity_rail_pointer_layout_prefers_shared_root_projection_when_left_region_geometry_is_stale`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_activity_rail_pointer/layout_projection.rs) 共同锁住“left region 几何已经过期，但 shared `ActivityRailRoot` 仍正确”时的根壳命中行为
  - root `host page` strip 这轮也接到了同一份 root-shell authority，不过它的收口点是 root 宽度而不是 region frame：[`BuiltinWorkbenchTemplateBridge::root_shell_frames()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在额外导出 `WorkbenchShellRoot`，而 [`build_workbench_host_page_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_page_pointer/build_workbench_host_page_pointer_layout.rs) / [`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 现在会优先消费 shared shell width，而不是继续把根 strip 宽度锁死在 `TAB_MIN_WIDTH` 推出来的 metric estimate 上
  - 对应的 real-host regression [`root_host_page_pointer_click_prefers_shared_projection_shell_width_over_metric_strip_estimate`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 已经 focused green；pure builder / bridge 复跑则暂时会被邻接 `workbench.slint` 的 `UiAssetCanvasNodeData` 与 preview-surface Slint build-script 漂移抢先阻断，因此当前记账是“host-page 主链 green + broader unrelated blocker”
- 这一刀又把同一份 root-shell authority 从“展示和粗粒度 drag hit-test”继续推到“tab-drop 精确 attach anchor”：[`resolve_workbench_tab_drop_route_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/route_resolution.rs)、[`resolve_tab_drop_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/drop_resolution.rs) 和 [`strip_hitbox.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/strip_hitbox.rs) 现在会把 `WorkbenchBody` / `DocumentHostRoot` 的 resolved frame 继续喂给 tool/document tab strip hitbox，而不是让 precise drop target 重新退回 `geometry.center_band_frame` / `geometry.region_frame(Document)`
- [`workspace_docking.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/workspace_docking.rs) 现在在真实宿主 drop dispatch 前，把 `template_bridge.root_shell_frames()` 一起传给 tab-drop route 解析；这修掉了一个真实的 mixed-authority 缝：shared pointer route 已经落在正确的右侧 tool/document 区域，但 precise attach 仍可能因为旧几何的 tab strip `x/y` 漂移而退化成 coarse `anchor: None`
- [`ui/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/tests.rs) 直接锁住 `drawers collapsed -> consume projection frame` 与 `drawers visible -> document region keeps geometry` 两条真实宿主规则；[`template_bridge/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge/mod.rs) 则把 `root_shell_frames()` 的 control-frame 映射一起固定下来
- [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 现在再补一条 `root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed`；这条 regression 的真实 red 基线是 `host.viewport_size = 1600x876`，而 root shell 展示的 `viewport_content_frame = 1544x884`
- 这轮 focused rerun 现在已经直接给出绿色验证：`cargo test -p zircon_editor --lib apply_presentation_uses_shared_root_projection_frames_when_drawers_are_collapsed --locked -- --nocapture`、`cargo test -p zircon_editor --lib apply_presentation_keeps_geometry_document_region_when_drawers_are_visible --locked -- --nocapture`、`cargo test -p zircon_editor --lib builtin_workbench_template_bridge_recomputes_surface_backed_frames_with_shell_size --locked -- --nocapture`、`cargo test -p zircon_editor --lib root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed --locked -- --nocapture` 和 `cargo check -p zircon_editor --lib --locked`
- 新增这轮 focused green 还包括：`cargo test -p zircon_editor --lib apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_positions --locked -- --nocapture` 与 `cargo test -p zircon_editor --lib shared_drawer_header_pointer_layout_prefers_shared_root_projection_for_visible_drawer_regions --locked -- --nocapture`
- 新增的 red regression 是 [`root_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs) 里的 `resolve_workbench_tab_drop_route_uses_shared_root_projection_tab_strip_when_drawers_are_collapsed`：在 shared `WorkbenchBody` 已把右侧 tool strip 向上平移后，旧逻辑会错误地落成 `Drawer(RightTop) + anchor None`
- 同一轮又把 pure helper/parity 入口补齐了：[`resolve_workbench_drag_target_group_with_root_frames(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag/bridge.rs) 现在允许 `resolve_workbench_drag_target_group(...)` 这类纯函数路径显式吃到 `BuiltinWorkbenchRootShellFrames`，不会再比真实宿主 `WorkbenchShellPointerBridge` 落后一拍；对应 regression `resolve_workbench_drag_target_group_with_root_frames_uses_shared_root_projection_document_bounds_when_drawers_are_collapsed` 已经 focused green
- 当前验证状态因此更新为：生产代码已落地，`cargo check -p zircon_editor --lib --locked` 已恢复通过；但更宽的 `cargo test -p zircon_editor --lib slint_tab_drag --locked -- --nocapture` 仍会被邻接 `ui_asset` 单测源漂移抢先阻断，所以这里只按“focused green + broader unrelated blocker”记账

这意味着 root workbench shell 已经开始从“shared projection 只提供 callback/query 辅助”转向“shared projection 直接喂给 presentation”。虽然当前 drawer/document shell 还是混合状态，但后续 clip/z-order 继续进入真实宿主时，接线位置已经明确，不需要再从零发明第二套 presentation authority。

## Startup Welcome Session

### Session Authority

启动现在不再默认伪造一个 `sandbox-project` 和假 world。编辑器先由 `EditorManager::resolve_startup_session()` 解析结构化启动会话，再决定进入：

- `EditorSessionMode::Project`
- `EditorSessionMode::Welcome`

最近工程配置统一落在 `editor.startup.session`，至少包含：

- `last_project_path`
- `recent_projects`

每次启动都会重新验证最近工程，而不是把上一次的验证结果当权威缓存。失效工程会继续保留在 recent list 中，并在 Welcome 页上显示诊断标签。

### Welcome Page Contract

Welcome 页不是独立 launcher，而是 workbench 内的 exclusive page：

- `editor.welcome` descriptor 由 `EditorManager` 注册并托管
- `EditorStartupSessionDocument` 负责 recent list、draft 和状态消息
- `WelcomePaneSnapshot` 只承载展示数据
- Slint `workbench/welcome.slint` 负责 JetBrains 风格的 Recent Projects + New Project 双栏界面

当前 Welcome 页主操作固定为：

- 打开 recent project
- 移除 recent entry
- 创建 `Renderable Empty` 目录式项目
- 按 `Project Name + Location` 推导现有目录并打开

这一轮又把 Welcome 页的 callback 语义入口固定成 builtin template authority，而不再直接把 `on_welcome_*` 回调名当协议：

- [`startup_welcome_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml) 定义 `WelcomeSurface/*` 稳定命名空间
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 用 `BuiltinWelcomeSurfaceTemplateBridge` 把 `control_id + event_kind + arguments` 解析成 typed `WelcomeHostEvent`
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 继续执行既有 `startup_session` 生命周期，但不再自己解析 callback 参数语义

因此现在 Welcome 页已经实现了“视觉仍由 [`welcome.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/welcome.slint) 提供，control 语义由模板权威提供”的分层。

Welcome recent list 这一轮又进一步收紧了宿主 ABI：[`welcome.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/welcome.slint) 和 [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 已经不再暴露 `open_recent_project(path)` / `remove_recent_project(path)` 直连 callback。最近工程列表现在只上传 pointer 坐标和滚轮 delta，再由 shared `UiSurface + UiPointerDispatcher` 推导出 stable `WelcomeSurface/OpenRecentProject` 或 `WelcomeSurface/RemoveRecentProject`。

真实宿主路径上，这条链又补了一层 projection-backed 几何兜底：

- [`BuiltinWorkbenchTemplateBridge::control_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/bridge.rs) 现在允许 host 直接读取 `PaneSurfaceRoot` 的 shared frame
- 当 [`welcome_recent_pointer_clicked(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/welcome_recent_pointer.rs) / move / scroll 收到的 Slint `width/height` 仍然是零时，[`pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 会回退到这份 shared pane frame，而不是把 welcome recent surface 视为不可命中
- 这意味着 recent list 在真实 root host 上的命中与滚动，现在既不依赖直连业务 callback，也不依赖 Slint 宿主必须同步上传有效宽高；shared template/layout projection 已经成为第二层权威尺寸来源

这一点由 [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 里的 `root_welcome_recent_pointer_click_uses_projection_fallback_in_real_host` 明确锁住，防止后续 real-host 时序再把 welcome recent 退回到 zero-size no-op。

### Renderable Empty Template

Welcome 创建的新工程不再只是 manifest 骨架，而是立即可打开、可渲染的目录式模板。`EditorProjectDocument::create_renderable_template(...)` 负责脚手架生成，固定写出：

- `zircon-project.toml`
- `assets/scenes/main.scene.toml`
- `assets/materials/default.material.toml`
- `assets/shaders/pbr.wgsl`
- `library/`

创建完成后，host 统一走 `create_project_and_open -> apply_startup_session` 链路，更新 recent list、恢复 layout、替换 runtime world，并关闭 Welcome exclusive page。

## Visual System

### Shell Character

Hybrid Shell 的视觉基调固定为：

- 冷灰底色，连续 IDE 壳体，不做圆角卡片式内嵌窗口
- 蓝色焦点高亮，用于活动 tab、rail 激活态、主按钮和输入焦点
- icon-first rail + 扁平 tab strip，而不是大块按钮矩阵或胶囊 chip
- 中央 Scene/Game 文档区面积最大，明确体现“编辑器优先”
- Inspector 与 Console 走高密度属性表 / 输出面板观感，不做大号占位卡片

HTML 原型在 `docs/editor-and-tooling/prototypes/editor-workbench-hybrid-shell.html` 中作为视觉 oracle。当前运行时已经切到 Slint host；运行时代码只负责跟随这套壳体语义，不把某个宿主实现细节反向定义成视觉规范。

### Stable Shell Chrome IDs

壳层稳定控件命名空间固定为：

- `WorkbenchMenuBar/*`
- `ActivityRail/*`
- `ToolWindow/*`
- `DocumentTabs/*`
- `InspectorView/*`
- `ViewportToolbar/*`
- `StatusBar/*`

这里的 `*` 代表具体 control id。HTML 原型和未来 Slint 组件必须映射到同一套稳定 id，避免“原型一套命名、宿主一套命名、headless 测试第三套命名”。

对 pane/exclusive surface，这一轮也已经固定了两组稳定命名空间：

- `AssetSurface/*`
- `WelcomeSurface/*`

## Builtin Hybrid Preset

### Default Startup Preset

source-controlled builtin preset 固定为：

- left `Project / Assets / Hierarchy`
- center `Scene / Game`
- right `Inspector`
- bottom `Console`

当前内置比例会刻意压低底部 Console、收窄左右 drawer，让中心 viewport 在默认启动时保持主导。

这个 preset 只定义 builtin startup shell，不定义用户最终布局。真正的布局恢复优先级仍然是：

1. project layout
2. global layout
3. builtin Hybrid preset

### Persisted Layout Authority

`zircon_editor/src/ui/workbench/layout/mod.rs` 现在只保留结构入口，真实 schema 声明分散在 `zircon_editor/src/ui/workbench/layout/*.rs`，但 `WorkbenchLayout` 仍然是唯一持久化 schema。
本轮没有引入第二套 HTML-only 或 Slint-only 布局格式。

因此：

- 用户把 `Project` 拖到右边，语义不变，只是 host slot 变了
- 用户把 `Console` 挪到左边，空状态和内容不变
- project/global layout 覆盖 builtin preset 时，仍然走现有 `LayoutManager` 恢复链路

### Named Layout Preset Assets

除了 startup builtin preset，workbench 现在还支持“用户命名 preset”：

- 打开项目后，preset 保存到 `assets/editor/layout-presets/<sanitized-name>.workbench-layout.json`
- preset 文件内部保留原始 `preset_name`，文件名只负责安全落盘
- 未打开项目时，preset 回退到用户配置键 `editor.workbench.presets`
- `EditorManager` 会先枚举项目 preset asset，再并入 global preset 名单，最后在 shell 中渲染可加载列表

这意味着“把 Project 固定放右边”这类偏好不再需要写死进 startup preset；用户可以把它保存成项目资产，让团队项目带着自己的默认壳体走。

## Current Drag And Drop Scope

当前 Slint shell 已经支持真实 pointer-driven tab drag/drop，但范围刻意收窄到“壳体组级别”：

- document tabs 可以拖到 `left / right / bottom / document`
- tool-window header tabs 也可以拖到同样四个宿主组
- drop 到 drawer side 时，会优先落到该侧当前活跃/可见的 drawer slot；如果该侧没有活跃 stack，则回退到 canonical slot
- drop 到 `document` 时，会优先落到当前 active workbench page；若当前主页面不是 workbench，则回退到第一个 workbench page
- document 区还额外支持更细粒度的 split edge：
  - `document-left`
  - `document-right`
  - `document-top`
  - `document-bottom`
  这些 edge key 由 shared `WorkbenchShellPointerBridge` 直接产出，并会在 drop 时归一成 `LayoutCommand::CreateSplit`

这一层已经足够支持用户把 `Project`、`Inspector`、`Console`、`Scene/Game` 在默认壳体内重新收纳，而不需要先保存 preset 再看结果。

当前仍未支持的拖放目标是：

- exclusive activity page promote target

这些更细粒度目标仍然保留在 layout/drag model 中，但 Slint 壳体暂时还没有把它们全部暴露为可视 drop zone。

这一轮之后，floating target 的状态需要拆开看：

- `WorkbenchViewModel` 已经显式暴露 `floating_windows`
- 每个 floating window 都带有 `focused_view` 和按 `WorkspaceTarget::FloatingWindow` 标注的 tab 列表
- `tab_drag` 现在也支持稳定 fallback key：
  - `floating-window/<window_id>`
  - `floating-window-edge/<window_id>/<edge>`
- 这些 key 会归一成：
  - `ViewHost::FloatingWindow(window_id, path)`
  - `LayoutCommand::CreateSplit { workspace: WorkspaceTarget::FloatingWindow(window_id), ... }`

这一轮又往前推进了一刀，真实 Slint shell 已经开始消费这份数据：

- `workbench.slint` 现在声明了 `FloatingWindowData`
- `slint_host/ui.rs` 会把 `WorkbenchViewModel.floating_windows` 投成可绑定的浮窗模型，并显式携带 shared solver 下发的 `frame`、`target_group` 和每条 edge 的 route key
- root shell 已经会渲染最小浮窗 overlay card：
  - 标题
  - tab strip
  - 活动 pane 的 `PaneSurface`
- 浮窗几何 authority 也已经不再留在 `workbench.slint`：
  - `FloatingWindowLayout.frame` 负责 editor-only 持久化
  - `WorkbenchShellGeometry.floating_window_frames` 负责运行时默认生成与 clamp
  - `workbench.slint` 只消费投影后的 `window.frame.*`

因此“浮窗 attach/split 语义不存在”已经不成立；现在连真实宿主消费也已经进入主链。当前这层已经额外固定了两个 shared-core 约束：

- `WorkbenchShellPointerBridge` 的 drag surface 会直接产出：
  - `FloatingWindow(window_id)`
  - `FloatingWindowEdge { window_id, edge }`
- document edge split 热区只落在真实 `Document` region frame 上，不再吞掉 empty bottom/right dock band，因此空工具区仍然能稳定命中 `Bottom` / `Right` dock target

当前仍未完成的是：

- 更完整的跨窗口窗口化宿主
- detach/promote 等更深一层 window host 语义
- 与多原生窗口同步的 capture/focus 生命周期

当前这层 pointer route 已经进一步收口：

- `SlintEditorHost` 现在持有单一 `WorkbenchShellPointerBridge`
- bridge 内部把 drag/drop route 和 splitter resize route 都建模成 shared retained surface，而不是再信任宿主 fallback 字符串
- drag surface 负责 `DragTarget / DocumentEdge / FloatingWindow / FloatingWindowEdge`
- resize surface 继续单独维护 capture，避免 splitter 拖拽被动态 drag 节点污染
- `WorkbenchShellPointerRoute` 现在同时承载 `DragTarget(...)`、`DocumentEdge(...)` 和 `Resize(...)`
- [`tab_drag.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/tab_drag.rs) 现在额外提供 `workbench_shell_pointer_route_group_key(...)`，把 shared route 统一归一成稳定宿主 key
- tab drop 不再由 `app.rs` 直接混合 pointer group 解析与 attach/reopen 命令，而是先归一到 `ResolvedWorkbenchTabDropRoute`，再交给统一 `dispatch_tab_drop(...)`
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 现在直接复用这份 helper，而不是自己重新拼 `document-*` / `floating-window*` 字符串
- Slint drag overlay 的 badge 与 document edge 高亮现在也直接消费 shared route 对应的稳定 key，而不是只剩一个 coarse `"document"` 状态

这意味着当前 editor shell 的 tab drop 已经具有一条明确的 shared-core-first 主链：

- pointer 坐标
  -> `WorkbenchShellPointerBridge`
  -> `WorkbenchShellPointerRoute::DragTarget / DocumentEdge`
  -> `workbench_shell_pointer_route_group_key(...)`
  -> `resolve_workbench_tab_drop_route(...)`
  -> `dispatch_tab_drop(...)`
  -> typed `LayoutCommand`

为了锁住这条链路，compatibility harness 现在也开始抓旧壳体浮窗 overlay 的 shared frame / route key 摘要：

- [`EditorUiCompatibilityHarness`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/harness.rs) 会记录 `floating-window/<window_id>=frame`
- 同时记录 attach 和四条 edge 的稳定 route key
- overlay snapshot 现在还会记录 `floating-window/<window_id>.focus_target_id`，把浮窗 header click 最终要 focus 到哪个 `ViewInstanceId` 也固定成 parity 数据
- [`ui.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui.rs) 里的浮窗 presentation 测试会直接校验这些值与 shared route helper 等价

这意味着现在不仅 shared bridge 会产出正确 route，连旧 Slint 壳当前真实消费到的 floating route key 也被基线化了，后续继续 cutover 时更容易发现 parity 回退。

## Current Non-Menu Transient Overlay Scope

把当前 shell 里真正还活着的 non-menu popup/dialog/transient overlay 盘完之后，inventory 已经比较明确：

- menu popup 仍然是唯一带 dismiss overlay 语义的 transient surface，这条链已经由 [`menu_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/menu_pointer/mod.rs) 完整收口到 shared `UiSurface + UiPointerDispatcher`
- menu 之外，当前 workbench shell 没有第二个 standalone popup/dialog/modal dismiss surface
- 当前唯一需要单独命中的 non-menu transient overlay，是 floating-window chrome header；它属于持久化 workspace host 的头部命中层，不属于 dismissible popup/dialog

这一块现在也已经进入 shared route authority：

- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 的浮窗 overlay 只上传 `floating_window_header_pointer_clicked(x, y)`
- [`callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 只把这条 pointer fact 接回宿主
- [`app/workbench_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/workbench_pointer.rs) 会先走 [`shell_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/shell_pointer.rs) 的 shared route 解析
- 当 route 命中 `WorkbenchShellPointerRoute::FloatingWindow(..)` 或 `FloatingWindowEdge { .. }` 时，再由 [`callback_dispatch/layout/floating_window/dispatch.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs) 收口成 `LayoutCommand::FocusView`
- [`FloatingWindowModel`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/model/mod.rs) 现在提供唯一的 `focus_target_instance()` / `focus_target_tab()` 语义，projection 和 dispatch 都复用它
- focus 目标 fallback 已经固定并由测试锁住：存在于 tab 集中的 `focused_view -> active tab -> first tab`；stale `focused_view` 不再把 overlay presentation 和 runtime focus dispatch 撕成两套结果

因此当前这条分支的真实结论不是“还有一批未迁移的 dialog/dismiss overlay”，而是“现有 shell 里真正活着的 non-menu transient overlay hit surface 只剩浮窗 header，而且它已经进入 shared route authority；后续新增 dialog/popup surface 必须直接落在同一条 shared seam 上”。

## Current Splitter Scope

当前 Slint shell 已经支持 root-captured drawer splitters：

- left / right / bottom 三个可见 stack 都提供真实 pointer-driven resize
- splitter 释放事件在 shell 根级捕获，不依赖原始 handle 命中区域
- side resize 会把同一侧的全部 drawer slot extent 同步写回，避免 `LeftTop/LeftBottom` 或 `RightTop/RightBottom` 切换时宽度跳变
- 保存 preset asset 时，这些 extent 会直接落入现有 `WorkbenchLayout`

本轮进一步把“当前 pointer 命中的是哪一个 splitter”也迁到了 shared core：

- Slint `begin_drawer_resize(...)` 不再上传 `target_group` 字符串，只上传 pointer 坐标
- host 通过 `WorkbenchShellPointerBridge` 在 `UiSurface + UiPointerDispatcher` 上解析 `WorkbenchShellGeometry.splitter_frames`
- splitter `Down` 会在 shared dispatcher 内触发 capture，后续 `Move / Up` 即使移出 splitter hit bounds 也会继续回到原 target
- `left / right / bottom` 目标组现在和 tab drag 的 dock target 一样，都是同一棵 host-owned shared surface 的 route 结果，而不是 Slint 本地命中真源

同一轮里，tab drag overlay 也继续复用这条 shared-core-first shell pointer path。这样 drag target 高亮和 splitter resize 都不再需要各自维护分散的命中真源。

## Current Menu And Popup Input Scope

menu / popup 这条链路现在也已经进入 shared-core-first 主链，而不再停留在旧 Slint callback 直连：

- [`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 现在只负责组合 [`HostMenuChrome`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 与其他主壳 surface；顶层 menu visual 已经不再由 root bootstrap 自己持有
- host 在 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) / [`app/pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 里只维护 shared `HostMenuPointerBridge` 与 grouped [`HostMenuStateData`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)，不再回读或回写 Slint `*_menu_button_frame` getter/setter
- bridge 自己维护一棵 shared `UiSurface`，里面显式放置：
  - top menu button 节点
  - dismiss overlay 节点
  - popup surface 节点
  - popup item 节点
- [`build_host_menu_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs) 会优先把 builtin `root_shell_frames()` 展开成六段 top-level menu button frame；[`HostMenuChrome`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint) 自己持有对应的 popup anchor visual，popup presentation 和 shared hit-test 都只再依赖同一份 projection-backed menu-state authority
- [`app/pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pointer_layout.rs) 现在只把 `open/hovered/scroll/popup-height` 汇总成 `WorkbenchHostContext.menu_state`，不再向 `WorkbenchShell` 同步 control-specific menu button frame property；这条 seam 已经从 root 壳体 ABI 收缩成纯 menu-state DTO
- `Window` 菜单 popup 现在通过 shared `ScrollableBox` 维护 `offset / viewport_extent / content_extent`，宿主只消费 `window_menu_scroll_px` 和 popup 高度
- dismiss overlay 点击现在不会清空 shared `popup_scroll_offset`；只有重新 `open_popup(...)` 时才重置滚动起点，因此 close/reopen 与真实宿主 replay 都继续复用同一份 canonical scroll state
- click 事件不再由 Slint menu row 直接决定 action，而是走：
  坐标
  -> `UiSurface` hit-test
  -> `UiPointerDispatcher`
  -> `dispatch_shared_menu_pointer_click(...)`
  -> template-aware / legacy-fallback menu dispatcher
  -> `EditorEventRuntime`

因此这条链路当前已经做到：

- top menu open/close 由 shared hit-test 决定
- popup item 选择由 shared route 决定
- popup 外点击关闭由 dismiss overlay 节点决定
- `Window` popup 的滚轮输入由 shared scroll state 决定
- popup `x/y` anchor 也已经服从 host-projected menu frame，而不是留在 local Slint geometry

这让 menu/popup/scroll 和 dock target / splitter 一样，都不再把 Slint 当命中与事件真源；Slint 只保留表现层宿主职责。

## Current Structural Tab And Rail Input Scope

menu/popup 之外，workbench shell 本身最常用的结构性标签输入现在也已经进入 shared-core-first 主链：

- `ActivityRail`
- host page strip
- document tab strip
- floating-window tab strip
- left/right/bottom drawer header tab strip

这几类输入现在分别由：

- [`activity_rail_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs)
- [`host_page_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_page_pointer/mod.rs)
- [`document_tab_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs)
- [`drawer_header_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs)

建立 focused shared surface，然后统一走 [`callback_dispatch/shared_pointer/`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer) 子树里的 `dispatch_shared_*` helper；[`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 只保留 `mod/pub use` 结构边界。

宿主边界现在被刻意压缩成两类输入事实：

- strip-local pointer 坐标
- 必要的视觉几何输入，例如当前 tab 的 `x/width`

其中 host/document/drawer 这三类 strip 刻意区分了两种触发时机：

- `ActivityRail` 和 host page strip 使用 `pointer_pressed(...)`
- document/floating tabs 与 drawer header tabs 使用 click-level `pointer_clicked(...)`

这样做不是风格偏好，而是为了保持交互语义正确：

- rail 和 host page 没有拖拽负担，按下即可进入 shared hit-test
- document 与 drawer header tab 同时承担 drag/drop 入口，必须复用 Slint 现有 click suppression，避免拖拽起手误发激活、关闭或 drawer toggle

这一刀之后，真实宿主已经移除了最后一批结构性直接点击回调真源：

- 不再依赖 `activate_host_page(page_id)`
- 不再依赖 `activate_document_tab(tab_id)`
- 不再依赖 `close_tab(tab_id)`
- 不再依赖 `toggle_drawer_tab(slot, tab_id)`

取而代之的是：

坐标与局部几何
-> shared `UiSurface`
-> `UiPointerDispatcher`
-> shared route
-> builtin template binding / typed fallback
-> `EditorEventRuntime`

因此现在 workbench shell 的结构性标签输入已经和 menu/popup、dock target、list/scroll 一样，进入了同一类 shared pointer authority，而不是继续让 Slint callback 名称承载业务语义。

## Current Asset List Input Scope

asset workspace 这一层现在也已经不只是 tree/content 两块 shared-pointer authority：

- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 现在为 `activity` 和 `browser` 两套 asset surface 同时维护：
  - folder tree shared pointer state
  - content list shared pointer state
  - `references` list shared pointer state
  - `used_by` list shared pointer state
- [`asset_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/asset_pointer.rs) 现在除了 tree/content bridge，还新增 `AssetReferenceListPointerBridge`
- [`assets.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint) 的 `ReferenceListView` 不再依赖本地 `ScrollView + TouchArea` 作为事件真源，而是改成 host-authoritative 的：
  - `hovered_index`
  - `scroll_px`
  - `pointer_clicked / moved / scrolled`

reference list 当前也已经和 tree/content 一样走同一条 shared-core-first 主链：

坐标
-> `UiSurface` hit-test
-> `UiPointerDispatcher`
-> `AssetPointerReferenceRoute`
-> `dispatch_shared_asset_reference_pointer_click(...)`
-> stable `AssetSurface/ActivateReference`
-> `EditorEventRuntime`

因此 asset shell 里当前已经进入 shared dispatcher 的非菜单列表输入包括：

- activity asset folder tree
- activity asset content list
- activity asset references list
- activity asset used-by list
- browser asset folder tree
- browser asset content list
- browser asset references list
- browser asset used-by list

这些区域现在都不再把 Slint 行级 `TouchArea` 当命中和滚动真源；Slint 只负责承载 visual tree 和显示 host 回灌状态。

这一轮又把这层边界从“行为上主要走 shared dispatcher”收紧成“ABI 上只允许 shared dispatcher”：

- hierarchy pane 不再暴露 `hierarchy_select(node_id)` 直连 callback
- asset pane 不再暴露 `select_folder` / `select_asset` / `activate_reference` 直连 callback
- workbench root 和 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 也同步移除了对应 forwarder 与 `ui.on_*` 注册

现在 hierarchy、welcome recent、asset tree/content/reference 这些列表面统一只保留 pointer/scroll 输入上传和 host 状态投影，不再给 Slint 业务树留第二条直接改 runtime 的旁路。这一点由 [`surface_contract.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_list_pointer/surface_contract.rs) 里的 authority regression test 固定下来。

同时，welcome recent 已经先走通了“zero-size Slint callback -> shared projection pane frame fallback”的真实宿主链路。这说明这些列表面的 shared authority 现在不只是 route/binding 真源，也开始接管 real-host 场景下的有效 pane geometry 兜底。

asset workspace 这一轮还把 header / utility / import 这层的非 pointer ABI 一起收口了：

- [`asset_panes.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/asset_panes.slint) 现在直接承接 `ProjectOverviewPane`、`AssetsActivityPane` 和 `AssetBrowserPane`；[`assets.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint) 只保留这些 pane 复用的 DTO/leaf surface
- `ProjectOverviewPane`、`AssetsActivityPane` 和 `AssetBrowserPane` 不再暴露 `search_edited(...)`、`kind_filter_changed(...)`、`view_mode_changed(...)`、`utility_tab_changed(...)`、`locate_selected()`、`import_model()` 这类业务 callback
- leaf surface 现在统一只上传：
  - `control_changed(control_id, value)`
  - `control_clicked(control_id)`
- [`pane_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_surface.slint) 现在承接这组 generic route：
  - `asset_control_changed(source, control_id, value)`
  - `asset_control_clicked(source, control_id)`
- [`app/callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 现在对这组 ABI 只注册 `pane_surface_host.on_asset_control_*`；asset/welcome/viewport-toolbar 这类 pane callback 已不再保留 root `ui.on_*` 注册
- [`app/assets.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/assets.rs) 只负责把稳定 `control_id` 翻译成 builtin asset template 所需的 `UiEventKind + arguments`
- `ProjectOverviewPane` 的 “Asset Browser” 按钮现在也复用 `asset_control_clicked("project", "OpenAssetBrowser")`，不再保留单独 `open_asset_browser()` 宿主 ABI
- `mesh_import_path_edited(...)` 仍然保留在 draft binding 主链，因为它属于 live draft 输入，而不是 `asset_surface_controls.ui.toml` 当前覆盖的 builtin template control

因此 asset shell 现在在宿主边界上明确分成两类稳定入口：

- pointer/scroll 事实继续走 shared `UiSurface + UiPointerDispatcher`
- 非 pointer control 只上传稳定 `control_id`，再由 builtin template bridge 解释成 typed payload

## Current Scroll-only Pane Input Scope

在 asset tree/list 之外，editor shell 里几块“主要只需要 wheel/scroll authority”的 pane surface 也已经继续并入同一条 shared-core-first 路线：

- [`detail_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/detail_pointer/mod.rs) 现在统一承载 `ScrollSurfacePointerBridge` 的结构入口，bridge 声明、dispatch/route/state、scroll 行为和 console/asset-details/inspector layout helper 已按功能拆进子模块
- [`scroll_surface_host.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/scroll_surface_host.rs) 现在统一承载 scroll-only surface 的宿主 `bridge + state + size`
- 同一类 bridge 已覆盖：
  - `ConsolePane`
  - asset browser `SelectionDetailsRail`
  - `InspectorPane`
- host 在 [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 中只同步 `pane_size + content_extent + scroll_state`
- wheel 输入先进入 shared `UiSurface + UiPointerDispatcher`
- Slint 最终只消费 `*_scroll_px`

这一轮里最关键的变化是不再留下任何非-menu pane 的 Slint `ScrollView.viewport-y` 真源：

- [`InspectorPane`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/panes.slint) 先一步改成 clipped content stack + host-driven `scroll_px`
- [`ConsolePane`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/panes.slint) 现在也改成 retained text stack + clip + shared `scroll_px`
- asset browser [`SelectionDetailsRail`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/assets.slint) 同样去掉了本地 `ScrollView`，只再消费 shared scroll state 回灌的 `scroll_px`

因此 shared core 现在真正成为这些 pane 的唯一滚动 authority：

- host 只维护 `pane_size + content_extent + scroll_offset`
- shared dispatcher 先更新 canonical `UiScrollState`
- Slint 只负责 clip 和按 `scroll_px` 平移内容，不再拥有独立 viewport state

同一轮里，[`callback_dispatch/viewport/snap_cycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/snap_cycle.rs) 的 helper 可见性也已经收口到 `viewport` 子树内可见，避免 [`route_mapping.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/viewport/route_mapping.rs) 在 shared viewport dispatcher 主链上再次被模块边界问题阻塞。

因此当前 shell 里已经进入 shared scroll authority 的非-menu pane surface 包括：

- `Console`
- asset details rail
- `Inspector`

## Current Viewport Overlay Input Scope

viewport 现在除了 toolbar command 和外框 pointer capture 之外，连 Scene 内部 overlay hit-test 也继续往 shared core 收口：

- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 里的 `SharedViewportPointerBridge` 仍然负责 outer viewport frame 的 bounds / capture / scroll 到 `ViewportCommand` 的映射
- 进入 [`SceneViewportController`](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/controller/mod.rs) 之后，handle overlay、scene gizmo pick shape、renderable 候选会被同步进 [`ViewportOverlayPointerRouter`](/E:/Git/ZirconEngine/zircon_editor/src/scene/viewport/pointer/mod.rs)
- overlay router 内部会建立最小 `UiSurface`，把 candidate frame、z-order 和 viewport-node `UiPointerDispatcher` 放进 shared retained route
- route 结果固定收口为 `ViewportPointerRoute::{HandleAxis, SceneGizmo, Renderable}`
- 当前优先级固定为 `HandleAxis > SceneGizmo > Renderable`，因此 handle hover/drag 继续压过 gizmo 和普通 renderable 选择
- Slint host 和 [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 现在不再拥有 gizmo/handle/renderable hit-test 真源；它们只负责呈现 viewport 纹理、上传 pointer 坐标和显示 chrome

这意味着 editor shell 里“最复杂的一块局部命中语义”也开始和 menu、dock target、asset list 一样走 shared `UiSurface + UiPointerDispatcher`，而不是为 viewport 再保留一份完全独立的本地 picking 体系。

## Responsive AutoLayout Solver

这一轮开始，Slint runtime 不再把 `left/right/bottom extent` 当作 UI 侧锚点公式的输入真源。壳体现在明确分成两层：

- `WorkbenchLayout` 继续只描述 drawer/document/floating window 拓扑、extent 和持久化 override；它不是底层布局基础类型的 owner。
- 共享 `zircon_ui` 提供真正的约束与几何权威：`AxisConstraint`、`ResolvedAxisConstraint`、`BoxConstraints`、`UiSize`、`UiFrame` 和 `solve_axis_constraints(...)`。
- editor `autolayout` 只负责根据真实窗口尺寸、`WorkbenchLayout`、descriptor 默认约束、layout/view override，以及当前活动 tab，把 workbench 语义映射到共享求解器，解出 shell 几何。
- Slint `WorkbenchShell` 只消费 frame 结果并渲染，不再自己推导 `document_zone_x`、`right_stack_x`、`bottom_panel_y`。

当前共享求解结果模型是 `WorkbenchShellGeometry`，其中固定包含：

- `window_min_width` / `window_min_height`
- `region_frames`
- `splitter_frames`
- `viewport_content_frame`

在 editor 侧：

- `PaneConstraints` 现在只是 `zircon_ui::BoxConstraints` 的别名
- `ShellFrame` / `ShellSizePx` 分别复用 `zircon_ui::UiFrame` / `zircon_ui::UiSize`
- `StretchMode`、`AxisConstraint`、`ResolvedAxisConstraint` 与 runtime UI 使用同一套实现，不再允许 editor 保持一份平行定义

当前 host 更新链路也固定为：

1. Slint 导出真实 `shell_width_px` / `shell_height_px`
2. host 监听尺寸变化并设脏 `WindowMetrics`
3. layout/preset/tab/drag/splitter 改动设脏 `Layout`
4. `recompute_if_dirty()` 统一调用 `compute_workbench_shell_geometry(...)`
5. 求解后的 frame 和最小窗体尺寸回灌到 Slint
6. `viewport_content_frame` 再送入 `ViewportInput::Resized`

### Constraint Semantics

当前轴向约束语义已经固定为：

- 这些语义现在由 `zircon_ui` 定义并测试，editor 只提供 region/content 默认值和 override
- `min == 0` 表示无最小值
- `max == -1` 表示无最大值
- `preferred` 是显式字段，drawer `extent` 作为主轴 preferred 覆盖
- 放大时只让 `Stretch` 项吸收空间，优先级高者先分配，同优先级再按 weight 分配
- 缩小时所有仍高于 `min` 的项都可压缩，但低优先级先缩，同优先级再按 weight 分摊

默认上：

- `Document` 保持更高优先级和更高权重
- 左/右/底工具区维持中优先级
- 顶栏、宿主栏、状态栏走固定尺寸

### Runtime Resize Behavior

splitter 现在不再在 Slint 里直接计算最终 extent。当前行为改成：

- pointer down 时 host 记录活动 region、起始指针位置和基础 preferred
- pointer move 只更新内存中的 transient preferred，并立刻重算 frame
- pointer up 才把结果提交回 `drawer.extent`

这样带来的结果是：

- 右侧 `Inspector` 和底部 `Console` 的锚点跟随 solver，而不是跟随旧公式
- 拖拽过程中 splitter 命中区与显示边界一致
- viewport 尺寸来自求解后的内容区，而不是从 Slint pane 反推
- 窗体最小尺寸来自所有 region/host chrome 的聚合结果，不能再被缩到把文档区挤没

这也让 editor shell 和未来 runtime screen-space UI 可以共享同一套主轴约束求解规则；不同的只剩节点库、宿主语义和 editor-only docking 模型，而不是两份各自演进的布局数学。

## Pane Catalog

### Tool Windows

下列 pane 属于 tool-window family，可以驻留任何 drawer stack：

- `Project`
- `Assets`
- `Hierarchy`
- `Inspector`
- `Console`

它们的 content kind 现在由 `ViewContentKind` 显式建模，而不是靠固定 slot 推断。

### Document Pages

文档区固定包含两类长期文档 pane：

- `Scene`
- `Game`

并支持按需创建：

- `Prefab Editor`

`Prefab Editor` 不在 startup layout 中创建占位 tab。只有真的打开 prefab 时才出现实例和标签。

## Close Rules

关闭规则固定为：

- `Scene` 不可关闭
- `Game` 不可关闭
- `Prefab Editor` 可关闭
- tool windows 默认不走 document-style close；显示/隐藏和激活通过 `DockCommand`/rail/menu 驱动

当前实现已经在 `EditorManager::close_view` 对 `editor.scene` 和 `editor.game` 做了 non-closeable 保护，并有测试覆盖。

## Empty State Ownership

空状态属于 pane，不属于 slot。也就是说：

- `Project` 挪到右边，仍显示 `No project open`
- `Hierarchy` 从左边换到 document host，仍然是 hierarchy 的空状态
- `Console` 换到别的位置，也仍然只显示 console 自己的空状态

这一轮里，empty-state 和 pane-level action 的宿主 ABI 也一起收紧了：

- [`pane_surface_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml) 现在提供 builtin `PaneSurface/TriggerAction`
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 根壳体只上传 `pane_surface_control_clicked(control_id, action_id)` 这一条 generic control route
- [`pane_surface_actions.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs) 再把它交给 `BuiltinPaneSurfaceTemplateBridge`

因此 Scene/Game empty-state 的 `Open Scene / Create Scene`，以及 Project pane 里的 `Open Assets`，现在都不再通过 handwritten `menu_action(action_id)` callback 直接越过模板/runtime 边界。

### Canonical Empty States

固定文案如下：

- `Project / Assets`
  - title: `No project open`
  - primary action: `Open Project`
  - `Recent Projects` 只保留在菜单，不做大按钮墙
- `Hierarchy`
  - no project: `No scene loaded`
  - project open but no nodes: `No nodes in scene`
- `Scene`
  - no project: `No project open`
  - project open but no active scene: `No active scene`
- `Prefab Editor`
  - 不渲染预占位空 tab
- `Inspector`
  - `Nothing selected`
  - 不显示一堆禁用输入框
- `Console`
  - 无输出时显示最近一次状态或 `No output yet`

当前 `WorkbenchViewModel` 已经输出 `PaneEmptyStateModel`，并且 Slint host 直接消费这套空状态语义。
当前壳体实现中，tool window 空状态使用顶部锚定的紧凑消息样式；中央 `Scene/Game` welcome state 继续保持 document-centered。

### Runtime Startup Selection Rule

运行时 startup 现在明确区分两件事：

- shell 仍然可以带着 renderable default level 启动，以保证 viewport / renderer 链路稳定
- 但 `EditorState::new` 必须在 no-project welcome 模式下主动清空 selection

这条规则的直接结果是：

- `Inspector` 在无项目启动时稳定显示 `Nothing selected`
- `Scene/Game` welcome state 不会再因为默认 camera/cube 选中而退化成可编辑状态
- 需要“默认就有选中对象”的测试或编辑路径，必须显式走 `EditorState::with_default_selection(...)`

## Backend-Neutral View Model

`zircon_editor/src/ui/workbench/model/mod.rs` 现在作为结构入口，把 menu/host strip/status bar 视图模型以及 tool/document/floating-window projection 拆到 `workbench/model/` 目录下，并额外输出：

- `tool_windows: BTreeMap<ActivityDrawerSlot, ToolWindowStackModel>`
- `document_tabs: Vec<DocumentTabModel>`
- `PaneTabModel`
- `PaneEmptyStateModel`
- `PaneActionModel`

本轮模块边界整治后，几个核心根入口都只保留结构职责：

- `zircon_editor/src/ui/binding_dispatch/mod.rs` 只保留 dispatch root wiring，animation/selection/asset/welcome/draft/inspector/docking/viewport 逻辑全部下沉到子目录
- `zircon_editor/src/core/host/manager/layout_hosts/mod.rs` 只保留 layout host bookkeeping wiring，active tab 查找、document host 遍历、workbench root 修复、builtin shell layout repair 分别落在独立脚本
- `zircon_editor/src/core/host/manager/builtin_views/mod.rs` 只保留 builtin descriptor wiring，activity view、activity window、welcome、UI asset editor descriptor 组装不再平铺在单一文件
- `zircon_editor/src/ui/workbench/model/mod.rs` 只保留 view-model 导出，菜单、host strip、document tab、empty state、floating window focus helper 分别落在独立脚本
- `zircon_editor/src/ui/workbench/reflection/mod.rs` 只保留 reflection root wiring，descriptor build、activity collection、route registration、typed default command 映射分成独立模块

这让宿主层不需要再根据固定 left/right/bottom 假定 pane 类型，而是可以直接读取：

- 哪个 slot 有哪些 pane
- 当前 active tab 是谁
- 某个 pane 是否 closeable
- 某个 pane 当前该显示正常内容还是空状态

### Scene Viewport Chrome Projection

这一轮又把 Scene 文档页的 viewport chrome 从“基础 toolbar 可见性”推进到真实状态投影：

- `EditorChromeSnapshot` 直接携带 `scene_viewport_settings`
- `slint_host/ui.rs` 把它投影成 `PaneData.viewport: SceneViewportChromeData`
- `workbench.slint` 的 `SceneViewportToolbar` 把 Scene-only 的紧凑分组控件渲染到 pane surface

当前 Scene chrome 语义固定为：

- 左上：`Drag / Move / Rotate / Scale`、`Local / Global`、display mode、grid mode、snap、lighting、skybox、Gizmos、frame selection
- 右上：`Perspective / Orthographic` 和六向轴 `+/-X/Y/Z`
- `Game` pane 不显示这组 editor-only toolbar，保持 scope 只在 `Scene`

宿主侧的命令权威也已经前移了一层：

- [`scene_viewport_toolbar.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml) 现在定义稳定 `ViewportToolbar/*` control/binding 契约
- [`template_runtime/runtime.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/runtime.rs) 会把 `scene.viewport_toolbar` 注册成 builtin projection
- [`callback_dispatch/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs) 用 `BuiltinViewportToolbarTemplateBridge` 把 `control_id + event_kind + arguments` 解析成 typed binding
- [`viewport_toolbar_pointer/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/viewport_toolbar_pointer/mod.rs) 现在持有 `ViewportToolbarPointerBridge`，先用 shared `UiSurface + UiPointerDispatcher` 做 toolbar 命中，再把 stable route 交给 template binding 解释
- [`app.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app.rs) 已经删掉本地 viewport toolbar `parse_*` 主链，不再把 `tool/space/projection/display/grid` 当作手写字符串协议
- [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 已经移除 `viewport_set_*` / `viewport_frame_selection` 直连 callback 声明与转发，只保留 `viewport_toolbar_pointer_clicked(...)` 这一条 pointer-fact 输入

同一条 real-host seam 这一轮又继续把“toolbar 自己的 surface 宽度到底听谁的”收到了 shared authority：

- [`app/viewport.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/viewport.rs) 里 root document 的 `viewport_toolbar_surface_size(...)` 现在不再直接信任 `WorkbenchShellGeometry::region_frame(Document).width`
- 当 `Left/Right/Bottom` drawer region 折叠、root projection 已经给出更宽的 `PaneSurfaceRoot` / resolved viewport frame 时，真实宿主会优先用 shared root projection 结果重算 [`BuiltinViewportToolbarTemplateBridge`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs)
- [`app/tests.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests.rs) 里的 `root_viewport_toolbar_pointer_click_prefers_shared_projection_surface_width_over_stale_document_geometry` 直接锁住这条 seam：即使 legacy document geometry 被压窄到 `800px`，root host 里的 `align.neg_z` 仍然必须通过 shared projection 宽度命中并派发 `EditorViewportEvent::AlignView`

也就是说，当前 Scene viewport toolbar 的视觉仍然由 `workbench.slint` 渲染，但真实命中与命令解释都已经切到 shared pointer route + template-runtime authority，而不是留在 Slint host 的局部 parser 或 legacy callback ABI。

同一轮里，`SceneViewportToolbar` 组件自身残留的 `set_tool` / `set_projection_mode` / `frame_selection` 等 legacy callback 壳声明也已经删掉。现在这块 chrome 在 Slint 侧只保留 pointer-fact 输入，不再保留任何未接线的旧 ABI 名字。

## Fixtures And Prototype Contract

`zircon_editor/fixtures/workbench/` 下的 fixture 现在对齐 builtin Hybrid preset：

- `default-layout.json`
- `view-descriptors.json`
- `view-instances.json`
- `editor-data.json`

这些 fixture 现在表达的是“startup shell 已经准备好，但还没有打开项目”的状态：

- `project_open = false`
- `Scene/Game` tab 已存在
- `Project/Assets/Hierarchy/Inspector/Console` pane 已存在
- `Project` 为左侧 active pane
- `Inspector` 默认右侧打开
- `Console` 底部打开

HTML 原型使用 fixture-shaped data 渲染 builtin preset，并提供 `Project docks right` 的 alternate preset，证明 pane placement 来自 layout JSON，而不是 DOM 写死。

## HTML Skeleton And Slint Mapping

当前 Slint 宿主的组件边界固定为：

- `WorkbenchShell`
- `WorkbenchTopBar`
- `ActivityRail`
- `ToolWindowStack`
- `DocumentWorkspaceHost`
- `WorkbenchStatusBar`
- pane components for `Project`, `Assets`, `Hierarchy`, `Inspector`, `Console`, `Scene`, `Game`, `Prefab Editor`

映射原则：

- Material components 只提供 base controls
- theme layer 负责把它们压到 JetBrains-like 视觉系统
- pane content 读取 `WorkbenchViewModel`，不直接绑定固定 slot
- dock tree 继续由 `WorkbenchLayout` 驱动，不额外引入 Slint-side 布局 schema

## Runtime Status

当前运行时宿主已经切到 Slint：

- `zircon_editor/src/ui/slint_host/app.rs` 负责启动 shell、绑定菜单/标签/选择/视口事件
- `zircon_editor/src/ui/slint_host/ui.rs` 现在只保留 Slint adapter 边界；`apply_presentation.rs` 负责把 grouped host DTO 写回 generated globals/setters
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/` 负责把 `WorkbenchViewModel + chrome snapshot` 组合成窗口级 presentation
- `zircon_editor/src/ui/widgets/common/tabs.rs` 负责 tab strip 的可复用控件投影
- `zircon_editor/src/ui/slint_host/viewport/mod.rs` 负责把共享 `wgpu` 纹理导入到 Slint `Image`
- `zircon_editor/ui/workbench.slint` 与 `zircon_editor/ui/workbench/chrome.slint` 提供 JetBrains-like shell chrome 与 pane surface

asset workspace 在 host 内的刷新边界也已经收紧：

- `AssetDetailsRefreshRequested` 只在选中资产上下文变化时触发，host 只查询 `asset_details(uuid)`
- `AssetPreviewRefreshRequested` 只在 visible set 或 preview surface 变化时触发，host 只对当前可见资产请求 preview 刷新
- `sync_asset_workspace()` 仍保留为 project open/save/import 这类显式后端同步入口，不再由搜索、filter、list/thumbnail、utility tab、普通选择事件触发

这条边界让 workbench shell 的 presentation tick 和 asset/resource backend sync 解耦，避免 asset panel 上展示的 runtime `resource_revision` 因为纯 UI 交互出现无意义漂移。

## UI Asset Editor Document Host

`UI Asset Editor` 这一轮已经作为真正的 workbench document pane 接进当前宿主，而不是停在 source-only sidecar：

- [`ui_asset_editor_pane.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_pane.slint) 已进一步收口成 orchestration shell，只继续持有 pane-level draft/session state、palette drag overlay 和对子 owner 的 wiring；`UiAssetEditorPaneData` 与 shared DTO 移到 [`ui_asset_editor_data.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_data.slint)，shared widgets 与 `UiAssetSourceTextInput` 移到 [`ui_asset_editor_components.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_components.slint)，Designer Canvas + Source column 移到 [`ui_asset_editor_center_column.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_center_column.slint)，Inspector 与 Stylesheet surface 分别移到 [`ui_asset_editor_inspector_panel.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_inspector_panel.slint) 和 [`ui_asset_editor_stylesheet_panel.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/ui_asset_editor_stylesheet_panel.slint)
- [`pane_fields.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/pane_fields.slint) 继续只承接 `CompactField/AxisField` 这种跨 pane 共享字段件；`Open Ref`、style state buttons、source cursor roundtrip 和 external drag projection 已不再要求 root `UiAssetEditorPane` 单文件持有
- [`UiAssetEditorSession`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/session.rs) 维护 `last_valid_document + preview surface + source buffer + UiDesignerSelectionModel` 四份 canonical state；Hierarchy、Canvas、Inspector 和 Source block 的选中都通过稳定 `node_id` 协调，不再依赖瞬时树索引或文本光标
- [`undo_stack.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset/undo_stack.rs) 已从 source-only snapshot 升级为 `source + selection` snapshot；tree edit redo 会一起恢复 `inspector_selected_node_id`、`source_selected_block_label` 和 excerpt，而不是只恢复 TOML 文本
- Palette 插入已经真正落成 native node 或 imported widget reference node 创建；Canvas authoring 则直接暴露 `insert child / insert after / wrap / unwrap / reparent into previous / reparent into next / outdent`
- detached child window 里的 pane-local `ui_asset_*` callback 现在会回指 source window session，因此 floating document host 不会绕开 manager/session authority
- Source parse 失败时，pane 会保留最后一个 valid preview，并通过 `source_roundtrip_status` 暴露结构化错误和 roundtrip 状态

这轮针对 workbench 宿主重新确认过的 focused 验证包括：

- `cargo test -p zircon_editor --test workbench_slint_shell --offline -- --nocapture`
- `cargo test -p zircon_editor --lib slint_window --offline -- --nocapture`
- `cargo test -p zircon_editor --lib slint_tab_drag --offline -- --nocapture`
- `cargo test -p zircon_editor --lib slint_drawer_resize --offline -- --nocapture`
- `cargo test -p zircon_editor --lib apply_presentation_ --offline -- --nocapture`
- `cargo test -p zircon_editor --test workbench_window_resize --offline -- --nocapture`
- `cargo check -p zircon_graphics --lib --offline`
- `cargo test -p zircon_editor --lib --locked ui_asset_editor_`
- `cargo test -p zircon_editor --lib --locked editor_manager_runs_ui_asset_`
- `cargo test -p zircon_editor --lib --locked editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo`
- `cargo test -p zircon_editor --lib --locked tests::host::slint_window::child_window_callback_wiring_tracks_source_window_for_pane_interactions`
- `cargo test -p zircon_editor --test workbench_slint_shell --locked ui_asset_editor_`
- `cargo test -p zircon_editor --locked --quiet --test workbench_slint_shell --test workbench_slint_ui_asset_authoring_shell --test workbench_slint_ui_asset_theme_shell`
- `cargo test -p zircon_editor --locked --quiet ui_asset_editor_owner_extracts_data_helpers_and_panels_out_of_root_file`

这一轮已经完成的是：

- Slint-only runtime path，`zircon_editor` / `zircon_app` 不再依赖 `iced`
- builtin Hybrid preset fixture 与 config-driven HTML prototype
- project-aware layout preset assets with config fallback
- Slint `Window` menu preset entry for save/load/reset
- Slint pointer-driven tab drag/drop across shell host groups
- floating workspace projection in `WorkbenchViewModel` and typed fallback drop-route normalization
- minimal floating workspace overlay host in real `workbench.slint`
- floating-window overlay header pointer route now resolves through shared `WorkbenchShellPointerBridge` and `dispatch_builtin_floating_window_focus(...)`
- floating-window overlay projection and focus dispatch now share the same canonical `focus_target_instance()` fallback, and overlay parity snapshots now carry `focus_target_id`
- same-fixture parity 现在已经补到 dock target normalized route 结果和 floating-window focus journal：
  - shared pointer route 与 fallback group key 会通过同一份 `capture_resolved_tab_drop_route_snapshot(...)` 产出等价 route snapshot
  - `dispatch_builtin_floating_window_focus(...)` 与直接 `LayoutCommand::FocusView` 现在会通过同一份 `capture_event_journal_delta_snapshot(...)` 对比 journal delta
- manager 侧的 native-window bookkeeping seam 也已经落下：
- [`window_host_manager.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/window_host_manager.rs) 现在维护 `NativeWindowHostState { window_id, handle, bounds }`
- [`workspace_state.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/workspace_state.rs) 会在 `recompute_session_metadata()` 里把 `WorkbenchLayout.floating_windows` 同步到这份记录
- [`layout_commands.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/layout_commands.rs) 会在最后一个 detached view 重新 attach 后清掉对应 window host 记录
- [`layout_hosts/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/layout_hosts/mod.rs) 现在把 active tab 查找、instance host 收集、workbench document root 修复和 builtin shell layout repair 拆成目录化子模块，`workspace_state.rs` 不再反向依赖聚合实现细节
- [`EditorManager::native_window_hosts()`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/workspace_state.rs) 和 [`native_window_hosts.rs`](/E:/Git/ZirconEngine/zircon_editor/tests/native_window_hosts.rs) 现在把 detach/restore/bootstrap 的 manager 态锁成公开可复用 seam
- 真实 secondary native presenter 已经开始消费这条 seam：
  - [`app/native_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/native_windows.rs) 现在维护 `NativeWindowPresenterStore`，会 create/update/hide secondary `WorkbenchShell`
  - target 收集会把 [`EditorManager::native_window_hosts()`](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/workspace_state.rs) 和 [`resolve_floating_window_outer_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 合并；当 manager 账本里的 bounds 还是零时，会回退到 shared floating-window outer frame，而不是生成 1x1 假窗口
  - [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 新增 `native_floating_window_mode`，同一套 `WorkbenchShell` 现在既能渲染主壳，也能把指定 floating window 投成 full-window child host
  - [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 现在会在创建 secondary shell 时复用同一份 [`wire_callbacks(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs)，让 child host 进入同一条 shared dispatcher / template dispatch 主链，而不是停在仅 presentation-only
- native-mode [`workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 现在把 floating document tab activate/close、header focus click 走 `WorkbenchHostContext`，而 `PaneSurface` 的 pointer/scroll/control/ui-asset 回调继续走 `PaneSurfaceHostContext`；child shell 不再自己声明这组 workbench-specific root callback ABI
- 这一条 native-mode seam 最近又继续 generic 化：[`host_surface.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_surface.slint) 现在先组装 `native_floating_surface_data`，再交给 [`HostNativeFloatingWindowSurface`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench/host_components.slint)；tab strip/header touch/pane body 不再从 wrapper 直接消费 `native_window_bounds/header_height/floating_windows` 这些 scalar ABI
  - [`presenter_store.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_window/presenter_store.rs) 和 [`native_mode.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_window/native_mode.rs) 现在额外锁住 presenter creation hook 和 native-mode callback forwarding 契约
  - [`floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 现在统一导出 floating-window `outer/tab/content` frame，并且会优先吃 non-zero native host bounds；[`floating_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs)、[`document_tab_pointer/build_workbench_document_tab_pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs)、[`shell_pointer/drag_surface.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs)、[`app/native_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/native_windows.rs)、[`app/viewport.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/viewport.rs) 和 [`app/helpers.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs) 不再各自重新决定“geometry 还是 host bounds”
  - [`app/callback_wiring.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/callback_wiring.rs) 现在也会给 child-window 的 `hierarchy/asset tree/content/reference` move 回调加上 `with_callback_source_window(...)`，所以 hover/move 零尺寸 callback 终于能和 click/scroll 一样回收到 projected floating-window content frame
  - 这轮新增 focused regressions `floating_window_projection_prefers_host_bounds_when_present`、`shared_document_tab_pointer_layout_prefers_native_window_host_bounds_for_floating_strip` 和 `shared_shell_pointer_route_prefers_native_window_host_bounds_for_floating_attach_surface`，把 presenter target、floating tab strip、drag attach surface 三条 consumer 统一锁到同一份 host-bounds-aware projection
- Slint pointer-driven left/right/bottom splitter resize
- drag overlay move/up and splitter/full-screen resize capture now upload only `workbench_drag_pointer_event(...)` / `workbench_resize_pointer_event(...)`; route resolution and final dispatch are owned by `WorkspaceShellPointerBridge + workspace_docking.rs`, not direct Slint business callbacks
- Scene-only viewport compact toolbar/right-top group，直接投影 `SceneViewportSettings`
- Scene viewport toolbar pointer semantics now route through shared `UiSurface + UiPointerDispatcher`
- Scene viewport toolbar callback semantics now route through builtin template bindings instead of host-local string parsers
- layout-agnostic pane empty states
- document closeability rules
- WGPU27 共享纹理 viewport bridge
- `chrome.slint` 图标源目录固定到 `zircon_editor/assets/icons/ionicons/`，不再直接引用 `dev/`

仍未完成的是：

- same-fixture shell parity 继续向更多 real-host shell snapshot 和 projection consumption 扩展；dock target route results 与 floating-window focus journals 已经进入 parity harness
- secondary native window presentation 已经落地，而且 child-window 的 document tab、header focus 与 pane-local pointer/scroll/control 回调已经接回 shared host；native window 的 close request 现在会在“窗口内全部 tab 都可关闭”时统一转成 shared `CloseView` 序列，并在布局里真的移除该 floating window 后允许窗口关闭。当前仍未完全迁完的是 pointer capture/focus 细节，以及包含 non-closeable tab 时更细的 close UX
- `SlintUiHostProjection` 的 root frame consumption 已经进入真实 `slint_host` presentation 主链：`center/status` 已直接吃 shared projection，`document` 在 drawer 折叠时也会对齐 `DocumentHostRoot`。仍未完成的是更完整的 clip/z-order 消费，以及 drawer 可见状态下继续把 document/pane shell 从 legacy geometry 迁到 shared authority
- 未来如果新增 dialog / popup / modal surface，必须直接落在 shared `UiSurface + UiPointerDispatcher`，不能回退到 host-local hit-test / dismiss
- Scene toolbar 的 snap 当前是 preset cycle 交互，而不是自由数值输入
- 持续把 Slint 视觉细节逼近 HTML 原型

## Visible Drawer Root Document Authority

root shell 当前又减少了一层 visible-drawer 混合权威：

- [`root_shell_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 现在会在 visible left/right/bottom drawer 存在时重建 root `document_region_frame`
- rebuilt frame 的 authority 分工是固定的：
  - shared `WorkbenchBody` 提供 root `x/y` 与总可用跨度
  - legacy visible drawer geometry 只继续提供 `width/height` extents
- [`ui/apply_presentation.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) 因此不再把 visible drawer 下的 root document shell 继续挂在 stale `geometry.region_frame(Document)` 上
- [`resolve_root_document_tabs_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 也复用了这份 resolved document frame，所以 root document-tab strip 与 root document shell 不再出现两套 `x/y/width` authority
- [`resolve_root_viewport_content_frame(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/root_shell_projection.rs) 现在也复用了同一份 resolved document frame，所以 root viewport content 在 visible drawer 下不再保留一套独立的 stale geometry `x/y/width/height`
  - 同一轮里，drawer shell/header 自身已经不再保留 “shared origin + legacy main-axis extent” 过渡桥：
  - left/right drawer 的 `width` 与 bottom drawer 的 `height` 现在都由 builtin template/runtime 直接投影到 shared root frame
- focused regressions 也继续覆盖这条收口链：`apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_positions`、`shared_activity_rail_pointer_layout_prefers_shared_visible_drawer_regions_when_cross_axis_geometry_is_stale`，以及 [`root_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_tab_drag/root_projection.rs) 里的 visible-drawer right/bottom strip route regressions

这一步已经把 drawer shell/header 直接模板化，并把 visible drawer 主轴 extent 从 root presentation / drawer-header pointer 的 legacy geometry 读取中拔掉；当前剩余的显式 fallback boundary 主要收敛到 dynamic floating-window frame authority：consumer 已经统一吃 host-bounds-aware `floating_window_projection` helper，但最底层 outer frame 仍来自 editor-only `WorkbenchLayout`/`WorkbenchShellGeometry` 与 manager 账本，而不是 builtin template/runtime 直接导出的 shared floating-window frame bundle。

## Latest Floating-Window Projection Bundle

这一刀把 dynamic floating-window 的 authority 再往前推了一层：真实宿主不再让 presentation、pointer hit-test、callback size fallback 和 native presenter target 各自临时拼 `geometry + native_window_hosts`，而是在每次 `recompute_if_dirty()` 时先产出一份 dedicated `FloatingWindowProjectionBundle`。

- [`floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 现在固定导出：
  - `FloatingWindowProjectionFrames { outer_frame, tab_strip_frame, content_frame, host_frame, native_host_present }`
  - `FloatingWindowProjectionBundle`
  - `build_floating_window_projection_bundle(...)`
- [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 会在 root host 重算时一次性构建这份 bundle，并缓存到 `SlintEditorHost::floating_window_projection_bundle`
- [`document_tab_pointer/build_workbench_document_tab_pointer_layout.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/document_tab_pointer/build_workbench_document_tab_pointer_layout.rs)、[`shell_pointer/drag_surface.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs)、[`floating_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs)、[`app/helpers.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/helpers.rs)、[`app/viewport.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/viewport.rs) 和 [`app/native_windows.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/native_windows.rs) 现在优先只吃这份 bundle；旧 helper 只保留 defensive fallback，不再是生产路径的第一真源
- real-host 回归 [`app/tests/floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs) 已经锁住 “detach child window 后 host recompute 确实缓存 bundle entry” 这条 seam，避免后续又退回 per-consumer stitching

这一步的结果是：floating-window mixed-authority seam 已经从 “每个 consumer 自己合并 geometry / host bounds” 收窄成 “host 重算时统一产出 bundle，但 bundle 的 outer authority 仍来自 editor-owned layout geometry 与 native host bookkeeping”。下一条才是把这个 outer authority 再继续推进到更纯的 shared projection producer。

## Latest Floating-Window Outer Authority

这一刀继续往前推的是 floating window 最底层 outer authority，而不是再加 presentation 侧补丁：

- [`sample_floating_window_chrome()`](/E:/Git/ZirconEngine/zircon_editor/src/tests/workbench/view_model/support.rs) 现在把 floating window 的 non-default `requested_frame` 明确投进 [`WorkbenchViewModel`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/model/workbench_view_model.rs)；这意味着 Hybrid Shell 的 floating-workspace 输入不再只存在于 `WorkbenchLayout` 和 geometry helper 里
- [`floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs) 现在把 recompute-time producer 顺序改成：
  - native host bounds
  - shared floating-window source（`requested_frame + DocumentHostRoot + WorkbenchBody`）
  - legacy geometry fallback
- [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 先把 shared-source-resolved outer frame 回灌到 native-window bookkeeping，再构建 `FloatingWindowProjectionBundle`；presentation、hit-test、callback size fallback 与 presenter targeting 终于共享同一个 base outer authority
- [`app/tests/floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs) 也不再接受 “bundle outer frame == geometry frame” 这个旧假设，而是要求 child-window cached bundle 跟 shared-source-resolved outer frame 一致

这说明当前 Hybrid Shell 剩余的 floating-window boundary 已经不再是“consumer 读谁”；真正剩下的是要把 `DocumentHostRoot + WorkbenchBody` 这层 root-shell-derived source 彻底替换掉。

## Latest Floating-Window Dedicated Template Source

这一刀已经把上面这条 boundary 落地切掉：

- [`floating_window_source.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml) 新增 dedicated builtin template，单独建模 `FloatingWindowCenterBandRoot` 与 `FloatingWindowDocumentRoot`
- [`callback_dispatch/template_bridge/floating_window_source/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/mod.rs) 现在只保留 structural wiring；[`bridge.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs) 提供 `BuiltinFloatingWindowSourceTemplateBridge`，[`source_frames.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/floating_window_source/source_frames.rs) 提供 frame bundle owner
- [`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 在 real host `recompute_if_dirty()` 里先刷新这份 bridge，再把它作为 floating-window source 输入传给 [`floating_window_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/floating_window_projection.rs)
- 现在 dynamic floating-window 的 base authority 顺序固定为：
  - native host bounds
  - dedicated shared floating-window source
  - legacy geometry fallback

结果是：`DocumentHostRoot + WorkbenchBody` 已经不再承担 floating-window outer-frame producer 角色；root shell projection 继续服务 document/tab/drawer/menu 等根壳层，但 floating-window 默认/钳制 source 已经有了独立的 builtin template/runtime authority。


