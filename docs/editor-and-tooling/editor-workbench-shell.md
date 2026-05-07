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
  - zircon_editor/src/ui/slint_host/hierarchy_pointer/constants.rs
  - zircon_editor/src/ui/slint_host/hierarchy_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/welcome.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/world_space_submission.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/redraw.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/geometry.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/slint_host/app/build_export_actions.rs
  - zircon_editor/src/ui/slint_host/app/build_export_actions/output_folder.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/module_plugins.rs
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
  - zircon_editor/src/ui/slint_host/app/module_plugin_actions.rs
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
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_binding.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/pane/surface_control.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/workbench/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/common/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/ui/workbench/model/menu/extension_menu.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/dispatcher.rs
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
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/drawer_toggle.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/document_tab.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/main_page.rs
  - zircon_editor/src/ui/template_runtime/runtime.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/slint_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/layouts/views/welcome.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/build_export.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml
  - zircon_editor/src/ui/slint_host/viewport/mod.rs
  - zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/core/host/manager.rs
  - zircon_editor/src/core/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/mod.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/ensure_shell_instances.rs
  - zircon_editor/src/ui/host/layout_hosts/repair_builtin_shell_layout.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/build_export_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
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
  - zircon_editor/src/ui/workbench/model/menu/view_menu.rs
  - zircon_editor/src/ui/workbench/fixture/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/src/ui/workbench/reflection/name_mapping.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/ui/workbench/startup/mod.rs
  - zircon_editor/src/ui/workbench/startup/display_project_path.rs
  - zircon_editor/src/ui/workbench/startup/editor_session_mode.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs
  - zircon_editor/src/ui/workbench/state/mod.rs
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
  - zircon_editor/assets/ui/editor/material_meta_components.ui.toml
  - zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml
  - zircon_editor/assets/ui/editor/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_menu_chrome.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
  - zircon_editor/assets/ui/editor/welcome.ui.toml
  - zircon_editor/assets/ui/editor/workbench_dock_header.ui.toml
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/src/tests/host/slint_detail_pointer/mod.rs
  - zircon_editor/src/tests/host/slint_list_pointer/
  - zircon_editor/assets/icons/ionicons/folder-open-outline.svg
  - zircon_editor/assets/icons/ionicons/git-branch-outline.svg
  - zircon_editor/assets/icons/ionicons/git-network-outline.svg
  - zircon_editor/assets/icons/ionicons/refresh-outline.svg
  - zircon_editor/assets/icons/ionicons/save-outline.svg
  - zircon_editor/assets/icons/ionicons/share-outline.svg
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
  - zircon_editor/src/tests/host/slint_window/shell_window.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - zircon_editor/src/tests/host/slint_window/native_material_painter.rs
  - zircon_editor/src/tests/host/slint_window/native_template_text.rs
  - zircon_editor/src/tests/host/slint_window/native_viewport_image.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs
  - zircon_editor/src/tests/host/slint_inspector_template_body.rs
  - zircon_editor/src/tests/host/template_runtime/inspector_surface.rs
  - zircon_editor/src/tests/host/template_runtime/viewport_toolbar.rs
  - zircon_editor/src/tests/host/template_runtime/pane_surface_controls.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/surface/render/text_measure.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
  - zircon_editor/src/ui/slint_host/app/tests/floating_window_projection.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/binding_dispatch/inspector/apply.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/slint_host/app/inspector.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/build_export.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/module_plugins.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/diagnostics_overlay.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/geometry.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
implementation_files:
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/core/editing/state/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/hierarchy_pointer/constants.rs
  - zircon_editor/src/ui/slint_host/hierarchy_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/slint_host/host_contract/redraw.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/surface_frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/geometry.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/slint_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
  - zircon_editor/src/ui/slint_host/app/build_export_actions.rs
  - zircon_editor/src/ui/slint_host/app/build_export_actions/output_folder.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/module_plugins.rs
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
  - zircon_editor/src/ui/slint_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_binding.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/pane/surface_control.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/workbench/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/common/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/ui/workbench/model/menu/extension_menu.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/tests/host/slint_menu_pointer/dispatcher.rs
  - zircon_editor/src/ui/slint_host/app/viewport.rs
  - zircon_editor/src/ui/slint_host/event_bridge.rs
  - zircon_editor/src/ui/template_runtime/runtime.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
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
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/mod.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/dispatch.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/resolution.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/layout/floating_window/tests.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/shell_pointer/drag_surface.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/slint_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/build_export.rs
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
  - zircon_editor/assets/ui/editor/host/build_export_desktop_body.ui.toml
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/ui/workbench/panes.slint
  - zircon_editor/ui/workbench/welcome.slint
  - zircon_editor/assets/icons/ionicons/folder-open-outline.svg
  - zircon_editor/assets/icons/ionicons/git-branch-outline.svg
  - zircon_editor/assets/icons/ionicons/git-network-outline.svg
  - zircon_editor/assets/icons/ionicons/refresh-outline.svg
  - zircon_editor/assets/icons/ionicons/save-outline.svg
  - zircon_editor/assets/icons/ionicons/share-outline.svg
  - zircon_editor/src/tests/host/slint_list_pointer/
  - zircon_editor/src/core/host/manager.rs
  - zircon_editor/src/core/host/manager/layout_hosts/mod.rs
  - zircon_editor/src/core/host/manager/builtin_views/mod.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_layout/ensure_shell_instances.rs
  - zircon_editor/src/ui/host/layout_hosts/repair_builtin_shell_layout.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/build_export_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
  - zircon_editor/src/core/host/manager/layout_commands.rs
  - zircon_editor/src/core/host/manager/window_host_manager.rs
  - zircon_editor/src/core/host/manager/workspace_state.rs
  - zircon_editor/src/ui/workbench/layout/mod.rs
  - zircon_editor/src/ui/workbench/model/mod.rs
  - zircon_editor/src/ui/workbench/model/menu/view_menu.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/src/ui/workbench/reflection/name_mapping.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/ui/workbench/startup/editor_session_mode.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs
  - zircon_editor/src/ui/workbench/state/mod.rs
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
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/binding_dispatch/inspector/apply.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/slint_host/app/inspector.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/build_export.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/module_plugins.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/slint_window/shell_window.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - zircon_editor/src/tests/host/slint_window/native_template_text.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/surface/render/text_measure.rs
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
  - .codex/plans/ZirconEngine Unity 式编辑器优先补齐计划.md
  - docs/superpowers/specs/2026-05-03-editor-core-usable-loop-design.md
  - docs/superpowers/plans/2026-05-03-editor-core-usable-loop.md
  - user: 2026-05-04 fix exported editor executable run-loop exit
  - user: 2026-05-04 exported editor executable shows white native window
  - user: 2026-05-04 inspect abnormal exported editor display with startup diagnostics
  - user: 2026-05-04 continue exported editor display repair as data-driven renderer then authored template renderer
  - user: 2026-05-04 exported editor native UI still shows text bars, no interaction, and layout drift
  - docs/superpowers/plans/2026-05-04-editor-host-data-driven-template-renderer.md
  - user: 2026-05-05 exported editor native screenshot shows structural labels and layout overlap after glyph renderer
  - user: 2026-05-05 exported Rust-owned editor native UI buttons unresponsive and menu/content occlusion follow-up
  - user: 2026-05-05 exported Rust-owned editor native UI shows missing drawer/status shell after project workspace restore
  - user: 2026-05-05 native mouse movement must not reload UITOML or force full redraw
  - user: 2026-05-05 native mouse events must produce visible UI state changes
  - user: 2026-05-05 native asset tree hover must visibly repaint without a full frame update
  - user: 2026-05-05 current editor UI layout calculation issue
  - user: 2026-05-05 exported Rust-owned editor native UI buttons must size from text plus padding
  - user: 2026-05-05 P2 hit-test the full viewport toolbar
  - user: 2026-05-05 P2 Gate viewport toolbar clicks to primary press
  - user: 2026-05-05 SVG/Image components, SVG icons, Material UI, and top-right debug refresh-rate overlay must stay on the .ui.toml chain
  - user: 2026-05-07 editor visual density B selection for smaller icons, rail, menu, and inspector controls
  - .codex/plans/Editor 绘制与鼠标事件优化计划.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - .codex/plans/Material UI 元组件与 .ui.toml 编辑器布局 Slate 化计划.md
  - user: 2026-05-06 native editor GUI regression: text input, chrome hit targets, tab drag, drawer/page clicks, and rebuild log pressure
  - docs/superpowers/plans/2026-05-06-ui-debug-reflector-full-closure.md
tests:
  - zircon_ui/src/tests/shared_core.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - zircon_editor/tests/workbench_drag_targets.rs
  - zircon_editor/tests/native_window_hosts.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs
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
  - zircon_editor/src/tests/host/slint_menu_pointer/dispatcher.rs
  - zircon_editor/src/tests/host/slint_tab_drag/
  - zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/layout/mod.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/workbench/template_bridge.rs
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/slint_event_bridge/mod.rs
  - zircon_editor/src/tests/host/slint_window/
  - zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
  - cargo test -p zircon_editor --lib native_host_generic_template_text_field_routes_commit_binding_on_enter --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib shared_menu_pointer_click_dispatches_editor_operation_payloads_from_extension_menu_items --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - zircon_editor/src/tests/host/slint_window/native_viewport_image.rs
  - cargo test -p zircon_editor --lib native_host_painter_draws_template_svg_image_pixels --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib rust_owned_host_painter_resolves_runtime_svg_image_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_draws_top_right_debug_refresh_rate --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - 2026-05-06 editor chrome SVG continuation: cargo check -q -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-svg-icons (passed with existing warnings)
  - 2026-05-06 editor chrome SVG continuation: cargo test -q -p zircon_editor --lib page_and_dock_tabs_project_svg_icons_and_close_button_icon --locked --jobs 1 --target-dir D:\cargo-targets\zircon-svg-icons -- --nocapture (passed)
  - 2026-05-06 editor chrome SVG continuation: cargo test -q -p zircon_editor --lib rust_owned_host_window_snapshot_renders_template_icon_states --locked --jobs 1 --target-dir D:\cargo-targets\zircon-svg-icons -- --nocapture (passed)
  - 2026-05-06 editor chrome SVG continuation: cargo test -q -p zircon_editor --lib rust_owned_host_painter_resolves_runtime_svg_image_assets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-svg-icons -- --nocapture (passed)
  - 2026-05-06 editor chrome SVG continuation: cargo test -q -p zircon_editor --lib activity_rail_nodes_project_tab_svg_icons_and_selected_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-svg-icons -- --nocapture (passed)
  - 2026-05-06 editor menu SVG continuation: cargo test -q -p zircon_editor --lib menu_popup_nodes_project_action_svg_icons --locked --jobs 1 --target-dir D:\cargo-targets\zircon-svg-icons -- --nocapture (passed)
  - 2026-05-06 editor menu SVG continuation: cargo test -q -p zircon_editor --lib menu_popup_nodes_project_absolute_rows_beyond_authored_slots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-svg-icons -- --nocapture (passed)
  - 2026-05-06 editor menu SVG continuation: cargo fmt -p zircon_editor -- --check (passed)
  - 2026-05-06 editor menu SVG continuation: cargo check -q -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-svg-icons (passed with existing warnings)
  - 2026-05-06 GUI regression slice: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never (passed with existing warnings)
  - 2026-05-06 GUI regression slice: cargo test -p zircon_editor --lib native_host_welcome_material_text_field_accepts_keyboard_input --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 GUI regression slice: cargo test -p zircon_editor --lib native_host_welcome_material_button_routes_welcome_callback --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 GUI regression slice: cargo test -p zircon_editor --lib native_host_document_tab_drag_releases_capture_and_forwards_drop --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 GUI regression slice: cargo test -p zircon_editor --lib native_host_repeated_hierarchy_hover_moves_do_not_rebuild_presentation --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 GUI regression slice: cargo test -p zircon_editor --lib fallback_page_chrome_preserves_clickable_tab_and_project_path_frames --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 GUI regression slice: cargo test -p zircon_editor --lib fallback_dock_header_preserves_tab_drag_and_close_hit_frames --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 GUI regression slice: cargo test -p zircon_editor --lib tab_chrome_fallback_detects_zero_height_or_zero_width_hits --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 viewport fast path slice: cargo test -p zircon_editor --lib native_host_viewport_button_and_scroll_wait_for_viewport_image_repaint --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 viewport fast path slice: cargo test -p zircon_editor --lib native_host_viewport --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 viewport fast path slice: cargo test -p zircon_editor --lib viewport_without_native_repaint --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never -- --nocapture (passed)
  - 2026-05-06 Debug Reflector host projection closeout: rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/ui/apply_presentation.rs (passed)
  - 2026-05-06 Debug Reflector host projection closeout: cargo test -p zircon_editor --lib host_scene_projection_converts_host_owned_panes_to_host_contract_panes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-shared --message-format short --color never (passed)
  - 2026-05-06 Debug Reflector host projection closeout: cargo test -p zircon_editor --lib ui_debug_reflector --locked --jobs 1 --target-dir D:\cargo-targets\zircon-shared --message-format short --color never (11 passed)
  - 2026-05-06 Debug Reflector host projection closeout: cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-shared --message-format short --color never (17 passed)
  - 2026-05-06 Debug Reflector host projection closeout: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-shared --message-format short --color never (passed with existing warning noise)
  - cargo test -p zircon_editor --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui --message-format short --color never
  - cargo test -q -p zircon_editor --lib native_template_painter_uses_material_state_palette_for_controls --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui -- --nocapture (M4 native Material palette: passed)
  - cargo test -q -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui -- --nocapture (M4 global Material asset boundary: passed)
  - cargo test -p zircon_runtime --lib layout_pass_measures_label_leaf_from_text_intrinsic_size --locked --jobs 1
  - cargo test -p zircon_runtime --lib layout_pass_measures_button_leaf_as_text_plus_padding --locked --jobs 1
  - cargo test -p zircon_editor --lib native_host_painter_composites_latest_viewport_image_into_scene_body --locked --jobs 1
  - cargo test -p zircon_editor --lib inspector_template_body_projection_replaces_legacy_inspector_view_data_for_slint_conversion --locked --jobs 1
  - cargo test -p zircon_editor --lib native_host_asset_tree_move_updates_visible_hover_state --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/tests/host/slint_asset_refresh/mod.rs
  - zircon_editor/src/tests/host/slint_builtin_assets.rs
  - zircon_editor/src/tests/editing/ui_asset/
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/tests/workbench_window_resize.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - cargo test -p zircon_editor --lib apply_presentation_resolves_splitters_from_shared_visible_drawer_projection --locked -- --nocapture
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
  - cargo test -p zircon_editor --lib inspector_surface --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-editor-layout-continuation --message-format short --color never
  - cargo test -p zircon_editor --lib viewport_toolbar -- --nocapture
  - cargo test -p zircon_editor --lib viewport_toolbar --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-editor-layout-continuation --message-format short --color never
  - cargo test -p zircon_editor --lib pane_surface_controls --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-editor-layout-continuation --message-format short --color never
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
  - 2026-05-03: cargo metadata --no-deps --format-version 1 (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins/Cargo.toml --locked --no-deps --format-version 1 (passed)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_support --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-gap-closure --message-format short --color never (passed)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --jobs 1 --target-dir E:\cargo-targets\zircon-gap-closure --message-format short --color never (passed; warnings remain)
  - 2026-05-03: cargo check --workspace --locked --all-targets --jobs 1 --target-dir E:\cargo-targets\zircon-gap-closure --message-format short --color never (passed; warnings remain)
  - 2026-05-03: git diff --check (passed; LF-to-CRLF warnings only)
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
  - cargo test -p zircon_editor --lib play_mode_ --locked --jobs 1
  - cargo test -p zircon_editor --lib default_preview_fixture_ --locked --jobs 1
  - cargo test -p zircon_editor --lib live_backend --locked --jobs 1
  - cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
  - cargo test -p zircon_editor --lib inspector_pane_projects_editable_field_nodes_and_actions --locked --jobs 1
  - cargo test -p zircon_editor --lib inspector_binding_applies_dynamic_plugin_component_fields_with_undo_history --locked --jobs 1
  - cargo test -p zircon_editor --lib inspector_pane_projects_plugin_component_drawer_fields_and_unload_degradation --locked --jobs 1
  - cargo test -p zircon_runtime --lib dynamic_plugin_component_instances_report_schema_when_loaded_and_protect_when_missing --locked --jobs 1
  - 2026-05-03: cargo test -p zircon_editor build_export_pane_projects_desktop_target_rows --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-unity-editor-final-check --message-format short --color never (timed out after producing the editor test exe)
  - 2026-05-03: E:\cargo-targets\zircon-unity-editor-final-check\debug\deps\zircon_editor-adc4066aa751f075.exe build_export_pane_projects_desktop_target_rows --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-unity-editor-final-check\debug\deps\zircon_editor-adc4066aa751f075.exe pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-unity-editor-final-check\debug\deps\zircon_editor-adc4066aa751f075.exe default_preview_fixture_exposes_hybrid_shell_tool_windows_and_empty_states --nocapture (passed)
  - 2026-05-03: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-gap-check --message-format short --color never (passed with existing warnings)
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe build_export_actions_parse_execute_profile --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe desktop_export_output_root_is_project_local_and_profile_scoped --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe builtin_viewport_toolbar_play_buttons_dispatch_menu_play_mode_operations --nocapture (passed)
  - 2026-05-03: E:\cargo-targets\zircon-editor-gap-check\debug\deps\zircon_editor-adc4066aa751f075.exe menu_action_dispatches_through_runtime_and_sets_scene_dirty_effects --nocapture (passed)
  - 2026-05-03: rustfmt --edition 2021 --check --config skip_children=true zircon_editor/src/ui/slint_host/app/build_export_actions.rs zircon_editor/src/ui/slint_host/app/build_export_actions/output_folder.rs zircon_editor/src/ui/slint_host/ui/pane_data_conversion/build_export.rs (passed)
  - 2026-05-03: git diff --check -- zircon_editor/src/ui/slint_host/app/build_export_actions.rs zircon_editor/src/ui/slint_host/app/build_export_actions/output_folder.rs zircon_editor/src/ui/slint_host/ui/pane_data_conversion/build_export.rs docs/editor-and-tooling/editor-workbench-shell.md docs/engine-architecture/runtime-editor-pluginized-export.md (passed; LF-to-CRLF warnings only)
  - 2026-05-03: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-gap-check --message-format short --color never (passed with existing warnings)
  - 2026-05-03: cargo test -p zircon_editor "build_export" --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-gap-check --message-format short --color never -- --nocapture (11 passed)
  - 2026-05-03 Milestone 4: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never (passed with existing warnings)
  - 2026-05-03 Milestone 4: cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never (timed out after compiling and running part of the lib suite)
  - 2026-05-03 Milestone 4: focused Milestone 1-3 filters for default layout/menu, pane payload, Slint projection, action parsing, and export queue (11/11 passed)
  - 2026-05-03 Milestone 4: .\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -TargetDir target\codex-shared-a -VerboseOutput (passed build and test; existing warnings remain)
  - 2026-05-04: cargo test -p zircon_editor --lib rust_owned_host_window_run_uses_native_event_loop --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-run-loop --message-format short --color never (passed)
  - 2026-05-04: cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --target-dir E:\zircon-build\targets\editor --locked --jobs 1 --message-format short --color never (passed with existing warnings)
  - 2026-05-04: python tools\zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (passed; Cargo PDB output filename collision warning remains)
  - 2026-05-04: exported E:\zircon-build\ZirconEngine\zircon_editor.exe stayed running after 5 seconds in smoke check
  - 2026-05-04: cargo test -p zircon_editor --lib rust_owned_host_window_snapshot_contains_editor_chrome_pixels --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-white-screen --message-format short --color never (RED failed with `[0, 0, 0, 0]` sampled pixels before presenter fix; GREEN passed after Rust-owned host painter)
  - 2026-05-04: cargo test -p zircon_editor --lib rust_owned_host_window_run_uses_native_event_loop --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-white-screen --message-format short --color never (passed with existing warnings)
  - 2026-05-04: rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/window.rs zircon_editor/src/ui/slint_host/host_contract/painter.rs zircon_editor/src/ui/slint_host/host_contract/presenter.rs zircon_editor/src/tests/host/slint_window/shell_window.rs (passed)
  - 2026-05-04: cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --target-dir E:\zircon-build\targets\editor-white-screen-app --locked --jobs 1 --message-format short --color never (passed with existing warnings after warmed rerun; first cold compile attempt timed out)
  - 2026-05-04: python tools\zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (passed after native host paint path; existing Cargo PDB output filename collision warning remains)
  - 2026-05-04: exported E:\zircon-build\ZirconEngine\zircon_editor.exe stayed running after 5 seconds in native paint-path smoke (`RUNNING_AFTER_5S PID=89048`)
  - 2026-05-04: exported E:\zircon-build\ZirconEngine\zircon_editor.exe smoke with diagnostics wrote E:\zircon-build\ZirconEngine\logs\2026-05-04-15-35-18\editor.log; asset/import records had no `exists=false` or `path_exists=false`, and presenter records reached 1280x720 frames with populated scene/document presentation data
  - 2026-05-04: git diff --check -- zircon_editor/Cargo.toml zircon_editor/src/ui/slint_host/host_contract/mod.rs zircon_editor/src/ui/slint_host/host_contract/window.rs zircon_editor/src/ui/slint_host/host_contract/painter.rs zircon_editor/src/ui/slint_host/host_contract/presenter.rs zircon_editor/src/tests/host/slint_window/shell_window.rs docs/editor-and-tooling/editor-workbench-shell.md .codex/sessions/20260504-1029-editor-native-white-screen.md (passed; LF-to-CRLF warnings only)
  - 2026-05-04 M1 data-driven renderer: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m1-check --message-format short --color never (passed with existing warnings)
  - 2026-05-04 M1 data-driven renderer: cargo test -p zircon_editor --lib rust_owned_host_window_snapshot --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m1-check --message-format short --color never (3 passed: chrome pixels, host scene data, pane template nodes; existing warnings remain)
  - 2026-05-04 M1 data-driven renderer: cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --target-dir E:\zircon-build\targets\editor-renderer-m1-app --locked --jobs 1 --message-format short --color never (passed with existing warnings)
  - 2026-05-04 M2 authored template/render-command renderer: rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/painter/mod.rs zircon_editor/src/ui/slint_host/host_contract/painter/frame.rs zircon_editor/src/ui/slint_host/host_contract/painter/geometry.rs zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs zircon_editor/src/tests/host/slint_window/shell_window.rs (passed)
  - 2026-05-04 M2 authored template/render-command renderer: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m2-check --message-format short --color never (passed with existing warnings)
  - 2026-05-04 M2 authored template/render-command renderer: cargo test -p zircon_editor --lib rust_owned_host --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-renderer-m2-check --message-format short --color never (9 passed: run-loop and generic/native host guards plus chrome pixels, host scene data, pane template nodes, template node styles, order/clip, runtime render commands; existing warnings remain)
  - 2026-05-04 M2 authored template/render-command renderer: cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --target-dir E:\zircon-build\targets\editor-renderer-m2-app --locked --jobs 1 --message-format short --color never (passed with existing warnings)
  - 2026-05-04 M2 authored template/render-command renderer: python tools\zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (passed; existing Cargo PDB output filename collision warning remains)
  - 2026-05-04 M2 authored template/render-command renderer: exported E:\zircon-build\ZirconEngine\zircon_editor.exe stayed running after 8 seconds in fresh post-build smoke, wrote E:\zircon-build\ZirconEngine\logs\2026-05-04-20-30-04\editor.log, recorded editor_host_presenter frames 1-5 at 1280x720 with populated scene/document data, and had no error/warn/missing/failed/path-not-found matches in that run
  - 2026-05-04 Native host interaction/glyph cut: rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/window.rs zircon_editor/src/ui/slint_host/host_contract/globals.rs zircon_editor/src/ui/slint_host/host_contract/data/host_root.rs zircon_editor/src/ui/slint_host/host_contract/painter/text.rs zircon_editor/src/ui/slint_host/host_contract/painter/primitives.rs zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs zircon_editor/src/ui/slint_host/host_contract/mod.rs zircon_editor/src/ui/slint_host/app/callback_wiring.rs zircon_editor/src/ui/slint_host/ui/apply_presentation.rs zircon_editor/src/tests/host/slint_window/shell_window.rs zircon_editor/src/ui/slint_host/app/tests.rs (passed)
  - 2026-05-04 Native host interaction/glyph cut: cargo check -p zircon_editor --lib --locked (passed with existing warning noise)
  - 2026-05-04 Native host interaction/glyph cut: cargo test -p zircon_editor --lib native_ --locked (35 passed, 932 filtered; existing warning noise remains)
  - 2026-05-04 Native host interaction/glyph cut: cargo test -p zircon_editor --lib rust_owned_host_painter_renders_distinct_glyph_shapes_instead_of_text_bars --locked (1 passed, 966 filtered; existing warning noise remains)
  - 2026-05-04 Native host interaction/glyph cut: cargo test -p zircon_editor --lib rust_owned_host --locked (10 passed, 957 filtered; existing warning noise remains)
  - 2026-05-04 Native host interaction/glyph cut: cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked (passed with existing warning noise)
  - 2026-05-04 Native host interaction/glyph cut: python tools\zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (passed and staged editor/runtime bundle plus runtime font asset)
  - 2026-05-04 Native host interaction/glyph cut: exported E:\zircon-build\ZirconEngine\zircon_editor.exe stayed running after 8 seconds in packaged smoke and was stopped intentionally; E:\zircon-build\ZirconEngine\logs\2026-05-04-22-56-02\editor.log contained 0 matches for panic/error/failed/fatal/missing and recorded packaged asset/template resolution plus native window creation
  - 2026-05-05 Structural template label fix: cargo test -p zircon_editor --lib rust_owned_host_painter_does_not_render_structural_control_ids_as_text --locked first failed with 706 changed pixels from empty Panel control_id glyph fallback; after the shared painter fix the same focused test passed (1 passed, 967 filtered; existing warning noise remains)
  - 2026-05-05 Structural template label fix: rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs zircon_editor/src/tests/host/slint_window/shell_window.rs (passed)
  - 2026-05-05 Structural template label fix: cargo check -p zircon_editor --lib --locked (passed with existing warning noise)
  - 2026-05-05 Structural template label fix: cargo test -p zircon_editor --lib rust_owned_host --locked (11 passed, 957 filtered; existing warning noise remains)
  - 2026-05-05 Structural template label fix: cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked (passed with existing warning noise)
  - 2026-05-05 Structural template label fix: python tools\zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (passed and staged editor/runtime bundle plus runtime font asset)
  - 2026-05-05 Structural template label fix: exported E:\zircon-build\ZirconEngine\zircon_editor.exe stayed running after 8 seconds in packaged smoke and was stopped intentionally; E:\zircon-build\ZirconEngine\logs\2026-05-05-00-00-08\editor.log had no panic/error/failed/fatal/missing matches via Select-String after rg was unavailable in this shell
  - 2026-05-05 Native document-tab region origin: cargo fmt --all --check (passed)
  - 2026-05-05 Native document-tab region origin: git diff --check -- zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs zircon_editor/src/tests/host/slint_window/native_host_contract.rs docs/editor-and-tooling/editor-workbench-shell.md .codex/sessions/20260505-0148-native-mouse-no-interaction.md (passed; LF-to-CRLF warnings only)
  - 2026-05-05 Native document-tab region origin: cargo test -p zircon_editor --lib native_host_pointer_click_routes_document_tab_with_document_region_origin --locked (1 passed, 971 filtered; existing warning noise remains)
  - 2026-05-05 Native document-tab region origin: cargo test -p zircon_editor --lib native_host_contract --locked (4 passed, 968 filtered; existing warning noise remains)
  - 2026-05-05 Native document-tab region origin: cargo check -p zircon_editor --lib --locked (passed with existing warning noise)
  - 2026-05-05 Native document-tab region origin: cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked (passed with existing warning noise)
  - 2026-05-05 Project workspace shell drawer restore: cargo fmt --all --check (passed)
  - 2026-05-05 Project workspace shell drawer restore: git diff --check -- zircon_editor/src/ui/host/workspace_state.rs zircon_editor/src/ui/host/builtin_layout/ensure_shell_instances.rs zircon_editor/src/ui/host/layout_hosts/repair_builtin_shell_layout.rs zircon_editor/src/tests/host/manager/bootstrap_and_startup.rs zircon_editor/src/ui/slint_host/host_contract/presenter.rs docs/editor-and-tooling/editor-workbench-shell.md .codex/sessions/20260505-0244-editor-native-visual-regression.md (passed; LF-to-CRLF warnings only)
  - 2026-05-05 Project workspace shell drawer restore: cargo test -p zircon_editor --lib applying_project_workspace --locked (2 passed, 971 filtered; existing warning noise remains)
  - 2026-05-05 Project workspace shell drawer restore: cargo test -p zircon_editor --lib native_host_pointer_click_routes_document_tab_with_document_region_origin --locked (1 passed, 972 filtered; existing warning noise remains)
  - 2026-05-05 Project workspace shell drawer restore: cargo check -p zircon_editor --lib --locked (passed with existing warning noise)
  - 2026-05-05 Project workspace shell drawer restore: python tools\zircon_build.py --targets editor,runtime --out C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-build-visual-regression-20260505 --mode debug (passed; isolated package used because E:\zircon-build was previously locked by another process)
  - 2026-05-05 Project workspace shell drawer restore: isolated packaged C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-build-visual-regression-20260505\ZirconEngine\zircon_editor.exe stayed running after 18 seconds and was stopped intentionally; C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-build-visual-regression-20260505\ZirconEngine\logs\2026-05-05-05-10-45\editor.log recorded `frame_size=1280x720`, `status_bar=0.0,696.0,1280.0,24.0`, `left=0.0,59.0,312.0,489.7`, `right=972.0,59.0,308.0,489.7`, `bottom=0.0,549.7,1280.0,146.3`, `document_tabs=2`, `left_tabs=4`, `right_tabs=1`, `bottom_tabs=3`, `document_pane_kind=Scene`, `left_pane_kind=Project`, `right_pane_kind=Inspector`, and `bottom_pane_kind=Console`; Select-String for `panic|error|failed|fatal|missing|left_tabs=0|right_tabs=0|bottom_tabs=0` returned no matches
  - 2026-05-05 Native mouse redraw regression: rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/redraw.rs zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs zircon_editor/src/ui/slint_host/host_contract/window.rs zircon_editor/src/ui/slint_host/host_contract/presenter.rs zircon_editor/src/ui/slint_host/host_contract/mod.rs zircon_editor/src/tests/host/slint_window/native_host_contract.rs zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs (passed)
  - 2026-05-05 Native mouse redraw regression: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (passed with existing warning noise)
  - 2026-05-05 Native mouse redraw regression: cargo test -p zircon_editor --lib native_host_pointer_move_routes_viewport_with_local_damage_without_frame_update --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (1 passed, 974 filtered; existing warning noise remains)
  - 2026-05-05 Native mouse redraw regression: cargo test -p zircon_editor --lib rust_owned_host_window_wait_cycle_does_not_unconditionally_request_redraw --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (1 passed, 974 filtered; existing warning noise remains)
  - 2026-05-05 Native mouse redraw regression: cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (5 passed, 970 filtered; existing warning noise remains)
  - 2026-05-05 Native mouse redraw regression: cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (passed with existing warning noise)
  - 2026-05-05 Native mouse redraw regression: cargo test -p zircon_editor --lib rust_owned_host --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (13 passed, 962 filtered; existing warning noise remains)
  - 2026-05-05 Native deep fast path: cargo fmt --all --check (passed)
  - 2026-05-05 Native deep fast path: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never (passed with existing warning noise)
  - 2026-05-05 Native deep fast path: cargo test -p zircon_editor --lib host_contract::redraw::tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (2 passed, 991 filtered; existing warning noise remains)
  - 2026-05-05 Native deep fast path: cargo test -p zircon_editor --lib tests::host::slint_window::native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (11 passed, 982 filtered; existing warning noise remains)
  - 2026-05-05 Native deep fast path: cargo test -p zircon_editor --lib presenter::tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture (2 passed, 991 filtered; existing warning noise remains)
  - 2026-05-05 Native deep fast path: python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug (passed; staged editor/runtime artifacts under E:\zircon-build\ZirconEngine with existing warning noise)
  - 2026-05-05 Native template text/input regression: cargo fmt --all --check (passed)
  - 2026-05-05 Native template text/input regression: cargo check -p zircon_editor --lib --locked (passed with existing warning noise)
  - 2026-05-05 Native template text/input regression: cargo test -p zircon_editor --lib native_host_pointer_click_routes_binding_only_template_buttons --locked (1 passed, 976 filtered; existing warning noise remains)
  - 2026-05-05 Native template text/input regression: cargo test -p zircon_editor --lib runtime_component_projection_preserves_primary_click_binding_id --locked (1 passed, 977 filtered; existing warning noise remains)
  - 2026-05-05 Native template text/input regression: cargo test -p zircon_editor --lib rust_owned_template_text_keeps_short_labels_legible --locked (1 passed, 976 filtered; existing warning noise remains)
  - 2026-05-05 Native template text/input regression: cargo test -p zircon_editor --lib native_host_pointer_click_routes_pane_template_button_actions --locked (1 passed, 976 filtered; existing warning noise remains)
  - 2026-05-05 Native template text/input regression: cargo test -p zircon_editor --lib native_host_pointer_click_routes_document_tab_with_document_region_origin --locked (1 passed, 976 filtered; existing warning noise remains)
  - 2026-05-05 Native template text/input regression: cargo test -p zircon_editor --lib applying_project_workspace --locked (2 passed, 975 filtered; existing warning noise remains)
  - 2026-05-05 Shared intrinsic text/button sizing: cargo fmt --all --check; cargo check -p zircon_editor --lib --locked --jobs 1; cargo test -p zircon_runtime --lib layout_pass_measures_label_leaf_from_text_intrinsic_size --locked --jobs 1; cargo test -p zircon_runtime --lib layout_pass_measures_button_leaf_as_text_plus_padding --locked --jobs 1 (fmt/check reached green; both runtime focused regressions passed through E:\cargo-targets\zircon-text-input-20260505; existing warning noise remains)
  - 2026-05-05 Shared intrinsic text/button sizing package smoke: C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-build-text-input-20260505 failed with `rustc-LLVM ERROR: IO failure on output stream: no space on device`; E:\tmp\zircon-build-text-input-20260505-e completed the runtime leg but timed out after 15 minutes during the editor leg, so package smoke is inconclusive for this cut
  - 2026-05-05 Native viewport image bridge and projected button frame follow-up: cargo fmt --all --check (passed); cargo test -p zircon_editor --lib native_host_painter_composites_latest_viewport_image_into_scene_body --locked --jobs 1 --message-format short --color never -- --nocapture (1 passed, 990 filtered); cargo test -p zircon_editor --lib inspector_template_body_projection_replaces_legacy_inspector_view_data_for_slint_conversion --locked --jobs 1 --message-format short --color never -- --nocapture (1 passed, 992 filtered after correcting the expected frame to the actual shared style resolution); cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never (passed with existing warning noise), all using E:\cargo-targets\zircon-text-input-20260505
  - 2026-05-05 Native viewport image/package smoke boundary: python tools\zircon_build.py --targets editor,runtime --out E:\tmp\zircon-build-text-input-20260505-e2 --mode debug and the warm E:\tmp\zircon-build-text-input-20260505-e rerun with TEMP/TMP on E: both stopped during the first runtime Cargo build with exit 4294967295 and no Rust diagnostic, so those attempts were environmental/tooling-boundary rather than source-boundary evidence
  - 2026-05-06 Package smoke recovery: direct lower-layer runtime Cargo leg `cargo build -p zircon_runtime --lib --no-default-features --features target-client --target-dir E:\tmp\zircon-build-direct-20260506\targets\runtime\lib --locked --jobs 1 --message-format short --color never` first timed out after 20 minutes on a cold target while compiling normally, then passed on the warmed target in 26.88s with existing warning noise; full package smoke `python tools\zircon_build.py --targets editor,runtime --out E:\tmp\zircon-build-package-20260506 --mode debug` passed with external transcript `E:\tmp\zircon-package-smoke-20260506-0140.log`, staging `zircon_editor.exe`, `zircon_runtime.exe`, `zircon_runtime.dll`, PDBs, and assets under `E:\tmp\zircon-build-package-20260506\ZirconEngine`
  - 2026-05-05 Native hierarchy hover state projection: cargo test -p zircon_editor --lib native_host_hierarchy_move_updates_visible_hover_state --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (RED failed because hierarchy hover callback state did not change Rust-owned painter pixels; GREEN passed after host-contract pane interaction state projection and native hierarchy row fallback painting; existing warning noise remains)
  - 2026-05-05 Native hierarchy hover state projection: cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (10 passed, 975 filtered; first broad rerun exposed stale viewport-toolbar host assertions and the derived menu-state default painting menu 0 open; passed after updating host assertions to the TOML-backed toolbar map and making default menu state closed)
  - 2026-05-05 Native hierarchy hover state projection final verification: cargo fmt --all --check (passed)
  - 2026-05-05 Native hierarchy hover state projection final verification: cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (passed with existing warning noise)
  - 2026-05-05 Native hierarchy hover state projection final verification: cargo test -p zircon_editor --lib native_host_hierarchy_move_updates_visible_hover_state --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (1 passed, 984 filtered; existing warning noise remains)
  - 2026-05-05 Native hierarchy hover state projection final verification: cargo test -p zircon_editor --lib native_host_contract --locked --jobs 1 --target-dir E:\zircon-build\targets\editor-redraw-regression --message-format short --color never (10 passed, 975 filtered; existing warning noise remains)
  - 2026-05-05 P2 viewport toolbar hit-test: rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs (passed)
  - 2026-05-05 P2 viewport toolbar hit-test: git diff --check -- zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs docs/editor-and-tooling/editor-workbench-shell.md .codex/sessions/20260505-1637-viewport-toolbar-hit-test.md (passed; docs LF-to-CRLF warning only)
  - 2026-05-05 P2 viewport toolbar hit-test: cargo test -p zircon_editor --lib hit_test_viewport_toolbar --locked (2 passed, 983 filtered; existing warning noise remains)
  - 2026-05-05 P2 viewport toolbar hit-test: cargo check -p zircon_editor --lib --locked (passed with existing warning noise after rerun with longer timeout; first 120s check timed out during dependency work)
  - 2026-05-05 P2 viewport toolbar continuation: cargo test -p zircon_editor --lib native_host_pointer_click_routes_late_viewport_toolbar_controls --locked (1 passed, 984 filtered; existing warning noise remains)
  - 2026-05-05 P2 viewport toolbar continuation: cargo test -p zircon_editor --lib hit_test_viewport_toolbar --locked (2 passed, 983 filtered; existing warning noise remains)
  - 2026-05-05 P2 viewport toolbar continuation: rustfmt --edition 2021 --check zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs zircon_editor/src/tests/host/slint_window/native_host_contract.rs (passed)
  - 2026-05-05 P2 viewport toolbar continuation: git diff --check -- zircon_editor/src/ui/slint_host/host_contract/surface_hit_test/viewport_toolbar.rs zircon_editor/src/tests/host/slint_window/native_host_contract.rs docs/editor-and-tooling/editor-workbench-shell.md (passed; docs LF-to-CRLF warning only)
  - 2026-05-05 Shared Slate-style visibility slice: focused runtime/interface/editor validation is tracked in `docs/ui-and-layout/slate-style-ui-surface-frame.md` and `tests/acceptance/shared-slate-ui-surface-frame.md`; this shell doc records the editor consequence that drawer/header/toolbar hit behavior must consume shared effective visibility instead of local bool-visible checks.
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

## 2026-05-04 Native Host Run Loop

`zircon_app` 的 `zircon_editor` binary 仍然通过 `zircon_editor::run_editor(...) -> SlintEditorHost::run() -> UiHostWindow::run()` 进入 editor shell，但 Rust-owned `UiHostWindow` 现在不再只是把 host contract 标记为 visible 后立即返回。`zircon_editor/src/ui/slint_host/host_contract/window.rs` 在 `run()` 内创建 native `winit` `EventLoop`，用 `ApplicationHandler` 创建一个最小 OS window，并把 resize、move、maximized 和 close-request state 同步回原有 `HostContractState`。这样保持了当前 Rust-owned host-contract architecture，不恢复 `slint::include_modules!()` 或 deleted generated Slint modules，同时让导出的 `zircon_editor.exe` 持有真实 native event loop。

这条修复刻意只补 executable/window lifetime，不扩大到 Build Export、plugin packaging 或 secondary native presenter 重构。`rust_owned_host_window_run_uses_native_event_loop` 源码守卫要求 `UiHostWindow::run()` 继续使用 `winit::application::ApplicationHandler`、`EventLoop::new().map_err(platform_error)?`、`run_app` 和 `WindowEvent::CloseRequested`，并拒绝回退到旧的 `self.show()` 后立即返回路径。2026-05-04 的 focused evidence 覆盖 editor binary check、export build 和 exported executable smoke：`E:\zircon-build\ZirconEngine\zircon_editor.exe` 在 5 秒后仍保持运行。

## 2026-05-04 Native Host Paint Path

run-loop 修复后暴露出的下一个问题是 native window 真实存在但没有任何 content presenter：`UiHostWindow::run()` 创建了 `winit` OS window，`SlintEditorHost` 也会把 `HostWindowPresentationData` 写入 Rust-owned host contract，但 `WindowEvent::RedrawRequested` 没有把这份 presentation 画到 surface 上，`HostWindowHandle::take_snapshot()` 也只返回全零 RGBA。新的 `rust_owned_host_window_snapshot_contains_editor_chrome_pixels` 先锁住这个 lower-layer regression：设置 center/document/status/viewport frames 后，旧实现采样到的仍是 `[0, 0, 0, 0]` blank pixels。

修复保持在 `zircon_editor/src/ui/slint_host/host_contract/` 内部，不恢复 generated Slint modules。第一版 native paint path 把 `HostWindowPresentationData` 的 shell/layout 字段转换成一帧最小 Rust-owned editor chrome RGBA：top bar、center band、left/document/viewport/status 区域和少量 marker bar 共同保证 snapshot 与 native presenter 都不是白屏/空 surface。`presenter.rs` 用与 runtime presenter 同类的 `softbuffer` surface，把同一份 CPU-painted frame 写入 native `winit` window；`window.rs` 在 create-surface 阶段创建 presenter，在 resize 时同步 surface，在 `RedrawRequested` 与 `about_to_wait` 的 redraw request 循环中持续 present 当前 host presentation。

这条 paint path 是当前 Rust-owned host contract 的最低可见性层，不试图替代后续完整 Slint/workbench scene renderer。它的 acceptance boundary 是：native editor window 有真实非空 content，snapshot 能证明 host chrome region 不同色，run-loop guard 仍然禁止回退到立即返回路径，export/app check 与 smoke 再证明 `zircon_app` 的 editor binary 能带着 presenter 编译并运行。

2026-05-04 的 file-backed exported smoke 进一步确认异常显示不在 asset/import 或 presentation 空数据层：`E:\zircon-build\ZirconEngine\logs\2026-05-04-15-35-18\editor.log` 没有 `exists=false` / `path_exists=false`，`editor_host_window` 在创建 native window 前收到了 project path、viewport label 和非零 center/document/viewport frame，`editor_host_presenter` 随后连续 present `1280x720` frame，并记录 `page_tabs=1`、`document_tabs=2`、`document_pane_kind=Scene`。因此，后续 display repair 的正确边界是让 native host renderer 消费 already-populated authored scene DTO，而不是回退到 generated Slint modules 或 asset/import fallback。

## 2026-05-04 Data-Driven Native Host Renderer

M1 renderer 把原来单文件 skeletal painter 拆成 `host_contract/painter/` folder-backed subsystem：`frame.rs` 只持有 `HostRgbaFrame`，`geometry.rs` 只处理可见性、translation、intersection 与 pixel clip，`primitives.rs` 只画稳定 CPU primitives，`template_nodes.rs` 把 `TemplatePaneNodeData` role/style/text 转成 deterministic panel/button/text-bar visuals，`workbench.rs` 负责按 host scene z-order 组合 menu/page/status/dock/pane/floating/resize surfaces。`mod.rs` 只暴露 `paint_host_frame(...)` 和 `HostRgbaFrame` 给 `window.rs` 与 `presenter.rs`。

数据源现在优先来自 `HostWindowPresentationData.host_scene_data`：root layout、menu chrome、page chrome、status bar、left/right/document/bottom dock surface、pane body DTO、floating windows 和 resize layer 都由 host scene DTO 驱动；`host_layout` 只在 scene frame 缺席时作为最低 fallback。pane body 不是再画单个 marker，而是根据 `PaneData.kind` 读取对应的 `TemplatePaneNodeData` list，例如 Hierarchy、Inspector、Console、Assets、AssetBrowser、Project、BuildExport、ModulePlugins、UiAssetEditor 与 Animation pane 都通过同一条 template-node drawing path 进入 native RGBA frame。

M1 deliberately remains CPU/deterministic: text is represented by stable bars keyed by text bytes, image/icon loading is not introduced, invalid or zero-size frames are skipped through shared geometry guards, and clipping is applied at every template-node/surface boundary. This keeps `HostWindowHandle::take_snapshot()` useful for regression tests and keeps the exported `softbuffer` presenter on the same paint entry as the test snapshot path.

Focused M1 evidence is recorded in the frontmatter test list: editor library check passed, `rust_owned_host_window_snapshot` ran the existing chrome regression plus new host-scene and pane-template-node regressions, and the `zircon_app` editor binary target check passed. Existing warning noise remains in runtime/editor dead-code imports and is not hidden by the renderer milestone.

## 2026-05-04 Authored Template Render-Command Renderer

M2 keeps the `workbench.rs` renderer as scene orchestration but moves template visual drawing onto a shared CPU command backend. `painter/render_commands.rs` owns `HostPaintCommand`, deterministic command ordering, runtime `UiRenderCommand` adaptation, style color parsing for `#rgb`, `#rgba`, `#rrggbb` and `#rrggbbaa`, opacity application, border-width rasterization, and deterministic text/image placeholders. `template_nodes.rs` now maps `TemplatePaneNodeData` into those host commands instead of drawing each node with an ad hoc path, so menu/page/status/dock header/rail/pane body nodes use the same primitive backend as runtime-style render commands.

The normal source of editor host pixels remains `HostWindowPresentationData.host_scene_data`; the skeletal `host_layout` painter is only a missing-scene fallback. Authored `.ui.toml` projections that arrive as template-node DTOs now preserve selected/disabled/hovered/button variants, borders, stable labels, z/list order, and clipping to the pane or dock surface. Runtime-style `UiRenderCommandKind::Quad`, `Text`, `Image`, and `Group` are supported through the same backend. The original M2 boundary used deterministic image placeholders; the 2026-05-06 follow-up keeps those placeholders only for missing assets and routes loaded image/SVG pixels through `visual_assets.rs` and `draw_rgba_image_clipped(...)`.

Focused M2 regressions cover the backend contract directly: `rust_owned_host_window_snapshot_renders_template_node_styles` samples selected, primary, disabled, and label-only node output; `rust_owned_host_window_snapshot_respects_template_node_order_and_clip` locks traversal order and pane clipping; `rust_owned_host_painter_draws_runtime_render_commands` builds `UiRenderCommand` values and proves quad, z-order, text, and image placeholder output. The M2 rerun also kept the M1 host-scene and pane-template-node snapshot coverage green through the shared backend, and the broader `rust_owned_host` filter kept the run-loop and generic/native host contract guards green in the same target directory.

The 2026-05-06 visual-asset follow-up makes the placeholder path a fallback instead of the primary image renderer. `TemplatePaneNodeData.preview_image` is converted to RGBA and drawn by `template_nodes.rs`; runtime `UiVisualAssetRef::Image` and `UiVisualAssetRef::Icon` are resolved by `visual_assets.rs`, cached, optionally icon-tinted, and emitted as image-pixel host commands. This is still the Rust-owned host-contract painter: `.ui.toml` remains the business source, Slint is only used as an image/SVG decoder, and the generated Slint UI path is not restored.

The same follow-up now treats icons as first-class editor chrome, not two-letter label placeholders. `workbench_activity_rail.ui.toml` authors `Icon` nodes for the rail stencil, and `chrome_template_projection.rs` maps tab `icon_key` values such as `project`, `hierarchy`, `asset-browser`, `scene`, and `console` to stable Ionicons SVG names before loading preview pixels through the shared `preview_images.rs` resolver. The active rail button returns `ActivityRailButton*` as the selected chrome control, while the icon node carries active/default or muted tone state. This keeps the JetBrains-style left rail visually dense and tool-like while preserving the Slate-like `.ui.toml -> shared template -> host DTO -> native painter` route.

The continuation extends the same icon contract to page tabs and dock tabs. `chrome_template_projection.rs` applies the tab icon mapping to `PageTab*` and `DockTab*`, marks the active tab as selected/focused for Material state tinting, and turns `DockTabClose*` into SVG `IconButton` nodes backed by `close-outline` instead of text placeholders. `workbench_dock_header.ui.toml` authors those close slots as icon buttons, so fallback and authored projections expose the same chrome control role.

The next menu-chrome continuation keeps menu actions on the same path. `menu_popup_nodes(...)` now maps stable action ids and labels to Ionicons SVG names before applying the icon to each `MenuPopupItemLabel*` template node. File actions use folder/save/sync glyphs, Edit actions use back/forward glyphs, Play actions use play/remove glyphs, Scene creation uses cube/camera/light glyphs, and View entries reuse the same descriptor-to-icon vocabulary as page and activity tabs. Because the icon is projected onto the existing label node, the native painter's icon-plus-text layout reserves leading icon space without changing popup hit testing or adding a second menu item schema.

Template projection also carries interactive appearance flags end to end. `ViewTemplateNodeData` now preserves `selected`, `focused`, `hovered`, `pressed`, and `disabled`; `template_node_conversion.rs` forwards them into `TemplatePaneNodeData`; and `template_nodes.rs` paints distinct hover, pressed, selected, and disabled Material state layers while suppressing structural text over icon-only controls. Image rects preserve aspect ratio, icon rects stay centered inside fixed-format controls, and icon-plus-text controls reserve leading icon space before painting text so labels do not overlap the SVG glyph. This keeps hover/selection from shifting or corrupting chrome layout.

## 2026-05-06 Welcome Main UI Visual Repair

The Welcome pane now travels through the same host-scene DTO path as the other document panes instead of relying only on `PaneSurfaceHostContext.set_welcome_pane(...)`. `apply_presentation.rs` projects the current `WelcomePresentation` into every `PaneData` whose kind is `Welcome`; `PaneData` carries `WelcomePaneData`; `world_space_submission.rs` and `surface_hit_test/template_node.rs` include `pane.welcome.nodes`; and `painter/workbench.rs` selects those nodes before painting native Welcome-specific content. This prevents the native presenter from falling back to the single `Welcome` label even when the `.ui.toml` projection and global welcome context were valid.

The authored Welcome layout now imports the Material component asset and `editor_material` style so the startup surface is still grounded in the Slint/Material `.ui.toml` component catalog. The Rust-owned host painter keeps the current deterministic CPU primitive boundary, but uses those projected control frames as anchors for a JetBrains-like dark workbench surface: recent-project panel, main form column, status strip, outlined fields, path preview, validation text, and disabled/enabled action states all paint from the same `HostWindowPresentationData.host_scene_data.document_dock.pane` data. Empty form data no longer reports a valid project state; the preview and validation copy now match the disabled actions.

Two adjacent chrome polish fixes belong to this same visual boundary. `chrome_template_projection.rs` filters `DockTabClose*` nodes by the live tab count and each tab's `closeable` flag, so non-closeable Welcome tabs do not render empty black close-button surfaces. Startup/project display text also strips Windows verbatim path prefixes for UI copy and uses the project title in the narrow top page chrome while leaving the full readable path in status text.

Focused evidence for this cut:
- `cargo test -p zircon_editor welcome_projection_maps_bootstrap_asset_into_mount_nodes --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui -- --nocapture`
- `cargo test -p zircon_editor rust_owned_host_window_snapshot_draws_welcome_main_content --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui -- --nocapture`
- `cargo test -p zircon_editor dock_header_nodes_hide_close_controls_for_non_closeable_tabs --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui -- --nocapture`
- `cargo test -p zircon_editor display_project_path --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui -- --nocapture`
- `cargo fmt --all --check`
- `cargo build -p zircon_app --bin zircon_editor --features target-editor-host --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui`
- Visual QA screenshot: `target/visual-layout/main-ui-after-polish-20260506-150348.png`

Export smoke after the M2 build used `E:\zircon-build\ZirconEngine\zircon_editor.exe` from the packaged folder. The process was still alive after 8 seconds and was stopped intentionally; `E:\zircon-build\ZirconEngine\logs\2026-05-04-20-30-04\editor.log` recorded `editor_host_presenter` frames `1` through `5` at `1280x720` with project path, center/document/viewport frames, `page_tabs=1`, `document_tabs=2`, and `document_pane_kind=Scene`. The same log had no `error`, `warn`, `missing`, `failed`, `exists=false`, or `path_exists=false` matches, so the remaining display boundary is no longer asset/template availability or native presenter lifetime.

## 2026-05-04 Native Host Interaction And Glyph Cut

The screenshot after M2 showed three lower-layer regressions in the Rust-owned host contract: text was still placeholder bars, the native `winit` loop dropped pointer events instead of forwarding them into the already-wired `UiHostContext` / `PaneSurfaceHostContext` callbacks, and redraw/present could use stale presentation data. The repair keeps the current Rust-owned host-contract architecture and does not restore generated Slint modules.

`host_contract/window.rs` now forwards `PointerMoved`, `PointerButton`, and `MouseWheel` events through `host_contract/native_pointer.rs`. The routing layer reads `HostWindowPresentationData.host_scene_data` plus current `HostMenuStateData` and dispatches menu, menu-popup, chrome tab, rail, resize, pane, and viewport events into the same shared callback bridges that tests and generated-host-era code already used. `RedrawRequested` and resize call the installed `on_frame_requested` callback before presenting, and `app/callback_wiring.rs` wires that callback to `SlintEditorHost::tick()` so dirty layout/presentation work is recomputed before the `softbuffer` frame is painted.

`painter/text.rs` adds the CPU glyph primitive. It embeds `zircon_runtime/assets/fonts/FiraMono-subset.ttf`, rasterizes with `fontdue`, clips through the existing pixel-rect geometry layer, and alpha-blends coverage pixels into `HostRgbaFrame`. `render_commands.rs`, `template_nodes.rs`, `primitives.rs`, and `workbench.rs` now route former text-bar calls through this glyph path while preserving the same `paint_host_frame(...)` entry used by snapshots and the native presenter.

The root layout authority was tightened in `painter/workbench.rs`: if `host_scene_data.layout` contains visible root frames, the painter uses that scene layout as the single root-frame source; only an empty scene layout falls back to legacy top-level `host_layout` plus synthetic defaults. This prevents per-field mixing of stale `host_layout` values with the authored scene projection.

Focused validation for this cut stayed scoped to the affected editor host path rather than claiming a workspace-wide green run. The validation stage covered formatting for the touched host-contract/app/test files, `zircon_editor` library type-check, focused native pointer/frame regressions, the glyph regression, the broader `rust_owned_host` host-contract filter, the `zircon_app` editor binary target check, a debug export build, and a packaged editor smoke. The packaged smoke used `E:\zircon-build\ZirconEngine\zircon_editor.exe`; it remained alive for 8 seconds before intentional termination, and `E:\zircon-build\ZirconEngine\logs\2026-05-04-22-56-02\editor.log` had no `panic`, `error`, `failed`, `fatal`, or `missing` matches while resolving staged assets and creating the native window.

## 2026-05-05 Structural Template Labels

The first glyph-backed exported screenshot exposed a separate lower-layer painter bug: structural template nodes with empty visible text were falling back to `control_id` or `node_id`, so routing/debug identifiers such as `WorkbenchPageBar`, `DockHeaderBar`, and status-panel ids became real glyphs. The old placeholder-bar path made this much less visible; the real glyph path correctly revealed that `control_id` is metadata, not display text.

`painter/template_nodes.rs` now resolves node labels only from explicit display fields: `text`, `value_text`, and `options_text`. Structural ids stay available for pointer routing, callback dispatch, and test selection, but they are not painted unless an authored template intentionally provides visible text. This keeps the fix at the shared template-node painter layer instead of adding one-off suppression rules for page chrome, dock headers, or status bar nodes.

The regression `rust_owned_host_painter_does_not_render_structural_control_ids_as_text` compares an anonymous structural `Panel` snapshot against the same `Panel` with only `control_id` set and requires zero changed pixels in the panel text region. It also renders the same node with explicit `text = "Workbench"` and requires visible pixel changes, so the guard blocks structural-id leakage without disabling legitimate template text.

## 2026-05-05 Native Host Interaction And Occlusion Follow-up

The next exported-editor screenshot exposed two support-layer gaps in the Rust-owned host contract rather than a need to restore generated Slint modules. Pointer facts were reaching `native_pointer.rs`, but pane body routing still fell through to broad pane-kind fallbacks before testing authored `TemplatePaneNodeData` controls, so native clicks on Build Export / Project-style buttons never reached the generic surface-control action bridge. Scene/Game toolbar points also sat inside the viewport content area and could be consumed as raw viewport body events before the toolbar controls were resolved.

`host_contract/surface_hit_test/template_node.rs` owns pane template-node hit testing for the native host path. It selects the correct node collection from `PaneData.kind`, consumes the submitted `PaneData.body_surface_frame`, ignores disabled or non-dispatchable nodes when building that frame, and returns `control_id`, `action_id`, `binding_id`, and `dispatch_kind` metadata without treating those ids as visible labels. `native_pointer.rs` routes those hits before pane fallbacks, dispatching inspector/showcase-specific controls to their existing callbacks and all other button-like controls to `PaneSurfaceHostContext.surface_control_clicked`.

`host_contract/surface_hit_test/viewport_toolbar.rs` resolves visible Scene/Game toolbar controls from `SceneViewportChromeData` before the viewport body route is chosen. The native hit-test consumes the toolbar `UiSurfaceFrame.hit_grid` through the generic `surface_frame.rs` adapter, so it mirrors `scene_viewport_toolbar.ui.toml` after shared layout instead of stopping at the early tool/space/display/grid group: snap, preview lighting/skybox/gizmos, frame selection, play-mode, projection, and align-view controls all produce `PaneSurfaceHostContext.viewport_toolbar_pointer_clicked` hits with their toolbar-local arranged control frame. Unhandled toolbar/background or body points still flow to the raw viewport pointer bridge.

The menu/content occlusion regression was a painter z-order issue in the same Rust-owned host layer. Open-menu state and popup hit regions already existed, but `painter/workbench.rs` rendered only the top menu chrome before pane and viewport surfaces, so an opened popup could be visually hidden by the document/viewport body. `draw_open_menu_popup(...)` now consumes `HostMenuStateData.open_menu_index`, menu frames, popup dimensions, and `HostMenuChromeMenuData.popup_nodes`, then paints the popup after dock/pane/floating/resize layers and before the status bar. The popup remains a host-contract render concern and uses the same template-node backend as pane bodies.

Focused regressions lock the boundary directly in `native_host_contract.rs`: one verifies pane template buttons dispatch their action metadata through the native click path, one verifies viewport toolbar controls win before raw viewport body events, and one snapshots an opened menu to prove popup pixels are painted above the document/viewport surface below the menu bar.

The focused module regression in `surface_hit_test/viewport_toolbar.rs` covers the shared surface-frame adapter and asserts that the hit result comes from arranged node geometry. The broader native host contract builds toolbar fixture frames through `BuiltinViewportToolbarTemplateBridge::surface_frame_for_projection_controls(...)`, so test clicks no longer depend on a Rust coordinate table that can drift from the TOML layout.

Projection control ids are no longer kept alive as legacy route aliases. `display.cycle`, `snap.translate`, `toggle.gizmos`, and `frame.selection` remain the hit-grid action ids consumed by `ViewportToolbarPointerBridge`; TOML projection ids such as `SetDisplayMode` and `FrameSelection` fall back through `dispatch_builtin_viewport_toolbar_control(...)` and template bindings when no legacy route exists. This deletes the old toolbar alias list rather than preserving it as a compatibility shim.

Real-host callbacks that upload `width == 0 && height == 0` are handled in two stages. If the callback id is a TOML projection id, the dispatcher recovers the authored frame from `BuiltinViewportToolbarTemplateBridge`. If the callback id is already a legacy hit-grid action id such as `display.cycle` or `align.neg_z`, there is intentionally no reverse projection alias lookup; the dispatcher uses the click point as a one-pixel active frame and lets `ViewportToolbarPointerBridge` validate and route that action id through the shared surface dispatcher. This keeps zero-rect fallback alive for the real host without restoring a Rust toolbar coordinate table.

Pane component projection has a display-only fallback for bound or button-like controls with no authored label. It humanizes ids such as `ApplyDraft` into visible text, but that helper is not a dispatch alias table and is not part of the toolbar hard-cutover surface.

The native late-toolbar regression now uses a wide enough test document surface for the TOML-aligned left group to be visible and clicks `frame.selection` at toolbar-local `x=659`. That locks the real routed frame at `x=649`, `width=20`, instead of the stale pre-TOML hard-coded frame that overlapped `snap.rotate`.

Native viewport-toolbar command dispatch is intentionally click-like, not a raw button event stream. `native_pointer.rs` now invokes `PaneSurfaceHostContext.viewport_toolbar_pointer_clicked` only for `NativePointerButtonState::Pressed` with `UiPointerButton::Primary`; primary release, secondary press, and middle press return idle for toolbar controls so actions such as `display.cycle` and `grid.cycle` cannot fire twice or from non-primary buttons. `native_host_contract.rs::native_host_viewport_toolbar_only_dispatches_primary_press` locks that contract with a real toolbar hit on `display.cycle`.

## 2026-05-05 Native Document-Tab Region Origin

The remaining native click regression was lower than callback wiring: exported `winit` pointer coordinates are client-window coordinates, but root document-tab frames are nested twice. `HostDocumentDockSurfaceData.header_frame` is local to `document_dock.region_frame`, and each tab frame is local to that header. `native_pointer.rs` had been feeding the header frame into `route_document_tabs(...)` without adding the document dock region origin, so a global click on visible Scene/Game document tabs missed the chrome route and fell through to idle routing.

`route_top_level_chrome(...)` now translates the root document dock header by `document_dock.region_frame.x/y` before hit-testing document tabs. This matches the already-global drawer region and floating-window header routes while keeping the shared `document_tab_pointer_clicked` callback bridge unchanged. The focused regression `native_host_pointer_click_routes_document_tab_with_document_region_origin` sets a non-zero document dock origin, clicks the global tab position, and requires the native host route to request redraw and emit the expected document tab callback payload.

## 2026-05-05 Project Workspace Shell Drawer Restore

The exported native screenshot with missing left/right/bottom drawer bars and no clear status/task shell was traced below the painter: the host presenter log had non-zero center/document/viewport frames and two document tabs, but `left_tabs=0`, `right_tabs=0`, `bottom_tabs=0`, and all drawer frames were zero. That means the native projection was faithfully painting an incomplete workbench model. The lowest shared failing layer was project workspace restoration, not the Rust-owned painter or generated Slint compatibility path.

`workspace_state.rs` now treats `apply_project_workspace_state(...)` the same way as startup bootstrap after loading serialized project state. It restores the project-owned instances, re-ensures builtin shell instances, then calls `repair_builtin_shell_layout(...)` before layout normalization and session metadata recomputation. This keeps user/project document tabs intact while reintroducing the baseline editor shell drawers that are required for the activity rails, side/bottom tabs, and bottom tool/status area to have visible model data.

`ensure_shell_instances.rs` preserves restored single-instance descriptors instead of blindly inserting the default builtin instance id. If a project workspace reopens `editor.hierarchy#restored`, the shell bootstrap does not add a competing `editor.hierarchy#1`. `repair_builtin_shell_layout.rs` takes the current open-instance list and maps every baseline builtin id back to the matching restored instance id when the descriptor is single-instance. The same repair is applied to both root `WorkbenchLayout.drawers` and the embedded `window:workbench` activity drawer layout, because presentation builders may consume either the root compatibility field or the activity-window-scoped drawer set during this transition.

Repair also refreshes shell drawer metadata when baseline shell tabs are inserted into an existing drawer, or when an already-present shell tab sits in a stale/invisible drawer. That keeps restored project workspaces from preserving `visible=false`, zero extent, or collapsed metadata that would still suppress `tool_region_has_tabs(...)` and keep side/bottom regions hidden even after the builtin tab ids are present.

The focused manager regressions cover both halves of the support fix. `applying_project_workspace_restores_single_instance_registry_state` now verifies that a restored hierarchy instance remains the shell layout id and that the default `editor.hierarchy#1` id is not reintroduced. `applying_project_workspace_preserves_builtin_shell_drawers` starts from a project workspace that serializes Scene/Game document tabs plus malformed drawer state, applies it through `EditorManager`, and requires Project/Inspector/Console drawers plus their builtin instances to exist in both root and stored workbench activity-window drawer layouts with visible non-zero shell metadata.

## 2026-05-05 Native Mouse Redraw Boundary

The native mouse redraw regression was in the Rust-owned host event loop rather than UITOML asset loading. Pointer movement could route through the viewport or menu path, return a generic redraw request, and then `WindowEvent::RedrawRequested` always called `request_frame_update()`. On top of that, `about_to_wait()` requested a redraw every idle cycle, so ordinary mouse movement looked like a full presentation tick and could replay template/resource diagnostics even when layout and authored UI assets did not change.

`host_contract/redraw.rs` now owns the native redraw contract. Pointer dispatch returns `HostRedrawRequest::None`, `HostRedrawRequest::Region(FrameRect)`, or `HostRedrawRequest::Full { frame_update }` instead of a boolean. `native_pointer.rs` uses region damage for move/scroll paths that only need the affected menu, pane, or viewport body repainted. Clicks, resize, and other state-changing operations still request a full frame update so `SlintEditorHost::tick()` can rebuild presentation data before the presenter paints.

`host_contract/window.rs` now queues those redraw requests explicitly. Region requests are unioned until the next `RedrawRequested`, full requests dominate region requests, and the idle `about_to_wait()` hook only synchronizes native window state. `RedrawRequested` calls `request_frame_update()` only when the queued request requires it, then passes any damage rectangle to `SoftbufferHostPresenter::present(...)`. `presenter.rs` converts that frame-space damage into a `softbuffer::Rect` and uses `present_with_damage(...)`; full or invalid damage still falls back to a normal full present.

The regression boundary is covered directly. `native_host_pointer_move_routes_viewport_with_local_damage_without_frame_update` verifies that a native viewport move still reaches the shared viewport pointer bridge but returns only the viewport body damage and `requires_frame_update=false`. `rust_owned_host_window_wait_cycle_does_not_unconditionally_request_redraw` is a source guard that rejects bringing back the idle-loop `request_redraw()` cycle.

## 2026-05-05 Native Paint And Pointer Fast Path

The next optimization turns the redraw boundary into an actual paint fast path. `SoftbufferHostPresenter` now keeps a retained `HostRgbaFrame` backbuffer. Full invalidation still repaints the whole workbench, but region redraws set an active paint clip on the frame, clear only the damaged rectangle, repaint the host skeleton/scene through that clip, and copy only the damaged pixel rows into softbuffer before `present_with_damage(...)`.

The native painter clip is centralized in `HostRgbaFrame` and consumed by primitives, text, render-command painting, template-node painting, and viewport image painting. This matters because a damage rectangle is only useful if every draw operation agrees on the same clipped spatial fact; otherwise a local draw helper can silently turn a region repaint back into an expensive full repaint.

Pointer movement now distinguishes input dispatch from native repaint. Viewport moves still reach the runtime viewport pointer bridge, but the native host returns idle because no Rust-owned host pixels changed; repaint follows the next viewport image. Hierarchy and asset-tree hover compare pointer-only interaction state before and after callbacks. Unchanged hover targets return idle, while hierarchy hover changes damage the affected row instead of the whole window.

Template loading also received a process-local cache. Builtin `.ui.toml` documents are cached as parsed and compiled documents using canonical path, file length, and modification timestamp as the key, so multiple bridge/runtime instances no longer repeatedly compile the same host templates during refresh. Host presentation logs now include a rebuild count, and presenter logs include full/region paint counts plus painted pixel totals for regression diagnosis.

`apply_presentation` now builds only the payload needed by the pane's actual kind before constructing `PaneData.body_surface_frame`. Scene panes avoid hierarchy/inspector/console/module/export conversion, and non-project panes avoid project overview projection. This keeps the full presentation path available for structural changes while reducing the amount of work done per valid slow path.

The deep fast-path follow-up adds an external redraw queue to `host_contract/window.rs`. App-side producers can now enqueue a `HostRedrawRequest::Region(...)` without marking presentation/layout dirty or calling `request_frame_update()`. `about_to_wait()` drains and coalesces those requests into the same pending redraw path used by pointer dispatch, so full redraws still dominate but independent paint-only changes do not recreate the presentation model.

Viewport image refresh now uses that channel. `poll_viewport_image_for_native_host()` accepts a fresh `SlintViewportController::poll_image()` result, publishes it through `PaneSurfaceHostContext::set_viewport_image(...)`, and requests only the current viewport content frame as damage when the image was accepted. The native host converts the Slint image into `HostViewportImageData` RGBA bytes, preserves that payload across `apply_presentation`, and `painter/workbench.rs` composites it into `Scene` / `Game` pane bodies through the shared clipped primitive path. Repeated status-line writes also short-circuit against `EditorEventRuntime::status_line()`, so recurring background diagnostics with identical text no longer keep flipping `presentation_dirty`.

Focused coverage for this cut is `presenter::tests`, `builtin_template_compile_cache_is_reused_across_runtime_instances`, `tests::host::slint_window::native_host_contract`, and `native_host_painter_composites_latest_viewport_image_into_scene_body`. The viewport-image regression writes a synthetic RGBA frame through the same `PaneSurfaceHostContext` seam used by `poll_viewport_image_for_native_host()` and asserts the Rust-owned native snapshot changes inside the Scene body rather than only repainting chrome. The old viewport-move regression has been replaced by `native_host_pointer_move_routes_viewport_without_native_repaint`, because viewport movement should not repaint native chrome until rendered viewport image data changes.

## 2026-05-05 Native Template Text And Binding Dispatch

The follow-up exported-editor screenshot showed two remaining support-layer symptoms after the drawer/status model repair: short authored labels were still partially clipped in Rust-owned template nodes, authored buttons without literal `action_id` metadata were not dispatchable, and some button frames collapsed toward their label bounds instead of preserving a clickable padded control rect. These failures were below individual panes. The text paint problem lived in the shared CPU glyph layout/clipping path, the button-frame problem lived in runtime shared layout measurement, and the dispatch problem lived in template projection, native template-node hit testing, and the generic pane-surface dispatch fallback.

Shared runtime layout now measures leaf text before the editor native painter sees it. `zircon_runtime/src/ui/surface/render/text_measure.rs` resolves template text/style metadata and produces a deterministic intrinsic text size, `layout/pass/measure.rs` applies that size to leaf nodes and adds button/icon-button padding around non-empty text, and `layout/pass/axis.rs` uses measured desired main-axis extents for default linear child constraints without overriding authored width/height constraints. This keeps the button contract as “text first, then padding,” and keeps the fix below native hit-test/painter code so exported and in-process host paths share the same layout cache frames.

`TemplatePaneNodeData` now carries a projected `binding_id` alongside `control_id`, `action_id`, and `dispatch_kind`. `pane_component_projection` derives that id from authored host-node `UiEventKind::Click` bindings, so buttons such as inspector `ApplyDraft` or pane-surface `TriggerAction` remain dispatchable even when the visual node has no literal `action_id`. `surface_hit_test/template_node.rs` treats template nodes as dispatchable when any of `action_id`, `binding_id`, or `dispatch_kind` is present, and `native_pointer.rs` forwards the binding id as the primary payload before falling back to action metadata.

Dispatch stays on existing host-contract seams instead of adding pane-specific one-offs. `callback_dispatch/template_binding.rs` resolves known builtin template binding ids through `builtin_template_bindings()`, `pane_surface_actions.rs` tries that binding route when the pane surface bridge has no matching control, and the existing workbench `dispatch_menu_action(...)` remains the fallback for literal menu/view action ids. `dispatch_kind = "asset"` remains routed to the asset control path so action-less Project Overview asset-browser controls are not mistaken for generic menu actions.

The native text repair keeps clipping in the shared painter layer. `template_nodes.rs` reduces vertical inset when a node is short, sizes labels from the resulting text rect height, and `painter/text.rs` clamps font size and line height to the available rect before asking `fontdue` for glyph layout. The focused regression `rust_owned_template_text_keeps_short_labels_legible` snapshots a 14px-tall primary button and counts foreground-influenced glyph rows rather than relying on exact fully-covered antialias pixels.

Focused evidence for this cut stayed scoped to the affected editor host/runtime layout path. Formatting and `cargo check -p zircon_editor --lib --locked` passed with the existing runtime/editor warning noise. The projection regression now proves the primary click binding id survives template-node conversion, the binding-only native click regression proves that binding id is the dispatched pane callback payload, and the short-template-text snapshot regression passed. Adjacent pane-template-button, document-tab-origin, and project-workspace restore regressions stayed green. Runtime layout regressions now prove label leaves get non-zero intrinsic text size and button leaves become text size plus padding, and `inspector_template_body_projection_replaces_legacy_inspector_view_data_for_slint_conversion` now asserts the real `ApplyDraft` projected frame is `(106.0, 27.2)`, i.e. current text/style measurement plus shared button padding, before native hit testing consumes it. Package smoke has now recovered on an isolated E: output: after proving the direct runtime Cargo leg could finish on a warmed target, `python tools\zircon_build.py --targets editor,runtime --out E:\tmp\zircon-build-package-20260506 --mode debug` completed and staged the editor/runtime artifacts plus assets. The remaining unproven acceptance item is a real launched packaged-editor screenshot/interaction check, not package construction.

The scene-viewport missing-rendering symptom was a separate support-layer issue from text and button sizing. `SlintEditorHost::tick()` could submit a render extract and `SlintViewportController::poll_image()` could produce a Slint `Image`, but the Rust-owned native host bridge had to retain CPU-readable viewport pixels and the native painter had to composite them into Scene/Game pane bodies. `HostViewportImageData` is now the host-contract storage boundary for accepted RGBA frames; `HostWindowPresentationData` carries it like menu and pane interaction overlays, so presentation rebuilds do not discard the latest renderer image.

## 2026-05-06 Native Visual Assets And Debug Overlay

The SVG/Image regression is fixed at the Rust-owned host painter layer rather than by restoring generated Slint UI. `TemplatePaneNodeData.preview_image` is now a real painter input: `template_nodes.rs` converts the retained `slint::Image` to RGBA pixels, tints icon-like roles such as `Icon` and `SvgIcon`, and emits a clipped image-pixel command before text. Runtime-style `UiRenderCommandKind::Image` uses the same host command backend; `visual_assets.rs` resolves `UiVisualAssetRef::Image` from editor/runtime asset roots and `UiVisualAssetRef::Icon` from the editor icon folders, decodes SVG/bitmap content through Slint's image loader, caches the result, and leaves the deterministic placeholder only for missing assets.

The same cut adds a top-right native debug marker. `HostWindowShellData.debug_refresh_rate` is projected from the workbench host presentation, copied through `apply_presentation.rs`, and consumed by `painter/workbench.rs` while drawing the top bar. The startup value is the static fallback `FPS 0.0 | present 0 | full 0 | region 0 | pixels 0 | slow 0 | render 0 | paint-only 0`, which exists only before the first native present has a real presenter snapshot.

Milestone 1 replaces the static marker with `HostRefreshDiagnostics` in `host_contract/diagnostics.rs`. `SoftbufferHostPresenter::present(...)` now records present count, full/region paint split, painted pixel totals, and FPS timing with saturating counters, then merges the latest `HostInvalidationRoot` snapshot before painting the top bar. The rendered overlay string includes `FPS`, `present`, `full`, `region`, `pixels`, `slow`, `render`, and `paint-only`, so the visible native shell reports both presenter damage behavior and invalidation-root rebuild pressure without parsing diagnostic log text.

The top-right marker geometry is owned by `painter/diagnostics_overlay.rs`, not by ad hoc math in the large workbench painter. `workbench.rs` uses the helper when painting the marker, and `SoftbufferHostPresenter` uses the same helper when a region-presented frame changes overlay text. The helper derives the top bar from `HostWindowPresentationData`: scene layout wins when it has visible root frames, otherwise the legacy host layout is used, and only empty layout data falls back to the startup top-bar height. If the text changed and a region repaint is already scheduled, the presenter unions the marker frame into the existing damage before repainting and presenting; if the text is unchanged, the original region damage is left untouched; if the frame is already a full repaint, no synthetic region damage is created. The presenter plans the overlay string after the final expanded damage is known, so `pixels` in the same-frame visible text matches the `HostRefreshDiagnostics` total recorded for that present instead of reporting the pre-overlay damage estimate. This keeps the live diagnostics overlay current during retained-region presents without scheduling a second redraw loop just to update the overlay.

Focused Milestone 1 coverage now includes the live native host contract path, not only DTO formatting. `presenter_diagnostics_plan_same_frame_overlay_pixels_match_expanded_region_damage` verifies that region damage expanded by the top-right marker produces matching overlay text, recorded diagnostics, and cloned presentation data for the same present. `host_window_refresh_diagnostics_update_state_overlay_text` verifies the `WindowEvent::RedrawRequested` success-path seam by exercising the host-state overlay update method used after `SoftbufferHostPresenter::present(...)` returns diagnostics.

The host lifecycle publishes invalidation diagnostics through the Rust-owned host contract whenever slow-path, render-path, or paint-only counters change. The presenter uses that snapshot on the same frame it paints, then writes the formatted overlay back into `HostWindowShellData.debug_refresh_rate` as the next-frame host state. This keeps the `.ui.toml -> runtime layout -> host projection -> native painter` chain intact: no generated Slint UI is restored, and `workbench.rs` still only consumes the final string through the existing overlay call.

The validation focus for this cut is intentionally painter-local: template SVG/image preview pixels, runtime SVG image commands, and top-bar debug overlay visibility. The later B-slice added focused evidence for SVG path aliases, activity-rail icon projection, page/dock tab icon projection, SVG close-button projection, and icon interaction-state pixels: `activity_rail_nodes_project_tab_svg_icons_and_selected_state`, `page_and_dock_tabs_project_svg_icons_and_close_button_icon`, `host_side_activity_rails_use_projected_toml_template_nodes`, `rust_owned_host_painter_resolves_runtime_svg_image_assets`, `rust_owned_host_window_snapshot_renders_template_icon_states`, and `native_host_painter_draws_template_svg_image_pixels` all pass on `D:\cargo-targets\zircon-svg-icons`. Broader Material/Asset Browser layout and text measurement work are separate lanes because active sibling sessions currently own nearby layout/text-input changes.

The Debug Reflector closeout also tightened the host-scene conversion seam that this shell doc owns. `apply_presentation.rs::to_host_contract_pane(...)` no longer treats pane kind strings as the only proof that a native-body payload should survive conversion. If the host-scene pane carries non-empty host-owned native data, such as UI Asset Editor nodes, animation nodes, hierarchy rows, inspector rows, console rows, assets data, asset-browser data, or project overview rows, that payload is preserved before `PaneData.body_surface_frame` is rebuilt. This matters for synthetic or bridged host-scene pane kinds: the native host can carry the real payload while the visible pane kind is not one of the canonical editor kind labels. The fix keeps conversion payload-driven and does not add compatibility aliases for old pane-kind names.

## 2026-05-05 Native Asset Tree Hover State Projection

The hierarchy hover repair exposed the same shared-layer gap in asset panes: native `PointerMoved` routed to `asset_tree_pointer_moved(...)`, app pointer state could compute a hovered row, but `PaneSurfaceHostContext::set_activity_asset_tree_hovered_index(...)` and the browser equivalent still discarded the state. The Rust-owned native painter therefore kept repainting the stored presentation with no asset-row hover overlay even though the event callback had executed.

`HostPaneInteractionStateData` now stores activity and browser asset-tree hover/scroll state beside the hierarchy state. The activity/browser asset tree setters in `PaneSurfaceHostContext` update that shared state and clamp scroll to non-negative pixels, while `HostWindowHandle::take_snapshot()` and presenter redraw continue to compose the live state through `HostWindowPresentationData` without requesting a full frame rebuild.

`painter/workbench.rs` now consumes the projected asset tree state after drawing authored template nodes. It resolves the projected `AssetsActivityTreeRowPanel` / `AssetBrowserSourcesRowPanel` frame, applies the stored scroll offset, and overlays the hovered row inside the pane clip. This keeps asset-tree hover visibility on the same native host state/projection/painter boundary as hierarchy hover instead of adding a callback-only special case.

The focused regression is `native_host_asset_tree_move_updates_visible_hover_state`: it dispatches a native pointer move through an `Assets` pane, drives `set_activity_asset_tree_hovered_index(0)` from the normal move callback, verifies redraw remains local with `requires_frame_update=false`, and asserts that the post-hover native snapshot changes the asset tree row pixels.

## 2026-05-05 Native Hierarchy Hover State Projection

The next visible-interaction regression was lower than pointer routing: a native hierarchy `PointerMoved` callback could run, but `PaneSurfaceHostContext::set_hovered_hierarchy_index(...)` was still a no-op and `HostWindowHandle::take_snapshot()` painted directly from the stale stored presentation. That meant the native event path could request a local redraw while the Rust-owned painter had no updated pane interaction state to consume, so users saw no hover response even though callback dispatch had happened.

`host_contract/data/host_interaction.rs` now adds `HostPaneInteractionStateData` for pointer-only pane state that can repaint native host pixels without forcing a full scene tick. `HostContractState` stores that state beside menu/drag state, `PaneSurfaceHostContext::set_hierarchy_scroll_px(...)` and `set_hovered_hierarchy_index(...)` write it, and `UiHostWindow::get_host_presentation()` plus the snapshot path compose it into `HostWindowPresentationData`. `apply_presentation.rs` preserves the latest pane interaction state when replacing the full scene presentation, matching the existing menu-state overlay behavior.

The same shared state-composition boundary also tightened `HostMenuStateData::default()`: menu and hover indices now default to `-1`, not Rust's derived zero value. Without that sentinel, a fresh Rust-owned host composed live global state as if menu 0 were open, so the closed-menu snapshot already contained the popup and the open-menu overlay regression could not detect a paint-order change. Tests that need an open popup now drive the live `UiHostContext` menu state explicitly while `set_host_presentation(...)` remains a pure stored-presentation update.

`painter/workbench.rs` now draws semantic hierarchy rows when a Hierarchy pane has `hierarchy_nodes`, using the same row metrics as `hierarchy_pointer`. Authored template nodes still paint the pane/header/slot structure first; the native hierarchy row fallback fills the hybrid slot or pane body with selected/hovered row pixels and labels from `SceneNodeData`. This keeps the repair at the shared host-contract state/projection/painter layer instead of special-casing a test callback or forcing full frame updates on mouse move.

The focused regression `native_host_hierarchy_move_updates_visible_hover_state` first failed with unchanged snapshot pixels after `on_hierarchy_pointer_moved` called `set_hovered_hierarchy_index(1)`. After the fix, the same native pointer move still returns local region damage with `requires_frame_update=false`, and the post-hover snapshot changes the child hierarchy row pixels.

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
  - `HostWindowLayoutData` 的 `left/right/bottom_splitter_frame` 现在也由同一份 visible drawer root projection 重建；legacy `WorkbenchShellGeometry.splitter_frames` 只在 splitter 本身不可见或 shared projection 缺席时作为 fallback。这修掉了 1280x750 这类真实窗口下 region 已按 shared drawer frame 展示、但 splitter overlay 仍按 stale legacy geometry 覆盖 Inspector/Console 的 mixed-authority 缝。
  - [`build_workbench_drawer_header_pointer_layout(...)`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/drawer_header_pointer/build_workbench_drawer_header_pointer_layout.rs) 现在在 shared `*DrawerHeaderRoot` 存在时直接复用 header frame，所以 visible drawer header pointer surface 与 root shell presentation 真正合并到同一份 authority
  - shared runtime 现在把 legacy `state_flags.visible=false` 统一规范化成 effective `UiVisibility::Hidden`，而 `HitTestInvisible` / `SelfHitTestInvisible` 分别控制 subtree hit 和 self hit；drawer shell/header projection 不能再用 editor-local bool visible 去重解释 render 与 hit policy。
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
- `EditorSessionMode::Playing`

最近工程配置统一落在 `editor.startup.session`，至少包含：

- `last_project_path`
- `recent_projects`

每次启动都会重新验证最近工程，而不是把上一次的验证结果当权威缓存。失效工程会继续保留在 recent list 中，并在 Welcome 页上显示诊断标签。

### Minimal Play Mode Contract

Unity 式播放模式的第一刀先落在 workbench state 层，然后由菜单、Scene 工具栏和 native runtime backend 复用同一条回滚契约：

- [`EditorSessionMode::Playing`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/startup/editor_session_mode.rs) 表示当前 shell 正在运行播放态快照
- [`EditorState::enter_play_mode()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs) 会捕获进入前的 runtime scene snapshot、选中节点、编辑器 undo/redo 历史和原 session mode，然后清空播放态历史，避免播放中操作混入编辑态 undo 栈
- [`EditorState::exit_play_mode()`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs) 会把世界、选中节点、编辑历史和原 session mode 回滚到进入播放前的状态；播放中创建、删除、重命名或 Inspector 变更都应作为运行态试验丢弃
- Welcome / unloaded world 不能进入播放模式，会返回 `No project open` 并保留原 session mode
- Play 菜单和 Scene toolbar 的 Play/Stop 控件现在都发出稳定 action id，host 先进入/退出 `EditorState` 播放态，再调用 runtime/native plugin play-mode backend；进入后端失败时会回滚编辑态，退出失败时保留回滚后的编辑态并返回诊断

这条契约现在由 [`play_mode_restores_edit_world_and_history_on_exit`](/E:/Git/ZirconEngine/zircon_editor/src/tests/editing/state.rs) 与 [`play_mode_rejects_unloaded_welcome_world`](/E:/Git/ZirconEngine/zircon_editor/src/tests/editing/state.rs) 固定。菜单和工具栏只负责触发同一组 `PlayMode.Enter/Exit` 操作声明，不重新定义播放态回滚规则。

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
- `Runtime Diagnostics`
- `Plugin Manager`

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

也就是说，当前 Scene viewport toolbar 的视觉仍然由 `workbench.slint` 渲染，但真实命中已经切到 `UiSurfaceFrame.hit_grid`，命令解释切到 hit-grid action id + template-runtime fallback，而不是留在 Slint host 的局部 parser、手写坐标表或 legacy callback ABI。

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
- `Project/Assets/Hierarchy/Inspector/Console/Runtime Diagnostics/Plugin Manager/Build Export` pane 已存在
- `Project` 为左侧 active pane
- `Inspector` 默认右侧打开
- `Console` 底部打开
- `Runtime Diagnostics` 默认驻留在右下抽屉并保持折叠，用来承接运行时渲染、物理和动画诊断入口
- `Plugin Manager` 默认驻留在左下抽屉并保持折叠，但通过 View 菜单和 activity view descriptor 稳定可打开
- `Build Export` 默认驻留在右下抽屉并保持折叠，通过 View 菜单和 `editor.build_export_desktop` activity view descriptor 稳定可打开
- `Plugin Manager` 的 pane payload 会随每个插件行投影启用/禁用、打包策略切换、target mode 切换、Unload 和 Hot Reload action id；当前启停与策略动作会通过 `ModulePluginAction` 回写项目 manifest 并刷新诊断，Unload/Hot Reload 会进入 `SlintEditorHost.module_plugin_live_host_backend` 持有的 runtime-owned `NativePluginLiveHost`，对当前项目根内的 editor native package 执行热重载和卸载；同一个 host 也暴露从导出根目录批量加载 runtime/editor native package 的入口，后续 runtime startup 可复用它来持有动态库 handle；缺少已构建动态库或插件 editor behavior 时会返回明确诊断
- runtime startup 的 native dynamic 路径现在返回 `NativePluginRuntimeBootstrap`，把 `CoreHandle`、`NativePluginLiveHost` 和启动诊断放在同一个 bundle 中；导出根目录里实际加载成功的 native library handle 会随 bundle 存活，而不是在注册报告投影后立即释放
- `Plugin Manager` 的 host contract 现在还会投影可视 row/button 节点：每个插件行都有 `ModulePluginRow.<id>` 和一组 `ModulePluginAction` 按钮节点，按钮文字使用紧凑标签，真实动作仍由 stable action id 决定
- `Inspector` 的 host contract 现在会在 `InspectorBodySection` 内补齐可消费的编辑节点：`NameField`、`ParentField`、`PositionXField`、`PositionYField`、`PositionZField`、`ApplyBatchButton` 和 `DeleteSelected` 都带稳定 edit/commit/action id。插件动态组件会从 scene `ComponentTypeDescriptor` 生成 component drawer 字段，字段 control id 固定为 `DynamicComponentField:<component.property>`，可编辑字段随 Apply 批量提交到 `EditorCommand` / Undo / Redo；插件 schema 卸载或缺失时，Inspector 仍显示受保护的只读字段和 warning 诊断，序列化数据不被丢弃。

HTML 原型使用 fixture-shaped data 渲染 builtin preset，并提供 `Project docks right` 的 alternate preset，证明 pane placement 来自 layout JSON，而不是 DOM 写死。

## HTML Skeleton And Slint Mapping

当前 Slint 宿主的组件边界固定为：

- `WorkbenchShell`
- `WorkbenchTopBar`
- `ActivityRail`
- `ToolWindowStack`
- `DocumentWorkspaceHost`
- `WorkbenchStatusBar`
- pane components for `Project`, `Assets`, `Hierarchy`, `Inspector`, `Console`, `Runtime Diagnostics`, `Plugin Manager`, `Build Export`, `Scene`, `Game`, `Prefab Editor`

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

## Latest Build Export Pane Wiring

这一轮补的是 Build Export pane 已经有数据/模板类型、但 presentation 链路没有完全吃进去的缺口：

- [`template_runtime/builtin/mod.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/template_runtime/builtin/mod.rs) 现在把 `PANE_BUILD_EXPORT_BODY_DOCUMENT_ID` 从 builtin template documents 重新导出，`BuildExportPaneBody` descriptor 不再引用一个未暴露的文档 ID。
- [`build_export_view_descriptor.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/host/builtin_views/activity_views/build_export_view_descriptor.rs) 把 `editor.build_export_desktop` 固化成内置 bottom-right 工具视图；默认布局会把它和 runtime diagnostics 放在同一个 bottom-right stack，View 菜单也能通过 `View.BuildExport.Open` 打开。
- [`build_export_desktop_body.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/build_export_desktop_body.ui.toml) 与 [`pane_payload_builders/build_export.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/build_export.rs) 共同提供桌面发布行的模板、payload 和 host contract。
- [`host_data.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs) 固定 `ModulePluginsPaneViewData` 与 `BuildExportPaneViewData` 作为 editor-owned pane DTO；[`app/host_lifecycle.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/host_lifecycle.rs) 在 root host 与 native floating child host 两条 `apply_presentation(...)` 路径都构建并传入这两份 view data，让 `ShellPresentation::from_state(...)`、pane payload builder 和 host contract 使用同一份 Plugin Manager / Desktop Export projection。
- [`name_mapping.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/reflection/name_mapping.rs) 补上 `ViewContentKind::BuildExport` 的 stable reflection 名称，避免新增 pane kind 在反射/远程 surface 名称转换里落到非穷尽匹配。
- [`build_export_actions.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/build_export_actions.rs) 现在把每个桌面目标的 `BuildExport.Execute.<profile>` action 接到 `DesktopExportJobQueue`。队列一次只运行一个后台 export job，默认输出到当前项目下 `Builds/zircon/<profile>`，并把最近一次 `EditorExportBuildReport` 归约成 `desktop_export_reports`。
- 同一个 action parser 也正式接收 `BuildExport.Cancel.<profile>`、`BuildExport.ChooseOutput.<profile>`、`BuildExport.SetOutput.<profile>|<path>`、`BuildExport.ClearOutput.<profile>` 和 `BuildExport.RevealOutput.<profile>`。Queued job 会被直接移出队列；Running job 会进入 cancel-requested 状态，并在 native-aware export backend 返回后丢弃该次结果，避免迟到 report 覆盖用户已取消的 UI 状态。Choose action 通过 [`app/build_export_actions/output_folder.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/build_export_actions/output_folder.rs) 调宿主原生目录选择器并写入同一份 output override；Reveal action 会确保当前输出目录存在，并用宿主 OS 的文件管理器打开它。
- [`pane_data_conversion/build_export.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/pane_data_conversion/build_export.rs) 现在独立承接 Build Export row 投影，避免继续扩大泛用 `pane_data_conversion/mod.rs`。每个桌面目标行会同时投影 `Export/Cancel`、`Choose`、`Open` 和 `Default` 四个 `BuildExportAction` 按钮，行级 actions 与可视按钮消费同一组稳定 action id。
- [`pane_surface_actions.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs) 会把 `BuildExportAction` 控件分派给上面的执行入口；pane contract 根据目标状态在 `Export` 和 `Cancel` 间切换稳定 action id，所以 template/Slint 渲染层不需要解析 profile 名称。

当前 pane 生成 Windows / Linux / macOS 三个桌面目标的 export plan 行，展示目标平台、运行目标、打包策略、启用插件数量、linked runtime crate、NativeDynamic package、生成文件数量、诊断和 fatal 状态。点击行级 `Export` 后，host 会排队后台 native-aware export build；`tick()` 会启动/轮询队列，并把 `Queued`、`Running`、`Cancel requested`、`Exported`、`Failed`、`Cancelled` 这些 coarse 状态、输出目录、generated file/package counts、诊断和 fatal 标记覆盖回下一次 pane projection。runner 还会通过 `EditorExportBuildProgress` 回传阶段/百分比：native package discovery、export plan resolve、native package preparation、materialization、staging cleanup、Cargo build、exported native manifest probe 和 diagnostics 写入都会投影到 active row diagnostics。取消令牌会传入 SourceTemplate Cargo build 和 native dynamic 插件 Cargo build，runner 会轮询取消请求并尽量终止正在运行的构建树：Windows 走 `taskkill /PID <cargo-pid> /T /F`，Unix 让 Cargo 进入独立 process group 后对该组发送强制终止信号，最后保留单进程 kill 作为兜底。自定义输出目录现在同时有 scriptable `SetOutput` contract 和行级原生 `Choose` UI；可视层也能打开当前输出目录并恢复 profile 默认输出目录。

这不是把 export build 执行权迁到插件里；`editor_build_export_desktop` 仍只贡献视图、菜单和 authoring descriptors，实际 plan 生成继续由 host 侧 `EditorManager` / runtime `ExportBuildPlan` 负责。

Milestone 4 inspection confirmed that missing or unreadable project manifests are pane diagnostics, not pane blockers: Plugin Manager falls back to a builtin-catalog `ProjectManifest` and still emits `ModulePluginsPaneViewData`; Desktop Export emits `BuildExportPaneViewData` with an empty target list plus the manifest diagnostic so `editor.build_export_desktop` can still open. Repeated open/refresh behavior stays idempotent because `ViewRegistry::open_descriptor(...)` reuses non-multi-instance descriptors, `restore_or_reuse_instance(...)` preserves builtin `#1` instances, and `TabStackLayout::insert(...)` removes an existing tab id before re-inserting it.

Milestone 4 validation evidence for this shell slice:

- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never` passed with existing warnings.
- `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never` compiled and started the full lib suite but timed out after 15 minutes before a full result.
- Focused plan filters from Milestones 1-3 passed: default layout/menu checks, pane payload checks, Plugin Manager and Desktop Export Slint projection checks, action parsing checks, and Desktop Export queue cancellation.
- `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -TargetDir target\codex-shared-a -VerboseOutput` passed package build and test with existing warnings, so this crate-local validator is the fresh green evidence for the milestone; no workspace-wide green claim is made.

## Latest Menu Operation Dispatch

菜单、工具栏和 builtin template binding 现在共享同一条 editor operation 分派边界：

- [`dispatch.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/common/dispatch.rs) 会把 `EditorUiBindingPayload::EditorOperation` 和可映射的 `MenuAction` 转成 `runtime.invoke_operation(...)`，只有没有 operation path 的 legacy payload 才回退到普通 binding dispatch。
- [`menu_action.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/callback_dispatch/workbench/menu_action.rs) 对 workbench menu action 做同样处理，`CreateCube`、view open、play-mode enter/exit 等路径先进入 `EditorOperationRegistry`，再由 runtime/editor event 层处理副作用。
- [`menu_item_model.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/workbench/model/menu_item_model.rs) 现在把 Build Export 的 View 菜单路径固定为 `View.BuildExport.Open`，与内置 operation descriptor 和 activity view id 对齐。
- [`scene_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 将 branch/leaf 菜单树投影为 native popup 当前仍能消费的扁平 rows：branch row 以 disabled breadcrumb label 表示层级，leaf row 保留 `MenuAction` 或 `EditorOperation` 的 action id。
- [`dispatcher.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_menu_pointer/dispatcher.rs) 的 extension-menu 回归用 `View.Weather.Open` 证明 shared menu pointer 点击不再把 operation id 误解析成 legacy `MenuAction`，而是进入 `EditorOperationRuntime` 并触发布局/presentation dirty。

这条链路让 Scene toolbar 的 Play/Stop 按钮通过 `Runtime.PlayMode.Enter` / `Runtime.PlayMode.Exit` 进入 native runtime play-mode backend，而不是停留在 UI-local 字符串分派。菜单贡献者也因此能把可撤销/不可撤销语义交给 editor operation contract，而不是让每个 pane 自己决定。

## Latest Asset Catalog Error Sync

同一轮验证还补了 `DefaultEditorAssetManager::sync_from_project(...)` 的错误资产路径。项目扫描现在会把缺插件、解析失败或无 artifact 的资源记录成 `ResourceState::Error`；editor catalog 必须展示这些记录和诊断，但不能为了建立引用图去读取不存在的 artifact。

- [`sync_from_project.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/host/editor_asset_manager/manager/project_sync/sync_from_project.rs) 现在只对 `ResourceState::Ready` 记录调用 `load_artifact_by_id(...)` 并解析直接引用；error/reloading 等非 ready 记录保留 catalog entry、diagnostics、meta path 和 preview 状态，`direct_reference_uuids` 为空。
- 回归 `sync_from_project_keeps_error_assets_without_artifacts_in_catalog` 构造一个坏 material TOML，证明 runtime project scan 会产出 Error/no-artifact 记录，而 editor asset manager 仍能同步 catalog。

本轮验证：

- `cargo test -p zircon_editor --lib sync_from_project_keeps_error_assets_without_artifacts_in_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ci-shaped-runtime-interface-gap -- --format terse`
- `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ci-shaped-runtime-interface-gap -- --format terse`
- `cargo test --workspace --locked --verbose --jobs 1` with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-ci-shaped-runtime-interface-gap`

## Latest Inspector Material Surface Controls

Inspector surface controls now consume the shared Material meta component asset instead of keeping placeholder native button rows in the host template. [`inspector_surface_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml) imports [`material_meta_components.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/material_meta_components.ui.toml) and projects Name/Parent as `MaterialLineEdit`, transform axes as `MaterialSpinBox`, and Apply/Delete as `MaterialButton`. The same authored binding ids remain on the `.ui.toml` nodes and are transferred to the expanded Material roots by the shared template runtime, so the editor host continues to receive `DraftCommand.SetInspectorField`, `InspectorFieldBatch`, and `MenuAction.DeleteSelected` through the generic binding route.

This keeps the Inspector on the Slate-style path: `.ui.toml` defines the component, the shared runtime expands and measures it, the arranged tree/hit grid sees a real interactive root, and the native host projection only consumes the resulting control facts.

## Latest Viewport And Pane Material Controls

The M3 editor-layout cutover now applies the same Material meta-component route to the scene viewport toolbar and pane surface action. [`scene_viewport_toolbar.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml) imports `MaterialButton` for the viewport command strip, keeps the existing `ViewportToolbar/...` bindings on each `.ui.toml` control, and relies on shared template expansion to attach the original fixed toolbar layout to the expanded `Button` roots. [`pane_surface_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml) now keeps `PaneSurfaceControls` as the host container while projecting `TriggerAction` as a `MaterialButton` child.

The important contract is that editor chrome no longer needs a Rust coordinate table to make these controls interactive: the authored `.ui.toml` reference carries control id, binding id, Material input metadata, and fixed layout into the shared surface; the native host sees the resulting `Button` frame from the arranged tree.

## 2026-05-06 Native GUI Regression Closure

The current native regression came from several adjacent boundaries failing at once rather than from one pane-local bug. Welcome Material nodes were visible but reached the host without native dispatch metadata, the Rust-owned host event loop did not translate keyboard/IME input into the focused template control, chrome metrics could collapse to zero when a `.ui.toml` chrome template failed to project a usable frame, and tab drag state had no native release path after the pointer moved outside the initial click rectangle.

The fix keeps those responsibilities on the Slate-style chain:

- [`apply_presentation.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) now annotates welcome Material controls with `dispatch_kind`, edit/action ids, role metadata, and current text values before building the pane `body_surface_frame`. `WelcomeProjectNameField` and `WelcomeLocationField` therefore become focusable/editable native hit targets without hard-coded screen coordinates.
- [`window.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/window.rs) routes `KeyboardInput` and `Ime::Commit` through a stored text-focus contract. Inserts, backspace, enter commit, and escape blur update the same host callback route used by pointer-dispatched template controls; the host also toggles platform IME allowance only while a text field is focused. `HostTextInputFocusData` now keeps edit and commit target ids separate, so ordinary text changes use `edit_action_id` while Enter can dispatch `commit_action_id` for apply/batch bindings.
- [`native_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs) focuses text fields on primary press, arms document/drawer/floating tab drags from the actual chrome hit route, forwards drag move/release events to the existing host drag callbacks, and clears stale drag state on release so the painter is not left with a ghost drag payload.
- [`chrome_template_projection.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs) now treats zero-height menu/page/dock metrics and zero-width tab slots as projection failures. It supplies conservative fallback chrome nodes for top menu slots, page tabs, dock tabs, close buttons, subtitles, and project path text so page bars and drawer headers remain visible and hit-testable while the authored `.ui.toml` asset is being repaired.
- Host presentation rebuild logging is sampled instead of emitted on every rebuild. The first few rebuilds and power-of-two counts remain visible for diagnosis, but pointer-only churn no longer floods the log path.

Focused validation for this slice is recorded in the document header. The targeted regressions cover native welcome text input, generic template edit/commit routing, welcome button dispatch, tab drag release/capture cleanup, repeated hover without presentation rebuild, page/dock chrome fallback hit frames, and shared menu operation dispatch. On 2026-05-07, `native_host_generic_template_text_field_routes_commit_binding_on_enter`, `shared_menu_pointer_click_dispatches_editor_operation_payloads_from_extension_menu_items`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never` passed with existing warnings. The full `native_host` filter was not used as acceptance for this slice after an earlier broad run exited with truncated output and then encountered unrelated viewport-image painter noise; the accepted evidence is the focused set plus the editor library check.

## 2026-05-06 Viewport Fast Path Closure

The scene/game viewport body now stays on the runtime image path for pointer-heavy interactions. [`native_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs) still forwards viewport press, release, move, and scroll facts into `PaneSurfaceHostContext::viewport_pointer_event`, but body press/release/scroll return an idle native redraw result just like move already did. The host repaints when [`viewport_image_redraw.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/app/viewport_image_redraw.rs) receives a fresh viewport image and queues a regional redraw for `viewport_content_frame`.

Toolbar controls keep the opposite contract: projected `.ui.toml` toolbar buttons still request visible host feedback on primary press, while release/secondary/middle input remain idle. This separates scene camera/object manipulation from Material toolbar UI and avoids repainting stale viewport pixels before the renderer has produced the next image, which is the path that caused drag ghosting and unnecessary rebuild pressure.

The focused regression `native_host_viewport_button_and_scroll_wait_for_viewport_image_repaint` locks this behavior by asserting that viewport body press, release, and scroll dispatch the correct shared pointer facts without requesting native redraw or increasing the presentation rebuild count.

## 2026-05-06 Live GUI Screenshot Follow-Up

Live validation launched `D:\cargo-targets\zircon-gui-regression-debug\debug\zircon_editor.exe` and captured the welcome/workbench path under `E:\Git\ZirconEngine\.codex\screenshots\20260506-live-gui`. The first direct launch exposed a runtime loader packaging gap: Cargo debug builds place `zircon_runtime.dll` under `debug\deps`, while the app loader only checked the executable sibling path unless `ZIRCON_RUNTIME_LIBRARY` was set. [`library_path.rs`](/E:/Git/ZirconEngine/zircon_app/src/entry/runtime_library/library_path.rs) now keeps the environment override and packaged sibling lookup first, then falls back to executable-sibling `deps/<platform runtime library>` for direct Cargo debug runs. The focused `zircon_app` runtime-library tests cover sibling preference and `debug\deps` fallback.

The live screenshots also reproduced the collapsed drawer bug. Clicking a left rail icon routed through the overlapping drawer-header hit path and set the status line to `Unknown drawer header surface left` instead of expanding the drawer. [`native_pointer.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs) now gives left/right activity rail buttons priority before drawer header tabs, matching the visible rail ownership. The regression lives in [`native_chrome_routing.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/host/slint_window/native_chrome_routing.rs) so the oversized native host contract test file does not keep accumulating unrelated chrome-routing cases.

Screenshot evidence captured before the code rebuild:

- `E:\Git\ZirconEngine\.codex\screenshots\20260506-live-gui\baseline-window.png` showed the welcome page and visible Material fields.
- `E:\Git\ZirconEngine\.codex\screenshots\20260506-live-gui\welcome-input-after-ime-commit.png` showed real keyboard input committed into `Project name`, with location/path auto-filled and `Create Project` enabled.
- `E:\Git\ZirconEngine\.codex\screenshots\20260506-live-gui\editor-after-recent-project-printwindow.png` showed the workbench opening into the Scene/Inspector layout.
- `E:\Git\ZirconEngine\.codex\screenshots\20260506-live-gui\after-left-drawer-click.png` captured the rail misroute status line before the priority fix.

Validation completed in this pass:

- `cargo fmt --all`
- `cargo test -p zircon_app --lib runtime_library --features target-editor-host --locked --jobs 1 --target-dir D:\cargo-targets\zircon-gui-regression-debug --message-format short --color never` passed with 4 runtime-loader tests.

Editor-side focused validation was blocked by local build conditions rather than by the new chrome-routing source: the running editor initially held `zircon_runtime.dll` and produced `LNK1104`, then the interrupted target dir reported a missing incremental dep-info path, and a fresh target spent 15 minutes cold-compiling while another active session was linking `zircon_editor` SVG icon tests. The next validation step is to rerun `cargo test -p zircon_editor --lib native_host_activity_rail_click_wins_over_overlapping_drawer_header ...` and rebuild the GUI once those competing editor test links finish.

## 2026-05-06 Adaptive SVG Icon Rasterization

SVG icons in the Rust-owned host painter now follow the actual draw rectangle instead of being decoded once at the source image size and later stretched as a bitmap. [`visual_assets.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs) resolves the same editor asset candidates as before, but `.svg` paths are rendered through `resvg` with a cache key that includes the requested pixel width/height and tint. Ordinary bitmap images keep the existing Slint decode path.

[`template_nodes.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs) now computes the final icon/image frame first, then requests SVG pixels at that size. This keeps menu icons, tab icons, rail icons, leading button icons, disabled/active Material icon tinting, and non-icon preview images on the same authored `.ui.toml` projection path. [`render_commands.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs) applies the same target-size request for runtime image/vector brush resources, so shared UI render commands also keep vector sharpness when the frame changes.

Validation for this slice:

- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\svg-adaptive-check --message-format short --color never` passed with existing warnings.
- `cargo test -p zircon_editor --lib svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets\svg-adaptive-check --message-format short --color never` passed 2 targeted tests.

## 2026-05-07 Native Material Theme Palette

The Rust-owned host painter now has a small Material palette module instead of spreading shell chrome fallback colors across the large workbench painter. [`painter/theme.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/painter/theme.rs) mirrors the token values from [`editor_material.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/theme/editor_material.ui.toml) for shell background, surface, inset, hover, pressed, selected, disabled, accent, accent-soft, border, text, muted text, disabled text, warning, popup, track, and focus-ring colors. The palette stays private to the painter subtree; it is not a public host-contract API.

[`template_nodes.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/painter/template_nodes.rs) consumes that palette only after shared template projection has already supplied arranged frames and visual state. The order is explicit: disabled wins, then pressed, selected/focused, primary/accent, hover/drop target, and only then authored `surface_variant` such as `inset` or `popup`. The generic Button hover fallback is restricted to buttons with no authored `surface_variant`, so a Material control that explicitly asks for an inset surface is not recolored by the fallback path.

[`workbench.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/painter/workbench.rs), [`render_commands.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/painter/render_commands.rs), and [`visual_assets.rs`](/E:/Git/ZirconEngine/zircon_editor/src/ui/slint_host/host_contract/painter/visual_assets.rs) use the same palette for shell chrome, command fallback colors, and icon tint. This keeps native fallback pixels aligned with the `.ui.toml` theme without adding screen-specific Material branches or Asset Browser-specific color tables.

Validation for this slice:

- `cargo test -q -p zircon_editor --lib native_template_painter_uses_material_state_palette_for_controls --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui -- --nocapture` passed with existing warnings.
- `cargo test -q -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui -- --nocapture` passed with existing warnings.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\global-ui --message-format short --color never` passed with existing warnings.

## 2026-05-07 Editor Visual Density B

The selected visual direction B keeps the editor in a dense tool-surface mode instead of inheriting large general-purpose Material defaults. [`material_meta_components.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/material_meta_components.ui.toml) now anchors compact/default/prominent heights at 28/32/40, uses 16px button icons and 18px standard icons, and reduces button/list/field padding so Material roots remain readable without swelling panel chrome.

Workbench chrome follows the same scale. [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) uses a 44px activity rail with 32px icon buttons and a 24px menu action strip, while [`workbench_activity_rail.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/workbench_activity_rail.ui.toml) centers 18px Ionicons inside 32px rail buttons for the authored stencil. The menu chrome templates keep a 24px top bar, and [`inspector_surface_controls.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml) lowers its Material line edits, spin boxes, and Apply/Delete actions to 28px.

The focused guard lives in [`material_meta_component_contracts.rs`](/E:/Git/ZirconEngine/zircon_editor/src/tests/ui/boundary/material_meta_component_contracts.rs). It prevents future regressions where standard icons return to 30px, rail buttons return to 40px, or menu actions exceed the menu bar height.
