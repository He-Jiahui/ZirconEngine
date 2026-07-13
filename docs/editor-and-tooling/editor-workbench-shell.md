---
related_code:
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/tests.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_segmented_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_selection_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_slider.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_architecture.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_floor.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_light.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_structure.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_surfaces.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/popup_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/property_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/surface.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs
  - zircon_editor/src/ui/retained_host/tab_drag/bridge.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_runtime_interface/src/ui/component/category.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_model.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
  - zircon_editor/assets/ui/editor/host/floating_window_source.zui
  - zircon_editor/assets/ui/editor/host/inspector_surface_controls.zui
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\data\workbench_component_property_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\chrome\workbench_axis_value_field.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_component_drawer.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\inputs\workbench_dropdown.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\inputs\workbench_segmented_control.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_activity_rail.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_inspector_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\chrome\workbench_rail_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\data\workbench_list_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\feedback\workbench_popup_menu.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\chrome\workbench_chip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\chrome\workbench_section_title.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\inputs\workbench_icon_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_status_bar.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\feedback\workbench_status_item.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\data\workbench_table_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\feedback\workbench_tooltip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\data\workbench_tree_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_viewport_panel.zui
  - zircon_editor/assets/ui/theme/editor_workbench_strict.zui
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_ensure_viewport.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/tests.rs
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/frame_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_ensure_viewport.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_popup_row.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_segmented_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_selection_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_slider.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_status_control.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_architecture.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_floor.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_light.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_structure.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_surfaces.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/popup_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/property_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/drawer_layout.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/bridge.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/floating_window_source/surface.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_runtime_interface/src/ui/component/category.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_model.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
  - zircon_editor/assets/ui/editor/host/inspector_surface_controls.zui
  - zircon_editor/assets/ui/editor/host/startup_welcome_controls.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\data\workbench_component_property_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\chrome\workbench_axis_value_field.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_component_drawer.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\inputs\workbench_dropdown.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\inputs\workbench_segmented_control.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_activity_rail.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_inspector_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\chrome\workbench_rail_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/index/workbench_additional_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\data\workbench_list_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\feedback\workbench_popup_menu.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\chrome\workbench_chip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\chrome\workbench_section_title.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\inputs\workbench_icon_button.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_status_bar.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\feedback\workbench_status_item.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\data\workbench_table_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\feedback\workbench_tooltip.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives\data\workbench_tree_row.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell\workbench_viewport_panel.zui
  - zircon_editor/assets/ui/theme/editor_workbench_strict.zui
plan_sources:
  - .codex/plans/Zircon Editor Runtime UI Rust-Owned Retained Host 重构计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - .codex/plans/Editor 基础组件 Material 化视觉优化计划.md
  - user: 2026-05-15 optimize retained editor UI styling with Material-like rounded controls and stronger feedback
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - user: 2026-07-11 continue editor-default Hybrid GI viewport activation
tests:
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/workbench_projection_cutover.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_layout_paths.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract.rs
  - zircon_editor/src/tests/host/template_runtime/host_window_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_window_menus.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_creates_and_resizes_render_framework_viewports.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/popup_primitives.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests/actions.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/floating_window_source.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/workbench/template_bridge.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls.rs
  - zircon_editor/tests/integration_contracts/workbench_retained_shell.rs
  - cargo check -p zircon_editor --lib --locked --message-format=short
  - cargo check -p zircon_editor --lib --tests --locked --message-format=short
  - cargo test -p zircon_editor --lib native_root_menu_pointer_click_dispatches_shared_menu_action_in_real_host --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib native_frame_request_recomputes_dirty_layout_before_presentation --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib child_window_viewport_pointer_event_focuses_source_window_before_runtime_dispatch --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib --locked --target-dir target\codex-shared-a -- --test-threads=1
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -TargetDir target\codex-shared-a (2026-05-09: workspace build passed, workspace test blocked in zircon_plugin_navigation_runtime --lib)
  - cargo test -p zircon_plugin_navigation_runtime --lib --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1 (2026-05-09: reproduced external navigation runtime blocker, 5 passed / 8 failed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --message-format short --color never (3 passed)
  - 2026-05-15 Material visual slice: cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --message-format short --color never (25 passed)
  - 2026-05-15 Material visual slice: cargo check -p zircon_editor --lib --tests --locked --jobs 1 --message-format short --color never (passed)
  - 2026-05-16 Material visual live capture: tools/ui-profile-capture.ps1 -ScenarioList startup,idle_hover -OutputRoot .codex/material-ui-capture -SkipBuild -AutoCloseSeconds 5 -AutoInteract -RequireScenarioEvidence (startup 20260516-000244 passed; idle_hover 20260516-000253 recorded redraw/GPU work with zero alerts but missed the strict batch gate)
  - 2026-05-16 Material visual live capture: tools/ui-profile-capture.ps1 -Scenario click -OutputRoot .codex/material-ui-capture -SkipBuild -AutoCloseSeconds 5 -AutoInteract -RequireScenarioEvidence (20260516-000343 passed)
  - 2026-05-16 Material feedback emphasis: rustfmt --edition 2021 --check retained painter theme/template_nodes/native_material_painter test files (passed)
  - 2026-05-16 Material feedback emphasis: Python tomllib parse for editor_material.zui, ui_zui editor_material.zui, and component_showcase.zui (passed)
  - 2026-05-16 Material feedback emphasis: cargo test -p zircon_editor --lib native_material_painter --locked --jobs 1 --message-format short --color never (blocked before compile by existing Cargo.lock mismatch; lockfile left unchanged)
  - 2026-06-02 Workbench Inspector component property split rows: rustfmt --edition 2021 --check over the Workbench property bridge, painter, and focused tests (passed)
  - 2026-06-02 Workbench Inspector component property split rows: Python tomllib parse of workbench_component_property_row.zui, workbench_inspector_panel.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench Inspector component property split rows: focused Cargo validation deferred while unrelated zircon_editor/zircon_hub Cargo lanes were already active
  - 2026-06-02 Workbench table-row split columns: rustfmt --edition 2021 --check over the table painter/projection/focused tests (passed)
  - 2026-06-02 Workbench table-row split columns: Python tomllib parse of workbench_table_row.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench table-row split columns: focused Cargo validation deferred while another zircon_editor Cargo lane was active
  - 2026-06-02 Workbench table-row full primitive: added native row surfaces, selected/tail/header tones, separators, column text, and right-side action glyphs in template_table_rows.rs
  - 2026-06-02 Workbench table-row full primitive: rustfmt --edition 2021 --check over template_table_rows.rs and template_nodes.rs (passed)
  - 2026-06-02 Workbench table-row full primitive: Python tomllib parse of workbench_table_row.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench table-row full primitive: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the touched Rust/docs/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench table-row full primitive: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench table-header text sync: template_table_rows.rs now paints WorkbenchTableHeader cells with the audited #aab5ba text tone, and editor_workbench_strict.zui exposes the same table-header text token
  - 2026-06-02 Workbench table-header text sync: rustfmt --edition 2021 --check over template_table_rows.rs/template_status_controls.rs/template_alerts.rs, Python tomllib parse of workbench_table_row.zui/workbench_component_drawer.zui/editor_workbench_strict.zui, and trailing-whitespace scan over related files (passed; focused Cargo not attempted because cargo/rustc lanes were active)
  - 2026-06-02 Workbench table-header content offset: TemplatePaneNodeData and both retained projection paths now carry layout_content_offset_x/y, WorkbenchTableHeader declares -1.0 / 3.0, and template_table_rows.rs applies the offset only to header cell text plus the gear glyph. rustfmt --edition 2021 --check, Python tomllib parsing for workbench_component_drawer.zui/editor_workbench_strict.zui, tracked editor-doc git diff --check, and a trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench table-tail native offsets: TemplatePaneNodeData and both retained projection paths now carry layout_first/second/third/fourth_cell_offset_x, WorkbenchTableTail declares the audited row/content/cell offsets and tones, and template_table_rows.rs applies them only to the tail row; rustfmt --edition 2021 --check, Python tomllib parsing for workbench_component_drawer.zui, tracked git diff --check, and a trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench table-tail fourth-cell color sync: workbench_window_projection.rs mapped fourth_cell_text_color into TemplatePaneNodeData.value_color, workbench_projection.rs asserted the then-current #a6b0b5 value, and template_table_rows.rs consumed declared value_color for WorkbenchTableTail fourth-cell text before falling back to the audited default. rustfmt --edition 2021 --check and Python tomllib parsing for the touched table assets passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-27 Runtime 15 M3 editor retained-host workbench window projection tests child-owner split (`runtime_15_editor_retained_host_workbench_window_projection_tests_child_owner_split_static_passed_cargo_deferred`): `zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs` now delegates authored-label/value-text/segmented-value projection tests to `zircon_editor/src/ui/retained_host/ui/workbench_window_projection/tests.rs`; guard `runtime_15_editor_retained_host_workbench_window_projection_tests_are_child_owner` locks the child-owner layout and closes the current large-file hotspot.
  - 2026-06-27 Runtime 15 M3 editor retained-host pane data conversion projection owner guard (`runtime_15_editor_retained_host_pane_data_conversion_owner_guard_static_passed_cargo_deferred`): `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs` stays a thin projection root while `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs` owns `project_nodes<T, F>`, `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/animation_projection.rs` owns animation payload projection, and `zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs` owns pane routing; guard `runtime_15_editor_retained_host_pane_data_conversion_uses_child_projection_owners` locks the F15 owner split.
  - 2026-06-03 Workbench table-tail Modified-column tone sync: WorkbenchTableTail now declares fourth_cell_text_color = #aab5ba, workbench_projection.rs asserts the projected value_color, and template_table_rows.rs uses the same #aab5ba fallback for the tail fourth cell while preserving all row/cell offsets and selection behavior
  - 2026-06-03 Workbench table-tail Modified-column tone sync: rustfmt --edition 2021 --check over template_table_rows.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; stale marker scan, targeted git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench table normal-row group/selection sync: workbench_component_drawer.zui now adds WorkbenchTableItem and wraps the four lower-table rows in transparent WorkbenchTableGroup with gap 0.0; componentized_window.rs routes SelectComponentLabTableItem through the existing component-lab selected-table group; workbench_projection.rs asserts the Item_01 row data, 4 px normal-row inset, zero-gap row adjacency, and exclusive click selection. rustfmt --edition 2021 --check, Python tomllib parsing, source-marker checks, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench list-row group layout sync: workbench_component_drawer.zui wraps WorkbenchListItem, WorkbenchListSelected, and WorkbenchListDisabled in transparent WorkbenchListGroup with gap 0.0; workbench_projection.rs asserts adjacent retained frames for the three rows. rustfmt --edition 2021 --check, Python tomllib parsing, source-marker checks, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench selection controls native painter: rustfmt --edition 2021 --check over template_selection_controls.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench selection controls native painter: Python tomllib parse of workbench_checkbox.zui, workbench_radio.zui, workbench_toggle.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench selection controls native painter: focused Cargo validation deferred while zircon_editor, zircon_runtime, and zircon_hub Cargo lanes were active
  - 2026-06-02 Workbench checkbox/radio mark metrics: template_selection_controls.rs now paints 16 px marks, 9 px mark-label gap, #828c93 labels, #141a1e/#424e56 idle marks, #209fa8 checked checkbox fill/border, #1b272d/#4c5b63 checked radio shell, and a 7 px radio dot; editor_workbench_strict.zui exposes matching selection tokens
  - 2026-06-02 Workbench checkbox/radio mark metrics: rustfmt --edition 2021 --check over template_selection_controls.rs and Python tomllib parse of workbench_checkbox.zui, workbench_radio.zui, and editor_workbench_strict.zui (passed; focused Cargo deferred while active workspace cargo/rustc lanes were compiling)
  - 2026-06-02 Workbench selection-control declared metrics projection, updated 2026-06-30: WorkbenchCheckboxOn/Off and WorkbenchRadioOn/Off explicitly project layout_icon_size = 16.0, layout_spacing = 9.0, and label_color = #828c93; radio now defaults to a 5 px accent dot, while WorkbenchToggleOn projects track_width = 34.0, track_height = 18.0, thumb_size = 12.0, and layout_spacing = 10.0. workbench_projection.rs asserts the retained leaf values and strict-theme mark colors; pane_component_projection/tests.rs locks runtime_component_projection_maps_workbench_metric_aliases; and template_selection_controls.rs consumes declared mark, track, thumb, label, and mark-label gap metrics before fallback constants
  - 2026-06-02 Workbench selection-control declared metrics projection: rustfmt --edition 2021 --check over workbench_window_projection.rs, pane_component_projection/mod.rs, pane_component_projection/tests.rs, template_selection_controls.rs, and workbench_projection.rs; Python tomllib parse of workbench_component_drawer.zui/workbench_checkbox.zui/workbench_radio.zui/workbench_toggle.zui/editor_workbench_strict.zui passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Checkboxes & Radios native tone/gap parity: workbench_checkbox.zui, workbench_radio.zui, workbench_component_drawer.zui, editor_workbench_strict.zui, template_selection_controls.rs, and workbench_projection.rs now mirror the accepted HTML/CSS tone pass: 9 px mark-label gap, #828c93 labels, #424e56 idle borders, #209fa8 checked checkbox fill, #1b272d/#4c5b63 checked radio shell, and a 7 px #43d8e2 radio dot. rustfmt --edition 2021 --check, Python tomllib parsing, and touched-file git diff --check passed; focused Cargo stayed deferred because active cargo/rustc lanes were still compiling.
  - 2026-06-03 Workbench Toggle tone native sync: editor_workbench_strict.zui now exposes idle track/border/thumb and checked track/border/thumb tokens for .workbench-toggle; template_selection_controls.rs consumes declared background/foreground/border for toggle track, thumb, and edge tones before fallback constants; workbench_projection.rs asserts the checked projected colors
  - 2026-06-03 Workbench Toggle tone native sync: rustfmt --edition 2021 --check over template_selection_controls.rs and workbench_projection.rs passed; Python tomllib declaration assertions for editor_workbench_strict.zui and workbench_component_drawer.zui passed; stale marker scan, git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench segmented/tab native painter: rustfmt --edition 2021 --check over template_segmented_controls.rs, template_selection_controls.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench segmented/tab native painter: Python tomllib parse of workbench_segmented_control.zui, workbench_tab.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench segmented/tab native painter: focused Cargo validation deferred while zircon_editor and zircon_hub Cargo lanes were active
  - 2026-06-02 Workbench Icon Toggle selected segment native sync: TemplatePaneNodeData now carries optional selected segment border width, underline height, and underline color; both retained projection paths map those `.zui` properties; WorkbenchIconToggleSegmented declares 0 px selected border plus a 1 px #32d3de7a underline; template_segmented_controls.rs consumes the declared values while preserving the legacy 1 px selected border default for undeclared segmented controls. rustfmt --edition 2021 --check and Python tomllib parsing passed. `cargo check -p zircon_editor --lib --locked --message-format=short` was attempted and failed before reaching editor-specific errors in the active zircon_runtime render graph path (`depth_attachment_ops` and render executor argument-count errors), matching the separate WGPU/render-main workstream.
  - 2026-06-02 Workbench Labs tabs native sync: workbench_component_drawer.zui now declares WorkbenchLabsTabOne/Two/Three with the accepted 3 px / 2 px tab offset, template_segmented_controls.rs applies layout offsets to Workbench tabs, and ComponentLabPreview routes switch Labs tab state plus the existing table-tail preview selection; rustfmt --edition 2021 --check, Python tomllib parsing for workbench_component_drawer.zui/workbench_tab.zui, tracked git diff --check, and a trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench segmented label projection: TemplatePaneNodeData now carries label text/color/brightness plus body layout offsets, WorkbenchInputSegmented projects those props, and template_segmented_controls.rs paints a 48 px label/body stack
  - 2026-06-02 Workbench segmented label projection: rustfmt --edition 2021 --check over the projection/painter/focused-test Rust files (passed)
  - 2026-06-02 Workbench segmented label projection: Python tomllib parse of workbench_component_drawer.zui, workbench_segmented_control.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench segmented label projection: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over touched Rust/docs/ZUI/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench segmented label projection: focused Cargo validation deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench segmented shell fill: template_segmented_controls.rs and editor_workbench_strict.zui now use the audited #1d2327 idle body shell while preserving hover/pressed/disabled state colors
  - 2026-06-02 Workbench segmented shell fill: rustfmt --edition 2021 --check over template_segmented_controls.rs, Python tomllib parse of workbench_component_drawer.zui/workbench_segmented_control.zui/editor_workbench_strict.zui, git diff --check, and trailing-whitespace scan (passed; focused Cargo deferred while active workspace cargo/rustc lanes were compiling)
  - 2026-06-02 Workbench Labs structural ownership: workbench_component_drawer.zui now mounts WorkbenchInputSegmented and WorkbenchToggleOn under WorkbenchComponentLabs, removes those samples from the Inputs and Checkboxes/Radios children, and keeps both existing control ids plus ComponentLab preview routes stable
  - 2026-06-02 Workbench Labs structural ownership: rustfmt --edition 2021 --check over workbench_projection.rs, Python tomllib parse of workbench_component_drawer.zui, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo was skipped because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench list/menu adornment native painter: rustfmt --edition 2021 --check over template_list_rows.rs, template_popup_rows.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench list/menu adornment native painter: Python tomllib parse of workbench_list_row.zui, workbench_popup_menu.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench list/menu adornment native painter: focused Cargo validation deferred while active zircon_editor/zircon_hub Cargo and rustc lanes were compiling
  - 2026-06-02 Workbench popup/dropdown state split: select_dropdown_option, select_popup_menu_item, toggle_popup, and transient menu flag cleanup now live in popup_state.rs instead of componentized_window.rs; rustfmt --edition 2021 --check, source-marker checks, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench popup/dropdown primitive split: popup_primitives.rs now owns reusable TOML string-list conversion, popup menu item state parsing, and transient menu flag cleanup; Workbench popup_state.rs and componentized_window.rs reuse it instead of carrying duplicate local helpers
  - 2026-06-02 Workbench popup/dropdown primitive split: rustfmt --edition 2021 --check over template_bridge/mod.rs, popup_primitives.rs, componentized_window.rs, and popup_state.rs passed; Python tomllib parsing for Workbench dropdown/popup assets and theme passed; recursive source scan found the helper definitions centralized in popup_primitives.rs; cargo check -p zircon_editor --lib --locked --message-format=short passed with existing warnings
  - 2026-06-02 Workbench frame/template geometry split: host_contract/frame_geometry.rs now owns generic FrameRect visibility, containment, and union helpers; host_contract/template_geometry.rs now owns only template node frame conversion and popup-bounds fallback; native keyboard, popup dismiss, template-node hit testing, native pointer damage, and redraw region merging reuse those focused modules instead of the former Workbench-named or duplicated geometry helpers
  - 2026-06-02 Workbench frame/template geometry split: rustfmt --edition 2021 --check over host_contract/mod.rs, frame_geometry.rs, template_geometry.rs, native_keyboard.rs, native_popup_dismiss.rs, native_pointer.rs, redraw.rs, and surface_hit_test/template_node.rs passed; recursive source scan confirmed the old workbench_popup_geometry/workbench_popup_bounds names are gone and native_pointer.rs/redraw.rs no longer define local union/visible helpers; focused Cargo was deferred while external cargo/rustc lanes were active
  - 2026-06-02 Workbench virtual-row namespace contract: workbench_projection.rs now asserts WorkbenchSceneTree and WorkbenchInspectorMesh repeat declarations keep their virtual row metadata and node_path_namespace = "v2" aligned
  - 2026-06-02 Workbench virtual-row namespace contract: rustfmt --edition 2021 --check over workbench_projection.rs, source-marker checks, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench toast declared style sync: both retained projection paths now map status_mark_size, status_mark_color, and action_color into TemplatePaneNodeData; template_alerts.rs consumes those values for WorkbenchToastRoot mark size, mark fill, and UNDO action color; workbench_projection.rs plus pane_component_projection/tests.rs lock the projected contract
  - 2026-06-02 Workbench toast declared style sync: rustfmt --edition 2021 --check over touched projection/painter/test files, Python tomllib parsing for workbench_toast.zui/workbench_component_drawer.zui/editor_workbench_strict.zui, and source-marker checks passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench tooltip native sync: workbench_tooltip.zui now declares WorkbenchTooltipRoot, workbench_component_drawer.zui inserts it after the alert stack, both retained projection paths map arrow_size/arrow_color, and template_tooltips.rs paints the native bubble, arrow, text, shadow, and info mark before generic fallback
  - 2026-06-02 Workbench tooltip native sync: rustfmt --edition 2021 --check over touched projection/painter/test files passed; Python tomllib parse of workbench_tooltip.zui, workbench_component_drawer.zui, workbench_window.zui, and editor_workbench_strict.zui passed; source-marker checks, git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench tooltip tone follow-up: workbench_tooltip.zui, the drawer-mounted WorkbenchTooltipRoot, template_tooltips.rs fallback constants, pane_component_projection/tests.rs, and workbench_projection.rs now mirror the latest tooltip tones: border #252d32, secondary text #a8b3b8, and info mark #259ca7
  - 2026-06-02 Workbench tooltip tone follow-up: rustfmt --edition 2021 --check over the touched tooltip painter/projection test files passed; Python tomllib parsing/assertions for workbench_tooltip.zui and workbench_component_drawer.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused cargo test -p zircon_editor --lib template_tooltips --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 timed out after 184 seconds without a pass/fail result, and the post-timeout process scan showed only unrelated active Cargo lanes
  - 2026-06-02 Workbench side-stack popup menu width: WorkbenchPopupMenu in workbench_component_drawer.zui now uses a fixed 145 px layout, and workbench_projection.rs asserts the projected host frame width
  - 2026-06-02 Workbench side-stack popup menu width: rustfmt --edition 2021 --check, Python tomllib parse of touched Workbench component assets/theme, git diff --check, and trailing-whitespace scan (passed; focused Cargo deferred while active workspace cargo/rustc lanes were compiling)
  - 2026-06-02 Workbench toast surface/border sync: template_alerts.rs now mirrors the HTML/CSS audited toast surface and 0.08 teal border, and editor_workbench_strict.zui exposes matching toast surface/border tokens
  - 2026-06-02 Workbench toast surface/border sync: rustfmt --edition 2021 --check over template_alerts.rs and Python tomllib parse of workbench_toast.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench toast surface/border sync: focused cargo test -p zircon_editor --lib template_alerts --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 was attempted after the cargo/rustc process list was clear, but timed out after 120 seconds before a pass/fail result; leftover cargo/rustc processes from that attempt were stopped
  - 2026-06-02 Workbench status controls native painter: rustfmt --edition 2021 --check over template_status_controls.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench status controls native painter: Python tomllib parse of workbench_status_bar.zui, workbench_status_item.zui, workbench_chip.zui, workbench_icon_button.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench status controls native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new status painter plus docs/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench status controls native painter: focused Cargo validation deferred while active zircon_editor/zircon_hub Cargo and rustc lanes were compiling
  - 2026-06-02 Workbench status-ready declaration sync: WorkbenchStatusReady now projects declared 4 px / -1 px item offset, 8 px gap, #9ba7ad text, #4eaa5f ready dot fill, and 9 px dot size into the retained status painter
  - 2026-06-02 Workbench status-ready declaration sync: rustfmt --edition 2021 --check over template_status_controls.rs and workbench_projection.rs, Python tomllib parse of workbench_status_bar.zui and workbench_status_item.zui, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo was skipped because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench status-warning declaration sync: browser-rendered warning candidates were rejected because the current CSS stayed best at status-warnings 33.74 and statusbar 22.74 close; template_status_controls.rs now consumes WorkbenchStatusWarnings declared text color, icon fill, icon size, item offset, icon offset, and layout gap
  - 2026-06-02 Workbench status-warning declaration sync: rustfmt --edition 2021 --check over the touched projection/painter/test files, Python tomllib parse of workbench_status_bar.zui, node interaction/responsive validators, focused status/pixel audits, and docs/session updates passed; focused Cargo was skipped because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench status-right chip declaration sync: template_status_controls.rs now offsets status chips by declared layout offsets and consumes projected text_color/value_color for chip labels and chevrons; workbench_projection.rs asserts WorkbenchStatusGrid text color and y offset
  - 2026-06-02 Workbench status-right parent defaults projection: workbench_status_bar.zui keeps status_right_offset_y and status_right_text_color on WorkbenchWindowStatusBar, removes duplicate leaf text_color/layout_offset_y from the right chips/icons, and workbench_window_projection.rs inherits those defaults only for WorkbenchStatusGrid/Snap/Zoom plus SnapToggle/World/Target through parent_id; workbench_projection.rs asserts inherited Grid/Snap color and Target offset
  - 2026-06-02 Workbench status-right parent defaults projection: rustfmt --edition 2021 --check over workbench_window_projection.rs and workbench_projection.rs, Python tomllib parse of workbench_status_bar.zui, source-marker assertions, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench status-message/right-icon declaration sync: WorkbenchStatusMessages now projects declared message-row offsets, text color, info fill, icon size, and icon offset; template_status_controls.rs applies the same offset helper to status snap/world/target icon buttons
  - 2026-06-02 Workbench status-message/right-icon declaration sync: rustfmt --edition 2021 --check over template_status_controls.rs and workbench_projection.rs, Python tomllib parse of workbench_status_bar.zui and workbench_status_item.zui, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo was skipped because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench status-right border sync: template_status_controls.rs now paints normal status-right chip/icon borders as #242c32, and editor_workbench_strict.zui exposes matching status-right border tokens/classes while preserving focus-ring state borders
  - 2026-06-02 Workbench status-right border sync: rustfmt --edition 2021 --check over template_status_controls.rs/template_alerts.rs, Python tomllib parse of workbench_status_bar.zui/workbench_status_item.zui/workbench_toast.zui/editor_workbench_strict.zui, git diff --check over the tracked editor-workbench doc, and trailing-whitespace scan over related Rust/ZUI/theme/docs/session files (passed; focused Cargo not attempted because cargo/rustc lanes were active)
  - 2026-06-02 Workbench No Errors icon fill sync: template_status_controls.rs now paints WorkbenchStatusErrors with the audited #58b866 icon fill, workbench_status_bar.zui records icon_fill = "#58b866", and editor_workbench_strict.zui exposes workbench_status_no_errors_fill
  - 2026-06-02 Workbench No Errors icon fill sync: rustfmt --edition 2021 --check over template_status_controls.rs, Python tomllib parse of workbench_status_bar.zui/workbench_status_item.zui/editor_workbench_strict.zui, and trailing-whitespace scan over the related files (passed; focused Cargo not attempted because cargo/rustc lanes were active)
  - 2026-06-02 Workbench No Errors mark foreground sync: TemplatePaneNodeData now carries icon_color, both retained projection paths initialize it, workbench_window_projection.rs maps icon_color/icon_stroke from WorkbenchStatusErrors, and template_status_controls.rs consumes the declared #112018 color for the success check mark before falling back to the generic dark mark tone
  - 2026-06-02 Workbench No Errors mark foreground sync: rustfmt --edition 2021 --check over the touched DTO/projection/status-painter/test files passed; Python tomllib parsing, trailing-whitespace, and tracked diff checks are the light gate for this sync while focused Cargo remains deferred until active cargo/rustc lanes clear
  - 2026-06-03 Workbench No Errors icon visual scale native sync: WorkbenchStatusErrors declares layout_icon_size = 12.04; template_status_controls.rs keeps the outer 14 px status icon layout slot for label placement and uses the declared size only for the centered success mark paint rect, while workbench_projection.rs asserts the projected value
  - 2026-06-03 Workbench No Errors icon visual scale native sync: rustfmt --edition 2021 --check over template_status_controls.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_status_bar.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench Warning icon mark native sync: WorkbenchStatusWarnings declares icon_color/icon_stroke = #11181a; template_status_controls.rs consumes that foreground for the warning triangle's internal mark without changing the 21 px icon slot, and workbench_projection.rs asserts the projected mark color
  - 2026-06-03 Workbench Checkboxes/Radios unchecked mark native sync: WorkbenchCheckboxOff and WorkbenchRadioOff declare background_color = #13191d and border_color = #374148; template_selection_controls.rs consumes those declared idle mark tones without changing selected states, label tones, or row spacing, and workbench_projection.rs asserts both projected style colors
  - 2026-06-03 Workbench Cast Shadows select native sync: Inspector data sync applies background_color = #282e32, border_color = #343d43, and value_color = #b5c0c5 only when the dynamic component-property field id is cast_shadows; non-Cast rows clear those row-level style props, and template_inspector_rows.rs consumes the declared select field/value tones before fallback colors
  - 2026-06-03 Workbench Ready status text native sync: WorkbenchStatusReady declares text_color = #8f9aa0; template_status_controls.rs consumes the projected Ready label tone without changing the 9 px dot, 8 px gap, item offsets, or following status item spacing
  - 2026-06-02 Workbench Mesh Renderer title-tone sync: template_section_titles.rs now paints WorkbenchMeshLabel with the audited #b0babf title tone, workbench_inspector_panel.zui marks that heading with workbench-mesh-title, and editor_workbench_strict.zui exposes workbench_mesh_title_text
  - 2026-06-02 Workbench Mesh Renderer title-tone sync: rustfmt --edition 2021 --check over template_section_titles.rs, Python tomllib parse of workbench_inspector_panel.zui and editor_workbench_strict.zui, git diff --check over the tracked editor-workbench doc, and trailing-whitespace scan over the related Rust/ZUI/theme/docs/session files (passed; focused Cargo not attempted because cargo/rustc lanes were active)
  - 2026-06-02 Workbench scene tree native painter: rustfmt --edition 2021 --check over template_tree_rows.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench scene tree native painter: Python tomllib parse of workbench_tree_row.zui, workbench_scene_tree_panel.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench scene tree native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new tree painter plus docs/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench scene tree native painter: focused Cargo attempted with cargo test -p zircon_editor --lib template_tree_rows --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1, but compilation stopped in existing zircon_runtime graphics scene renderer code before these tests ran; errors were missing RenderPassMeshDrawLists.non_transparent and changed render-pass argument lists in render_scene_passes.rs
  - 2026-06-02 Workbench icon button native painter: rustfmt --edition 2021 --check over template_icon_buttons.rs, template_icon_buttons_tests.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench icon button native painter: Python tomllib parse of workbench_icon_button.zui, workbench_rail_button.zui, workbench_top_toolbar.zui, workbench_activity_rail.zui, workbench_scene_tree_panel.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench icon button native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new icon painter plus docs/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench icon button native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling and the existing zircon_runtime render_scene_passes.rs render-main-chain compile blocker remained unresolved
  - 2026-06-02 Workbench Inspector resource-row native painter: rustfmt --edition 2021 --check over template_inspector_rows.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench Inspector resource-row native painter: Python tomllib parse of workbench_inspector_panel.zui, workbench_component_property_row.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench Inspector resource-row native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new Inspector painter plus docs/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench Inspector resource-row native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Inspector Lighting disclosure tone sync: template_inspector_rows.rs now paints the empty Lighting disclosure-row label with the audited #9da8ae nested-resource tone while preserving the existing chevron, layout, and generic property-row fallback
  - 2026-06-02 Workbench Inspector Lighting disclosure tone sync: rustfmt --edition 2021 --check over template_inspector_rows.rs/template_section_titles.rs, Python tomllib parse of workbench_inspector_panel.zui/workbench_component_property_row.zui/editor_workbench_strict.zui, tracked editor-doc git diff --check, and trailing-whitespace scan over related Rust/ZUI/theme/docs/session files (passed; focused Cargo not attempted because cargo/rustc lanes were active)
  - 2026-06-02 Workbench Inspector nested Lighting row layout: template_inspector_rows.rs applies the accepted 14 px nested-select inset to Cast Shadows while preserving the field right edge; WorkbenchComponentPropertySlot03Row now declares layout_content_offset_x = 34.0 for the Receive Shadows checkbox, and the painter falls back to the old 28 px spacer for undeclared rows. Lighting disclosure geometry stays unchanged
  - 2026-06-02 Workbench Inspector Lighting checkbox native sync: rustfmt --edition 2021 --check over template_inspector_rows.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_inspector_panel.zui passed; tracked git diff --check and touched-file trailing whitespace scan passed; focused cargo test -p zircon_editor --lib template_inspector_rows was attempted twice but timed out and later exited without captured pass/fail output, so no Cargo pass is claimed
  - 2026-06-02 Workbench Inspector Materials-row tone sync: WorkbenchMaterialRow now declares the counted Materials label, #9aa5ab label/count tone, #8f9aa0 select value, #20272c select border, and #13181b native field fill; template_inspector_rows.rs consumes declared resource label/count colors plus field background/border before fallback constants
  - 2026-06-02 Workbench Inspector Materials-row tone sync: rustfmt --edition 2021 --check over template_inspector_rows.rs and workbench_projection.rs passed; Python tomllib parse/declaration assertions for workbench_inspector_panel.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused Cargo was not started because unrelated render-main-chain and editor bridge Cargo lanes were already active
  - 2026-06-02 Workbench Inspector axis value field native painter: rustfmt --edition 2021 --check over template_axis_value_fields.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench Inspector axis value field native painter: Python tomllib parse of workbench_axis_value_field.zui, workbench_inspector_panel.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench Inspector axis value field native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new axis field painter plus docs files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench Inspector axis value field native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Inspector axis label/link native painter: rustfmt --edition 2021 --check over template_axis_labels.rs, template_axis_value_fields.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench Inspector axis label/link native painter: Python tomllib parse of workbench_inspector_panel.zui, workbench_axis_value_field.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench Inspector axis label/link native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new axis label painter plus docs/ZUI files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench Inspector axis label/link native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Inspector Transform axis/title tone: template_axis_labels.rs now paints non-Scale Transform axes with #81888c and Scale axes with #7e8488, while template_section_titles.rs lowers only the Transform title icon to 0.38 alpha
  - 2026-06-02 Workbench Inspector Transform axis/title tone: rustfmt --edition 2021 --check over template_axis_labels.rs/template_section_titles.rs, tracked editor-doc git diff --check, and trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Inspector Transform Scale link spacing: workbench_inspector_panel.zui declares layout_offset_x = -12.0 on WorkbenchTransformScaleLink, template_axis_labels.rs applies it only to the chain glyph, and workbench_projection.rs asserts the projected offset
  - 2026-06-02 Workbench Inspector Transform Scale link spacing: rustfmt --edition 2021 --check over template_axis_labels.rs/workbench_projection.rs, Python tomllib parse over workbench_inspector_panel.zui/workbench_axis_value_field.zui/editor_workbench_strict.zui, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Inspector Transform Position text tone: workbench_inspector_panel.zui declares Position axis label_color #566871 and Position value_color #929ea4, template_axis_labels.rs/template_axis_value_fields.rs consume those declared colors, and workbench_projection.rs asserts the projected values
  - 2026-06-02 Workbench Inspector Transform Position text tone: refreshed prototype screenshots, focused component/pixel audits passed for the accepted Position row tone, node interaction/responsive validators passed, rustfmt/tomllib/git diff --check/trailing-whitespace checks passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Alert/Toast native painter: rustfmt --edition 2021 --check over template_alerts.rs, template_axis_value_fields.rs, and template_nodes.rs (passed)
  - 2026-06-02 Workbench Alert/Toast native painter: Python tomllib parse of workbench_component_drawer.zui, workbench_toast.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench Alert/Toast native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new alert painter plus docs/ZUI/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench Alert/Toast native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench slider native painter: rustfmt --edition 2021 --check over template_sliders.rs, template_alerts.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench slider native painter: Python tomllib parse of workbench_slider.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench slider native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new slider painter plus docs/ZUI/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench slider native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench button native painter: rustfmt --edition 2021 --check over template_buttons.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench button native painter: Python tomllib parse of workbench_button.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench button native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new button painter plus docs/ZUI/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench button native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Add Component button sync: template_buttons.rs now gives WorkbenchAddComponent the accepted 1.5 px native y offset, #bac4c9 text, #c5ced2 plus glyph, and non-duplicated Add Component label; rustfmt --edition 2021 --check, Python tomllib parse of workbench_inspector_panel.zui, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench button/icon-button drawer structure sync: workbench_component_drawer.zui now mirrors the HTML reference with an eight-control Buttons stack, a separate eight-button Icon Buttons column, and a grid/list/columns icon toggle; workbench_projection.rs asserts the new column widths, projected button/icon/dropdown fields, routes, and retained preview state updates
  - 2026-06-02 Workbench button/icon-button drawer structure sync: template_buttons.rs now consumes declared button layout offsets, template_icon_buttons.rs consumes declared icon-button offset and icon_size, rustfmt --edition 2021 --check passed over the touched Rust files, and Python tomllib parse passed for workbench_component_drawer.zui plus the related button/icon/segmented component assets; focused Cargo stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench bottom-row offset native sync: workbench_component_drawer.zui now carries the bottom Disabled/Dropdown and Input Dropdown/Stepper offsets on the actual leaf controls; template_dropdowns.rs and template_fields.rs consume those offsets in their paint rects, template_nodes.rs anchors Workbench dropdown popup rows to the shifted trigger, and workbench_projection.rs asserts the projected leaf offsets
  - 2026-06-02 Workbench leaf-offset ownership cleanup: workbench_component_drawer.zui also removes redundant offset props from WorkbenchIconButtonGridTop/Bottom and WorkbenchLabsTabs, leaving accepted paint offsets on the mini icon buttons and Labs tab leaves so future container-level layout support will not double-apply them
  - 2026-06-02 Workbench bottom-row offset native sync: rustfmt --edition 2021 --check over template_dropdowns.rs/template_fields.rs/template_nodes.rs/workbench_projection.rs, Python tomllib parse of workbench_component_drawer.zui/workbench_dropdown.zui/workbench_field.zui, leaf-offset ownership assertions for icon-grid/Labs row containers, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because cargo/rustc lanes were active
  - 2026-06-02 Workbench button-row native style sync: workbench_component_drawer.zui now declares Primary/Secondary font_size = 12.22, Disabled background/border/label/opacity, and Button Dropdown label/chevron/border tones; workbench_window_projection.rs and pane_component_projection/mod.rs project arrow_color into native icon color; template_buttons.rs consumes declared disabled style plus style opacity; template_dropdowns.rs consumes declared dropdown style colors
  - 2026-06-02 Workbench button-row native style sync: rustfmt --edition 2021 --check over template_buttons.rs/template_dropdowns.rs/workbench_window_projection.rs/pane_component_projection/mod.rs/workbench_projection.rs passed; Python tomllib parse and declaration assertions for the Workbench button/dropdown assets passed; stale-doc scan and tracked git diff --check passed with only LF-to-CRLF working-tree warnings; focused Cargo stayed deferred because cargo/rustc lanes were active
  - 2026-06-02 Workbench Icon/Delete native style sync: workbench_component_drawer.zui now declares WorkbenchButtonIcon foreground_color = "#7f8a91" plus corner_radius = 9.0 and WorkbenchButtonDelete foreground_color = "#d05a50" plus corner_radius = 9.0; template_buttons.rs consumes declared button radius before fallback and applies declared foreground color to text/glyph paint after visual brightness; workbench_projection.rs asserts both projected tones and radius values
  - 2026-06-02 Workbench Icon/Delete native style sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs passed; Python tomllib parse and declaration assertions for WorkbenchButtonIcon and WorkbenchButtonDelete passed; stale-doc scan, tracked git diff --check, and touched-file trailing-whitespace scan passed with only LF-to-CRLF working-tree warnings; focused Cargo stayed deferred because cargo/rustc lanes were active
  - 2026-06-02 Workbench Tertiary/Outline native style sync: workbench_component_drawer.zui now declares WorkbenchTertiaryButton and WorkbenchOutlineButton layout_offset_x = 1.0, foreground_color = "#879299", border_color = "#252e35", and corner_radius = 9.0, while Tertiary keeps background_color = "#171c20"; existing template_buttons.rs foreground/border/radius/offset paths consume those declarations, and workbench_projection.rs asserts the projected values
  - 2026-06-02 Workbench Tertiary/Outline native style sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs passed; Python tomllib parse and declaration assertions for WorkbenchTertiaryButton/WorkbenchOutlineButton passed; stale-doc scan, tracked git diff --check, and touched-file trailing-whitespace scan passed with only LF-to-CRLF working-tree warnings; focused Cargo stayed deferred because cargo/rustc lanes were active
  - 2026-06-02 Workbench Icon Buttons edge native sync: the eight 38 px component-drawer mini icon buttons now declare #171f26 borders and 10 px radius; template_icon_buttons.rs consumes declared radius and declared danger border values before fallback constants; workbench_projection.rs asserts the projected mini icon edge values
  - 2026-06-02 Workbench Icon Buttons edge native sync: rustfmt --edition 2021 --check over template_icon_buttons.rs, template_icon_buttons_tests.rs, and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; focused Cargo stayed deferred because an existing editor bridge Cargo/rustc lane was already compiling
  - 2026-06-02 Workbench field native painter: rustfmt --edition 2021 --check over template_fields.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench field native painter: Python tomllib parse of workbench_field.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench field native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new field painter plus docs/ZUI/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench field native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench component field focused/disabled tone sync: template_fields.rs now mirrors the audited #1b98a0 focused border plus #7d878d/#30383e/#24292d disabled text/border/surface values, and workbench_component_drawer.zui/editor_workbench_strict.zui carry the same component-field tokens and props
  - 2026-06-02 Workbench component field focused/disabled tone sync: rustfmt --edition 2021 --check over template_fields.rs, Python tomllib parse of workbench_field.zui/workbench_component_drawer.zui/editor_workbench_strict.zui, tracked editor-doc git diff --check, and trailing-whitespace scan over related Rust/ZUI/theme/docs/session files (passed; focused Cargo not attempted because cargo/rustc lanes were active)
  - 2026-06-02 Workbench input dropdown/stepper row vertical sync: workbench_component_drawer.zui now mirrors the audited bottom input row placement by carrying layout_offset_x = -4.0 and layout_offset_y = 8.0 on WorkbenchInputDropdown and WorkbenchInputStepper while preserving the 8 px row gap, dropdown width, and stepper width
  - 2026-06-02 Workbench input dropdown/stepper row vertical sync: rustfmt --edition 2021 --check over template_fields.rs, Python tomllib parse of workbench_field.zui/workbench_component_drawer.zui/editor_workbench_strict.zui, tracked editor-doc git diff --check, and trailing-whitespace scan over related Rust/ZUI/theme/docs/session files (passed; focused Cargo not attempted because cargo/rustc lanes were active)
  - 2026-06-02 Workbench dropdown native painter: rustfmt --edition 2021 --check over template_dropdowns.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench dropdown native painter: Python tomllib parse of workbench_dropdown.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench dropdown native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new dropdown painter plus docs/ZUI/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench dropdown native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench chip/section title native painter: rustfmt --edition 2021 --check over template_chips.rs, template_section_titles.rs, template_nodes.rs, and painter mod.rs (passed)
  - 2026-06-02 Workbench chip/section title native painter: Python tomllib parse of workbench_chip.zui, workbench_section_title.zui, workbench_viewport_panel.zui, workbench_inspector_panel.zui, workbench_component_drawer.zui, and editor_workbench_strict.zui (passed)
  - 2026-06-02 Workbench chip/section title native painter: git diff --check over tracked touched Rust/docs/session files and trailing-whitespace scan over the new chip/title painters plus docs/ZUI/session files (passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - 2026-06-02 Workbench chip/section title native painter: focused Cargo validation deferred while active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench shell panel native painter: added template_shell_panels.rs for root/topbar/main band/activity rail/scene tree/viewport/inspector/component drawer/status/tabs/inspector-section container backgrounds and separators
  - 2026-06-02 Workbench viewport scene native painter: added template_viewport_scene.rs for declared WorkbenchViewport scene layers: floor grate slots, cargo/rack striping, handrail posts, selection glow, transform axes, axis origins, caps, and orientation-gizmo facets
  - 2026-06-02 Workbench viewport scene native painter: rustfmt --edition 2021 --check over template_viewport_scene.rs, template_nodes.rs, painter mod.rs, and workbench_projection.rs; Python tomllib parse of workbench_viewport_panel.zui and editor_workbench_strict.zui; viewport scene source/ownership assertions; tracked git diff --check; and touched-file trailing-whitespace scan passed; focused Cargo validation deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport scene interaction guard: surface_hit_test/template_node.rs now asserts decorative viewport scene nodes stay out of the host-contract hit surface, and workbench_projection.rs asserts pointer down/up on WorkbenchViewportFloorGrateRight yields no feedback, pressed target, or runtime event
  - 2026-06-02 Workbench viewport scene interaction guard: rustfmt --edition 2021 --check over touched viewport/pointer/painter test files, Python tomllib parse of workbench_viewport_panel.zui and editor_workbench_strict.zui, source-marker assertions, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo validation deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport light/shadow native painter: template_viewport_scene.rs now paints WorkbenchViewportLightwash, Shadow, FloorReflection, WallLight, and Beacon layers as layered native quads with brighter light cores, darker shadow centers, reflection streaks, hot wall-light cores, and amber beacon cores
  - 2026-06-02 Workbench viewport light/shadow native painter: rustfmt --edition 2021 --check over template_viewport_scene.rs and workbench_projection.rs, Python tomllib parse of workbench_viewport_panel.zui/editor_workbench_strict.zui, and source-marker assertions for the new light/shadow painter and projection coverage passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport scene modularization: split template_viewport_scene.rs into a 322-line classifier/dispatcher, template_viewport_scene_structure.rs for 469 lines of structural scene primitives, and template_viewport_scene_light.rs for 293 lines of light/overlay primitives
  - 2026-06-02 Workbench viewport scene modularization: rustfmt --edition 2021 over painter mod.rs plus the three viewport painter modules passed; focused Cargo validation stayed deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport architectural primitive pass: template_viewport_scene_structure.rs now paints native grid glow bands, floor panel insets, floor seam glow, side-panel guide lines, side stair steps, center wall detail lines, rear door inset seams, door-core seams, and wall-column edge strips for the declared viewport architecture nodes
  - 2026-06-02 Workbench viewport architectural primitive pass: workbench_projection.rs now locks representative grid, floor panel, floor seam, stairs, wall-detail, back-door, door-core, and wall-column dimensions; rustfmt over touched viewport painter/projection files passed, with focused Cargo still deferred while active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport structural sub-split: moved Grid/FloorPanel/FloorSeam drawing into template_viewport_scene_floor.rs and SidePanel/SideStairs/WallDetail/BackDoor/DoorCore/WallColumn drawing into template_viewport_scene_architecture.rs, leaving template_viewport_scene_structure.rs focused on base surfaces, grate slots, cargo/rack, handrail, axes, selection, and gizmo primitives
  - 2026-06-02 Workbench viewport structural sub-split: rustfmt --edition 2021 --check over painter mod.rs, the five viewport painter files, and workbench_projection.rs; Python tomllib parse of workbench_viewport_panel.zui/editor_workbench_strict.zui; source-marker assertions; tracked git diff --check; and touched-file trailing-whitespace scan passed. At that checkpoint viewport painter file sizes were dispatcher 450 lines, structure 469, floor 204, architecture 332, and light 293; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport cargo-inner primitive pass: CargoInner viewport ids now classify before Cargo ids and draw native transparent internal frame lines instead of inheriting cargo-body stripe/shadow detail; workbench_projection.rs locks WorkbenchViewportCargoRightInner dimensions
  - 2026-06-02 Workbench viewport cargo-inner primitive pass: rustfmt --edition 2021 --check over touched viewport painter/projection files, Python tomllib parse of workbench_viewport_panel.zui/editor_workbench_strict.zui, source-marker assertions, tracked git diff --check, and touched-file trailing-whitespace scan passed. At that checkpoint viewport painter file sizes were dispatcher 475 lines, structure 580, floor 204, architecture 332, and light 293; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport selected-prop primitive pass: WorkbenchViewportPropBody and WorkbenchViewportPropTop now route to dedicated selected-prop body/top painters instead of inheriting Cargo stripe detail; workbench_projection.rs locks their declared 112 x 74 and 112 x 22 frames
  - 2026-06-02 Workbench viewport selected-prop primitive pass: rustfmt --edition 2021 --check over the touched viewport painter/projection files, Python tomllib parse of workbench_viewport_panel.zui/editor_workbench_strict.zui, source-marker assertions, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport axis declaration color sync: template_viewport_scene_structure.rs now lets native axis lines prefer authored background_color before X/Y/Z fallback constants, the painter test locks a custom declared AxisX color, and workbench_projection.rs locks the declared AxisY/AxisZ dimensions alongside AxisX
  - 2026-06-02 Workbench viewport axis declaration color sync: rustfmt --edition 2021 --check over the touched viewport painter/projection files, Python tomllib parse of workbench_viewport_panel.zui/editor_workbench_strict.zui, and source-marker assertions passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport base-surface primitive pass: template_viewport_scene_surfaces.rs now owns Backdrop, Ceiling, BackWall, and Floor native surface detail with side/top shadows, ceiling ribs, wall panel lines, and floor depth bands; workbench_projection.rs locks the stretched backdrop/ceiling/back-wall/floor dimensions
  - 2026-06-02 Workbench viewport base-surface primitive pass: rustfmt --edition 2021 --check over touched viewport painter/projection files, Python tomllib parse of workbench_viewport_panel.zui/editor_workbench_strict.zui, source-marker assertions, tracked git diff --check, and touched-file trailing-whitespace scan passed. Current viewport painter file sizes are dispatcher 553 lines, structure 580, surfaces 300, floor 204, architecture 332, and light 293; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench viewport gizmo label color sync: workbench_window_projection.rs and the generic pane component projection now map authored foreground_color into TemplatePaneNodeData.value_color, and workbench_projection.rs asserts WorkbenchViewportGizmoX/Y/Z project the declared red/green axis-label colors from workbench_viewport_panel.zui
  - 2026-06-02 Workbench viewport gizmo label color sync: rustfmt --edition 2021 --check over the touched projection/test files, Python tomllib parse of workbench_viewport_panel.zui/editor_workbench_strict.zui, source-marker assertions, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench mini icon-button surface declaration sync: workbench_component_drawer.zui now declares the non-danger mini icon-button background/border, template_icon_buttons.rs consumes resolved style colors in the normal panel branch, template_icon_buttons_tests.rs locks surface/border plus hover priority, and workbench_projection.rs asserts projected top/bottom mini icon-button style values
  - 2026-06-02 Workbench mini icon-button surface declaration sync: rustfmt --edition 2021 --check over the touched icon-button painter/projection files, Python tomllib parse of workbench_component_drawer.zui/workbench_icon_button.zui, marker checks, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench slider track declaration sync: WorkbenchInputSlider now declares background_color = "#2a3338", template_sliders.rs consumes the resolved style color for enabled base-track painting before the fixed fallback, its focused test locks declared-track plus disabled-track priority, and workbench_projection.rs asserts the projected slider style background
  - 2026-06-02 Workbench slider track declaration sync: rustfmt --edition 2021 --check over template_sliders.rs and workbench_projection.rs, Python tomllib parse of workbench_component_drawer.zui/workbench_slider.zui/editor_workbench_strict.zui, marker checks, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench slider row/proportion native sync: workbench_component_drawer.zui now exposes three explicit Value/Range/Steps slider rows with declared label/value text, track_offset_x = -10.0, track_width_delta = 18.0, Range range_min = 20.0, and Steps step_tick_count = 5.0; both retained projection paths map those aliases, template_sliders.rs paints the left label, declared value chip text, adjusted track rectangle, Range dual-thumb span, and Steps ticks, and workbench_projection.rs asserts the three slider declarations
  - 2026-06-02 Workbench slider row/proportion native sync: rustfmt --edition 2021 --check over template_sliders.rs, workbench_window_projection.rs, pane_component_projection/mod.rs, and workbench_projection.rs passed; Python tomllib parse plus slider declaration assertions passed for workbench_component_drawer.zui/workbench_slider.zui/editor_workbench_strict.zui; stale-doc marker scan, tracked git diff --check, and touched-file trailing-whitespace scan passed; focused cargo test -p zircon_editor --lib template_sliders --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 timed out after 304 seconds without compiler diagnostics, and no cargo/rustc processes remained afterward
  - 2026-06-02 Workbench Sliders structural column sync: workbench_component_drawer.zui now declares a dedicated 260 px WorkbenchComponentSliders column with WorkbenchSlidersTitle, moves WorkbenchInputSlider/Range/Steps out of Inputs, sets Inputs to 214 px, Checkboxes/Radios to 168 px, and Labs to 236 px, preserving slider control ids and preview state paths
  - 2026-06-02 Workbench Sliders structural column sync: rustfmt --edition 2021 --check over workbench_projection.rs and Python tomllib parse of workbench_component_drawer.zui passed; tracked git diff --check over tracked touched files and touched-file trailing-whitespace scan passed; focused Cargo was skipped because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench component drawer top/lower row split: WorkbenchComponentDrawerBody now owns WorkbenchComponentTopRow for the component grid and WorkbenchComponentLowerRow for lower table/feedback demos; WorkbenchComponentList keeps only List/Menu, and WorkbenchComponentTable owns the lower Table title plus table_group
  - 2026-06-02 Workbench component drawer top/lower row split: rustfmt --edition 2021 --check over workbench_projection.rs and Python tomllib parse of workbench_component_drawer.zui passed; tracked git diff --check over tracked touched files and touched-file trailing-whitespace scan passed; focused Cargo was skipped because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench component drawer feedback/toast layout sync: WorkbenchComponentFeedback now exposes WorkbenchFeedbackAlerts, WorkbenchTooltipRoot, and WorkbenchFeedbackToastColumn; the standalone feedback_toast omits an instance control_id so the mounted notification sample expands as the single WorkbenchToastRoot while inline alert ids stay distinct
  - 2026-06-02 Workbench component drawer feedback/toast layout sync: rustfmt --edition 2021 --check over workbench_projection.rs passed; Python tomllib parse and feedback/toast ownership assertions for workbench_component_drawer.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused Cargo was skipped because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench composite icon-button surface declaration sync: WorkbenchButtonIcon now declares background_color = "#20262a" and border_color = "#303840", template_buttons.rs consumes both through the existing resolved button-style path, and workbench_projection.rs asserts the projected style values while WorkbenchButtonDelete remains on the danger branch
  - 2026-06-02 Workbench composite icon-button surface declaration sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs, Python tomllib parse of workbench_component_drawer.zui/workbench_button.zui/editor_workbench_strict.zui, marker checks, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Tertiary/Outline declaration assertion sync: WorkbenchTertiaryButton declares background_color = "#171c20" and border_color = "#2b343b", WorkbenchOutlineButton declares border_color = "#2b343b", workbench_projection.rs asserts those projected style values, and template_buttons.rs now locks the Tertiary surface/border plus Outline border-only painter paths
  - 2026-06-02 Workbench Tertiary/Outline declaration assertion sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs, Python tomllib parse of workbench_component_drawer.zui/workbench_button.zui/editor_workbench_strict.zui, marker checks, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Primary/Secondary declaration assertion sync: WorkbenchPrimaryButton and WorkbenchSecondaryButton have projection assertions for first-row offsets and font_size = 12.22; WorkbenchPrimaryButton now asserts visual_brightness = 1.0 plus projected background_color = "#29a4b8" and border_color = "#1c8798", while WorkbenchSecondaryButton keeps visual_brightness = 1.01 plus projected background_color = "#1a1f23"; template_buttons.rs locks first-row offset, declared Primary/Secondary surface, and declared border behavior
  - 2026-06-02 Workbench Primary/Secondary declaration assertion sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs, Python tomllib parse of workbench_component_drawer.zui/workbench_button.zui/editor_workbench_strict.zui, marker checks, tracked git diff --check, and trailing-whitespace scan passed; focused Cargo validation stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench Primary button color native sync: WorkbenchPrimaryButton declares background_color = "#29a4b8", border_color = "#1c8798", and visual_brightness = 1.0; native keeps a single declared surface color while the browser prototype keeps the accepted #1c8395 bottom gradient fill until the native surface contract grows gradient support
  - 2026-06-03 Workbench Primary button color native sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; focused Cargo stayed deferred because other Cargo/rustc lanes were active
  - 2026-06-03 Workbench Add Component button border native sync: WorkbenchAddComponent declares border_color = "#364047"; template_buttons.rs consumes the declared border before the Inspector-specific text/glyph override, and workbench_projection.rs asserts the projected border while preserving layout, fill, label, icon tones, and route ownership
  - 2026-06-03 Workbench Add Component button border native sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_inspector_panel.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench icon-button style selector: style_selector/workbench_icon_button.rs now resolves toolbar, rail, and panel icon-button chrome from UiPainterState / UiPainterResolvedState instead of direct node-state branching in template_icon_buttons.rs; template_icon_buttons.rs keeps only glyph geometry and command emission
  - 2026-06-03 Workbench icon-button style selector: rustfmt --edition 2021 over style_selector/mod.rs, style_selector/workbench_icon_button.rs, template_icon_buttons.rs, and template_icon_buttons_tests.rs passed; cargo test -p zircon_editor icon_button_style_selector_uses_shared_state_priority --locked --message-format short passed (1 test, 0 failed), then cargo test -p zircon_editor template_icon_buttons --locked --message-format short passed (11 tests, 0 failed; existing zircon_runtime and zircon_editor warnings remained)
  - 2026-06-03 Workbench slider style selector: style_selector/workbench_slider.rs now resolves slider track, fill, thumb, halo, value chip, label, disabled, focus, pressed, dragging, and drop-hover tones from UiPainterState / UiPainterResolvedState instead of keeping those visual decisions in template_sliders.rs; template_sliders.rs keeps slider recognition, geometry, range span, ticks, and paint-command emission
  - 2026-06-03 Workbench slider style selector: validation pending in the current slice; expected focused checks are rustfmt over style_selector/mod.rs, style_selector/workbench_slider.rs, and template_sliders.rs, followed by focused zircon_editor slider painter tests
  - 2026-06-03 Workbench dropdown style selector: style_selector/workbench_dropdown.rs now resolves dropdown surface, border, text, chevron, disabled, open, focus, pressed, hover, dragging, drop-hover, declared style colors, placeholder text, and visual brightness from UiPainterState / UiPainterResolvedState; template_dropdowns.rs keeps Workbench dropdown recognition, paint rect alignment, label layout, chevron geometry, and paint-command emission
  - 2026-06-03 Workbench dropdown style selector: rustfmt --edition 2021 --check over style_selector/mod.rs, style_selector/workbench_dropdown.rs, and template_dropdowns.rs passed; focused cargo test -p zircon_editor --lib template_dropdowns --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-dropdown-selector --message-format short --color never -- --nocapture --test-threads=1 timed out after 604 seconds without Rust diagnostics, and no cargo/rustc process remained afterward
  - 2026-06-03 Workbench popup-row style selector: style_selector/workbench_popup_row.rs now resolves popup option/menu row surface, selection mark, text, shortcut, adornment, disabled, focused, pressed, selected, checked, hover, and danger tones from UiPainterState / UiPainterResolvedState; template_popup_rows.rs keeps popup bounds, row geometry, menu flag parsing, label layout, adornment geometry, and paint-command emission
  - 2026-06-03 Workbench popup-row style selector: rustfmt --edition 2021 --check over style_selector/mod.rs, style_selector/workbench_popup_row.rs, and template_popup_rows.rs passed; git diff --check over the touched selector, popup painter, docs, and session note passed with the existing docs LF-to-CRLF warning; focused cargo test -p zircon_editor --lib template_popup_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-popup-row-selector --message-format short --color never -- --nocapture --test-threads=1 timed out twice during compilation (about 608s and 909s) without Rust diagnostics or a produced zircon_editor test binary, and the remaining popup-row Cargo/rustc lane was stopped after the second timeout
  - 2026-06-03 Workbench segmented/tab style selector: style_selector/workbench_segmented_control.rs resolves segmented-control and tab surface, border, selected segment, underline, selected text, idle text, disabled, pressed, focused, hovered, dragging, drop-hover, and declared idle tab background from UiPainterState / UiPainterResolvedState; template_segmented_controls.rs keeps segment splitting, layout offsets, label placement, underline geometry, and paint-command emission
  - 2026-06-03 Workbench segmented/tab style selector: expected focused checks are rustfmt over style_selector/mod.rs, style_selector/workbench_segmented_control.rs, and template_segmented_controls.rs, node verify-native-component-contract.mjs, and cargo test -p zircon_editor --lib template_segmented_controls --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Editor Workbench Shell

## Purpose

This document describes the current Rust-owned retained editor workbench shell. The active editor shell is owned by `zircon_editor::ui::retained_host`, consumes `.zui` host assets, and projects editor state into Rust host-contract DTOs. It is not a compatibility layer for deleted generated UI sources.

## Ownership

- `zircon_editor::ui::host` owns `EditorManager`, module wiring, layout/view registration, startup state, asset manager access, and editor-side service boundaries.
- `zircon_editor::ui::workbench` owns `EditorState`, `WorkbenchLayout`, workbench snapshots, model projection, menu models, and editor pane semantics.
- `zircon_editor::ui::retained_host` owns native window glue, retained input bridges, presenter state, host-contract DTOs, template projection application, and native painting.
- `zircon_runtime::ui` and `zircon_runtime_interface::ui` own the shared UI surface, tree, layout, dispatch, component, and binding contracts that the editor host consumes.

The boundary is intentionally split this way so the retained host can draw and dispatch editor UI without taking ownership of workbench business state or runtime UI contracts.

## Host Runtime Flow

`run_editor(...)` creates a `UiHostWindow`, constructs `RetainedEditorHost`, wires retained callbacks, refreshes the initial state, and enters the host window run loop. `RetainedEditorHost` holds the editor runtime, manager, asset/resource channels, workbench chrome metrics, shared pointer bridges, template bridges, viewport controller, native window presenter store, and dirty flags.

The recompute path builds a `WorkbenchViewModel`, computes workbench geometry, refreshes builtin host template bridges, resolves floating-window projection bundles, and then calls `apply_presentation(...)`. Presentation application converts editor-owned workbench data into `HostWindowPresentationData` plus `PaneSurfaceHostContext` state. Existing live host interaction state, menu state, close prompts, text focus, and viewport images are preserved across full presentation replacement.

## Template Authority

The current shell structure comes from source-controlled `.zui` assets, not from generated UI files. Important host assets include:

- `zircon_editor/assets/ui/editor/components/workbench/shell/activity_drawer_window.zui`
- `zircon_editor/assets/ui/editor/host/animation_graph_body.zui`
- `zircon_editor/assets/ui/editor/host/animation_sequence_body.zui`
- `zircon_editor/assets/ui/editor/host/asset_surface_controls.zui`
- `zircon_editor/assets/ui/editor/host/build_export_desktop_body.zui`
- `zircon_editor/assets/ui/editor/host/console_body.zui`
- `zircon_editor/assets/ui/editor/host/editor_main_frame.zui`
- `zircon_editor/assets/ui/editor/host/floating_window_source.zui`
- `zircon_editor/assets/ui/editor/host/generated_bottom_body.zui`
- `zircon_editor/assets/ui/editor/host/hierarchy_body.zui`
- `zircon_editor/assets/ui/editor/host/inspector_body.zui`
- `zircon_editor/assets/ui/editor/host/inspector_surface_controls.zui`
- `zircon_editor/assets/ui/editor/host/module_plugins_body.zui`
- `zircon_editor/assets/ui/editor/host/pane_surface_controls.zui`
- `zircon_editor/assets/ui/editor/host/performance_timeline_body.zui`
- `zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui`
- `zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui`
- `zircon_editor/assets/ui/editor/host/startup_welcome_controls.zui`
- `zircon_editor/assets/ui/editor/host/workbench_shell.zui`

`EditorUiHostRuntime` loads these assets, builds shared UI surfaces, registers stable bindings, and exposes retained host projections. `RetainedUiHostAdapter` maps generic host models into retained host node models with component kind, frame, clip, z, style, state, validation, popup, selection, drag/drop, and route metadata.

Current root-shell frame authority is the host `.zui` geometry, not old workbench metrics. `workbench_shell.zui` gives the menu bar `24px`, the page strip `32px`, the separator `1px`, the status bar `24px`, and the activity rail `44px`. That makes the body and document top boundary `57px`; at 1280x720 the document host frame is `44,57,1236,639`, and at 960x540 it is `44,57,916,459`.

The 2026-05-15 Material visual slice keeps that geometry authority unchanged while strengthening first-look editor controls. `workbench_shell.zui` marks representative menu and activity-rail icon buttons with inset surfaces, 1px borders, and 10px rounded corners. `inspector_surface_controls.zui` applies the same rounded inset treatment to fields and uses pill-rounded primary/danger actions. `startup_welcome_controls.zui` does the same for welcome fields and actions. These values are visual metadata consumed by the retained painter and command stream; action routes, binding ids, drawer ownership, and workbench business state stay unchanged.

The 2026-05-16 feedback pass keeps the same shell frames and route ownership but strengthens state contrast through shared Material tokens and the retained painter. Hovered controls now receive a brighter teal-tinted container plus accent/focus border, pressed controls move to a deeper teal, selected and checked controls use the saturated selected container, and focus/pressed/selected borders use the 2px focus-ring width. Workbench chrome, pane controls, welcome controls, Inspector controls, and component showcase controls inherit these states through the common theme/painter path instead of separate host-specific styling.

The 2026-06-02 Workbench Inspector split-row pass replaces the temporary dropdown-backed component property rows with `WorkbenchComponentPropertyRow`. The retained bridge now keeps property label text and editable value text as separate template attributes, including generated overflow rows from the Inspector property repeat pool. Native painting recognizes these rows as split property rows and draws the value side as an inset editable field while preserving the existing Change/Submit binding ids.

Inspector resource and nested property rows now have a focused native painter lane on top of the split-row data path. `template_inspector_rows.rs` recognizes only known Inspector row shapes before the generic property fallback: Mesh/Material resource selectors receive a compact resource icon or swatch plus right chevron, with `WorkbenchMaterialRow` declaring the counted `Materials` label, `#9aa5ab` label/count tone, `#8f9aa0` select value, `#20272c` select border, and `#13181b` native field fill. `Cast Shadows` becomes an on/off select field with the accepted 14 px nested-select inset, stable right edge, dynamic `#282e32` field fill, `#343d43` border, and `#b5c0c5` value tone; those style props are injected only when the live field id is `cast_shadows` and cleared for non-Cast rows. `Receive Shadows` becomes a checkbox row that consumes the declared `WorkbenchComponentPropertySlot03Row` 34 px content offset before the old 28 px fallback, and an empty `Lighting` row becomes a disclosure header with the audited `#9da8ae` nested-resource label tone. Other live plugin property rows deliberately continue through `template_property_rows.rs`, so arbitrary component fields are not misclassified as resource controls.

Inspector Transform axis fields now have the same kind of bottom-up primitive coverage. `workbench_inspector_panel.zui` declares the Position, Rotation, and Scale X/Y/Z values as `WorkbenchAxisValueField` children with independent Change/Submit routes, and `template_axis_value_fields.rs` consumes those child `InputField` nodes before the generic Material text-field fallback. The painter draws compact dark inset value fields with focused, hovered, pressed, disabled, and validation-state borders while leaving the parent Transform rows responsible for layout. `template_axis_labels.rs` consumes the X/Y/Z label children and the declared `WorkbenchTransformScaleLink` child, drawing declared Position axis text as `#566871`, fallback non-Scale axis text as `#81888c`, Scale axis text as `#7e8488`, and the small Scale chain glyph as native primitives instead of relying on generic label text or pseudo-overlays. Position value fields also declare `value_color = "#929ea4"`, which `template_axis_value_fields.rs` applies before the normal text fallback while preserving disabled/error overrides. `WorkbenchTransformScaleLink` carries the accepted `layout_offset_x = -12.0`, which the native painter applies only to the link glyph so the real Scale X label and value boxes keep their declared row geometry. The Transform title icon is also softened to the accepted 0.38 alpha in `template_section_titles.rs` without changing row layout.

The same Workbench component pass also promotes table rows from whitespace-aligned labels into declared table cells. `WorkbenchTableRow` now projects as a table component, component-drawer rows provide their cell values through `options`, and the native template painter draws each cell into stable column rectangles. `template_table_rows.rs` now owns the full Workbench table-row primitive for `WorkbenchTable*` rows: row surfaces, selected/header/tail tones, the audited `#aab5ba` header text, the header-only `layout_content_offset_x = -1.0` / `layout_content_offset_y = 3.0` text-and-gear alignment, subtle bottom separators, column text, and right-side gear or overflow glyphs are painted before the generic Material fallback. `WorkbenchTableTail` is now an explicit measured branch rather than a future hint: the retained projection carries its row/content/per-cell offsets plus declared `#aab5ba` fourth-cell Modified text color through `TemplatePaneNodeData`, and the painter applies those values only to tail cells while preserving the tail deep surface and fallback fourth-cell tone. Existing row selection and click bindings remain unchanged, but table view pixels are no longer dependent on font-specific spacing hacks or generic selected-surface behavior.

Workbench text buttons now have a dedicated native painter path. `template_buttons.rs` recognizes `WorkbenchButton`/`Button` nodes before the generic Material surface fallback, draws compact filled, outlined, tertiary, danger, disabled, hover, focus, and pressed states, centers text, and supplies small plus/trash/dropdown glyphs for composed button variants. `WorkbenchAddComponent` keeps the same click route but now uses the accepted Inspector-specific 1.5 px paint offset, `#bac4c9` text, `#c5ced2` plus glyph, and declared `#364047` border; `workbench_inspector_panel.zui` and the reference builder use `Add Component` text so the native glyph supplies the icon. The retained bridge still owns click routes and state mutation; this slice only makes low-level button pixels deterministic.

The component drawer button lane now mirrors the HTML reference as an explicit eight-control two-column stack: Primary/Secondary, Tertiary/Outline, Icon/Delete, and Disabled/Dropdown. `workbench_component_drawer.zui` carries the measured 204 px column width, per-row gaps, and the small per-control paint offsets on the actual button/dropdown controls; `visual_brightness` now projects into `TemplatePaneNodeData.label_brightness`, and `template_buttons.rs` applies that brightness only to Workbench button surface/border/text/glyph colors so the Primary/Secondary and Icon/Delete micro-retunes are not stranded in `.zui`. The first row declares the audited 3 px / 1 px x offsets, -1 px y offset, 12.22 px font size, 1.01 brightness, and Secondary `#1a1f23` surface; projection and painter tests lock those values. The second row declares the audited 1 px x offset, Tertiary `#171c20` surface, `#879299` content tone, `#252e35` Tertiary/Outline border, and 9 px radius. The third row declares the audited `WorkbenchButtonIcon` `#20262a` surface, `#303840` border, `#7f8a91` content tone, and 9 px radius; `WorkbenchButtonDelete` keeps the danger branch while declaring `#d05a50` content tone and the same 9 px radius. The bottom row now declares Disabled as `#2d3337` background, `#343d44` border, `#747f86` label, and `0.72` opacity, and declares the Button Dropdown label, chevron, and border as `#7f8a91`, `#67737a`, and `#1f272e`. `template_buttons.rs` consumes declared offsets, disabled style, style opacity, foreground colors, and radius before drawing native button surfaces/content; `template_dropdowns.rs` consumes declared offsets plus background, border, value, and chevron colors, and the projection layer maps `arrow_color` into native icon color for dropdown chevrons.

Workbench input fields now have a matching native painter path. `template_fields.rs` consumes component-drawer `WorkbenchField` nodes such as `WorkbenchInputText`, `WorkbenchInputFocused`, `WorkbenchInputDisabled`, `WorkbenchInputStepper`, and the reusable `WorkbenchFieldRoot`, drawing neutral dark field surfaces, focused/hovered/pressed/disabled borders, placeholder tone, and the stepper divider/arrows. The component-field lane now mirrors the audited focused border `#1b98a0` and disabled text/border/solid-surface values `#7d878d` / `#30383e` / `#24292d`; `focus_border_color` is aliased into the shared `border_color` style path during retained projection and the field painter consumes that declared border before its fallback constant. `disabled_opacity` is likewise aliased into the shared `opacity` style path, and the field painter multiplies it into the native field surface, text, and stepper commands. The browser prototype keeps the extra disabled gradient and cyan shadow as CSS-only effects. `workbench_component_drawer.zui` also mirrors the latest bottom input dropdown/stepper row by putting `layout_offset_x = -4.0`, `layout_offset_y = 8.0`, and fixed `30.5` heights on the actual dropdown and stepper controls while keeping the row `layout_gap = 8.0`; `template_fields.rs` preserves half-pixel Workbench field heights when aligning the stepper paint rect. Transform axis value fields stay on `template_axis_value_fields.rs`, so Inspector vector editing remains isolated from general input-field styling.

Workbench dropdown triggers now have a matching native painter path. `template_dropdowns.rs` consumes `WorkbenchInputDropdown`, `WorkbenchDropdownRoot`, and Workbench dropdown/combo-box roles before the generic Material fallback, drawing the compact field surface, focused/open/hovered/disabled borders, selected text, right chevron, declared paint offsets, and the component-lab bottom-row `30.5` half-pixel height without rounding it back to an integer. Open Workbench dropdown nodes pass that shifted trigger rect into `template_popup_rows.rs`, so structured option rows keep their selected checks and hover state while staying visually anchored to the moved trigger.

Workbench popup/dropdown state is now isolated from the main componentized window bridge. `popup_state.rs` owns `toggle_popup`, dropdown option selection, popup menu item selection, and transient menu flag cleanup, while `componentized_window.rs` keeps routing high-level component-lab actions to those state methods. `frame_geometry.rs` owns the generic host-contract FrameRect helpers and `template_geometry.rs` owns template-node popup bounds, so keyboard navigation, outside-click dismiss, popup-row hit testing, native pointer damage, and redraw region merging share one focused geometry base instead of a Workbench-specific module name. This preserves the existing preview behavior but keeps popup state transitions and popup geometry in focused modules before the next runtime-level primitive pass. Module workspace dropdowns now share this path: an `EditWorkbench...` module field action opens a source `WorkbenchDropdown` when the control has authored `options`, and `popup_state.rs` handles option validation, `value` / `value_text` writeback, popup closure, and retained projection refresh. The Material Domain regression is `workbench_module_dropdowns_open_select_and_close_with_shared_dropdown_path`; the selected-state comparison is `target/editor-workbench-visual-check/editor-workbench-ai-material-vs-native-module-dropdown-selected-1672x941.png`.

Workbench chips and section titles now have dedicated retained-host painter routes. `template_chips.rs` consumes the viewport toolbar chips and reusable `WorkbenchChipRoot`, drawing raised chip chrome, hover/focus/open state, text, and dropdown chevrons while deliberately leaving `WorkbenchStatus*` chips on `template_status_controls.rs`. `template_section_titles.rs` consumes component-drawer titles plus Inspector `Props` / `Transform` / `Mesh Renderer` titles, draws consistent Workbench title text, supplies small leading glyphs for the Inspector title variants, and mirrors the audited `#b0babf` Mesh Renderer title tone through `WorkbenchMeshLabel` plus the `.workbench-mesh-title` strict-theme selector.

Workbench shell containers now have their own retained-host painter route. `template_shell_panels.rs` consumes only explicit Workbench container ids such as `WorkbenchWindowRoot`, `WorkbenchWindowTopToolbar`, `WorkbenchMainBand`, `WorkbenchWindowActivityRail`, `WorkbenchSceneTreePanel`, `WorkbenchViewportPanel`, `WorkbenchInspectorPanel`, `WorkbenchComponentDrawer`, `WorkbenchWindowStatusBar`, tab bands, Inspector property sections, and component-drawer columns. It draws the deep shell surfaces plus top/side/bottom separator lines before leaf widgets are painted; buttons, fields, chips, rows, status controls, and business interaction state stay on their existing specialized routes.

Workbench viewport scene layers now have their own retained-host painter route. `template_viewport_scene.rs` consumes only internal `WorkbenchViewport*` scene nodes after toolbar chips are handled and before generic text/section fallbacks, leaving viewport toolbar chips, axis labels, and gizmo text on the existing chip/text paths. The native painter adds backdrop/ceiling/back-wall/floor surface depth, floor-grate slot repetition, cargo body/rack inset detail, cargo inner frame lines, selected-prop body/top facets, handrail posts, selection glow, transform-axis glow/caps/origin, orientation-gizmo rod/facet primitives, architectural grid/floor/wall/door primitives, and layered light/shadow/reflection primitives for Lightwash, Shadow, FloorReflection, WallLight, and Beacon nodes while `workbench_viewport_panel.zui` remains the layout and color authority. It does not introduce viewport business state or input routing; it only replaces generic pane fallback pixels for the declared scene layers. Host-contract hit testing and componentized pointer dispatch both keep these decorative scene layers non-dispatchable so they cannot steal clicks from toolbar controls or future viewport body interactions. The viewport painter is split by responsibility: `template_viewport_scene.rs` owns classification and order dispatch, `template_viewport_scene_surfaces.rs` owns Backdrop/Ceiling/BackWall/Floor large surface depth, `template_viewport_scene_structure.rs` owns floor-grate slots, cargo bodies, cargo inner frames, selected-prop body/top facets, rack, handrail, selection, axes, and gizmo primitives, `template_viewport_scene_floor.rs` owns grid, floor-panel, and floor-seam detail, `template_viewport_scene_architecture.rs` owns side panel, stair, wall-detail, rear-door, door-core, and wall-column primitives, and `template_viewport_scene_light.rs` owns lightwash, shadow, reflection, light-strip, and beacon overlays.

`WorkbenchViewportSurface` and `WorkbenchViewportGizmoPanel` are classified as viewport layout containers, not scene paint primitives. They keep their arranged clip and child coordinate space, but return without paint commands so the backdrop, floor, light, axes, cargo, and gizmo children remain the only source of viewport scene pixels.

Viewport gizmo X/Y/Z labels remain text-like declarative nodes, but their authored `foreground_color` is now projected into the retained host contract as `value_color` in both the Workbench window projection and generic pane component projection. That keeps the native contract aligned with `workbench_viewport_panel.zui` even when a later specialized painter consumes value colors instead of the generic text foreground path.

Viewport axis lines now treat `workbench_viewport_panel.zui` as the color authority as well as the layout authority. The native axis painter still has X/Y/Z fallback constants, but it first consumes the authored `background_color` from the retained node style so local color tuning in the `.zui` asset is reflected without changing Rust constants.

The Workbench component-drawer selection controls now have a dedicated native painter path. `template_selection_controls.rs` recognizes checkbox, radio, and toggle nodes before the generic surface pass, draws the control mark/track/thumb plus label directly, and avoids treating `checked`/`selected` selection primitives as full-row highlighted list items. Checkbox/radio marks use the measured 16 px mark, 9 px mark-label gap, muted `#828c93` label tone, `#13191d` / `#374148` unchecked mark surface, current strict-theme checked checkbox/radio shell tokens, and the current 5 px radio dot default from `template_selection_control_geometry/metrics.rs`. The reusable checkbox/radio components and the concrete `WorkbenchCheckboxOn/Off` plus `WorkbenchRadioOn/Off` showcase instances carry `layout_icon_size = 16.0`; the concrete showcase instances carry `layout_spacing = 9.0`, and `WorkbenchCheckboxOff` / `WorkbenchRadioOff` declare the audited unchecked fill and border directly. `WorkbenchToggleRoot` and `WorkbenchToggleOn` carry `track_width = 34.0`, `track_height = 18.0`, `thumb_size = 12.0`, and `layout_spacing = 10.0`; the strict theme owns idle track/edge/thumb tones plus the low-emphasis checked `#173942` track, `#414b54` border, and `#a4aeb4` thumb. Retained projection maps `layout_spacing` into `TemplatePaneNodeData.layout_content_offset_x`, `track_height` into `layout_content_offset_y`, `thumb_size` into `layout_icon_size`, and `track_width` into `value_number`, and the native painter consumes those declared values and style colors before falling back to Rust defaults. The retained bridge still owns state mutation and binding dispatch; this slice only changes the final native visual primitive.

Tabs and segmented controls now follow the same bottom-up painter route. `template_segmented_controls.rs` recognizes the Workbench drawer tabs, Labs tabs, and segmented input, draws active tabs as text plus an accent underline instead of a filled selected button, and paints declared segmented `options` into stable equal-width cells with a selected middle segment. `TemplatePaneNodeData` also projects `label_text`, `label_color`, `label_brightness`, `layout_offset_x`, and `layout_offset_y`, so `WorkbenchInputSegmented` can draw the softer `Segmented Control` label above a 30 px body inside one 48 px authored node, and the new `WorkbenchLabsTabOne/Two/Three` strip can consume the accepted 3 px / 2 px tabs offset from the HTML audit. `WorkbenchLabsTabs` now also declares the audited `#1c2226` container background; retained projection exposes it as resolved button-style metadata, and the tab painter consumes declared idle backgrounds before state colors. The selected segment contract now carries optional `selected_segment_border_width`, `selected_segment_underline_height`, and `selected_segment_underline_color` style metadata through both retained projection paths; `WorkbenchIconToggleSegmented` uses that declaration to suppress the selected outer border while keeping the darker selected fill and a 1 px semi-transparent cyan underline. The idle segmented body shell now uses the same audited `#1d2327` tone in both `template_segmented_controls.rs` and `editor_workbench_strict.zui`, while hover/pressed/disabled states still use the shared state palette. `WorkbenchInputSegmented` and `WorkbenchToggleOn` are now laid out under `WorkbenchComponentLabs`, matching the HTML reference's Labs stack while preserving their existing control ids and preview routes; the Inputs and Checkboxes/Radios columns no longer own those Labs-only samples. Labs tab clicks route through ComponentLabPreview selection state, keeping the component drawer visually and interactively closer to the reference without turning showcase-only controls into production editor commands.

Component-drawer sliders now consume the existing `RangeField` projection directly. `template_sliders.rs` recognizes Workbench slider nodes and draws a compact track, filled range, thumb halo, left-side row label, and right-side value chip before the generic template fallback. The final visual state is selected by `style_selector/workbench_slider.rs`, which consumes the shared `UiPainterState` / `UiPainterResolvedState` priority model for disabled, pressed, focused, hovered, dragging, and drop-hover slider states. `template_sliders.rs` keeps geometry, range spans, tick placement, and command emission so slider state priority is shared with the other selector-owned Workbench controls instead of living as direct node-state branches in the painter. The drawer now declares a dedicated 260 px `WorkbenchComponentSliders` column with `WorkbenchSlidersTitle`, so the three slider samples are no longer compressed into Inputs; the neighboring component columns now match the reference proportions more closely with Inputs at 214 px, Checkboxes/Radios at 168 px, and Labs at 236 px. The slider rows keep explicit `WorkbenchInputSlider`, `WorkbenchInputRangeSlider`, and `WorkbenchInputStepsSlider` ids with `Value`, `Range`, and `Steps` labels, authored chip text (`0.75`, `0.80`, `3`), the audited base-track `background_color = "#2a3338"`, fill `track_fill_color = "#2099a2"`, label `#889399`, 9 px thumb size, `track_offset_x = -10.0`, `track_width_delta = 18.0`, Range `range_min = 20.0`, and Steps `step_tick_count = 5.0`. The latest thumb retune is declaration-owned as `thumb_color = "#b7f1f8"`, `thumb_outline_color = "#2ab1bc33"`, and `thumb_halo_color = "#32d3de3d"` on each slider row. Both retained projection paths map the track/range/tick/thumb aliases into the existing generic layout and color slots on `TemplatePaneNodeData`; the painter consumes those fields only for Workbench slider geometry and thumb chrome, including the Range dual-thumb span and Steps tick marks, while pointer capture and value mutation remain owned by the retained bridge.

The component drawer body now follows the HTML/CSS reference's coarse layout bands. `WorkbenchComponentDrawerBody` is a vertical container with a 202 px `WorkbenchComponentTopRow` for the six component columns plus the side List/Menu stack, and a `WorkbenchComponentLowerRow` for lower table/feedback demos. `WorkbenchComponentList` keeps the List rows and popup Menu sample only; `WorkbenchComponentTable` now owns the `Table` section title plus `WorkbenchTableGroup`, so table rows no longer consume vertical space inside the side list column. `WorkbenchComponentFeedback` owns the lower feedback samples, including `WorkbenchFeedbackToastColumn` and the standalone `WorkbenchToastRoot`, so the lower toast sample is now part of the native component drawer rather than a later completion item.

Workbench icon buttons now have a general native painter route for the top toolbar, activity rail, scene-panel actions, and component-drawer mini buttons. `template_icon_buttons.rs` recognizes Workbench `IconButton` nodes after status controls but before the generic Material/image fallback, draws toolbar/rail/panel-specific chrome, and renders the common glyph set directly from `control_id` / `icon_name` including menu, file, folder, save, cursor, move, rotate, scale, snap, play, grid, sun, plus, trash, filter, cube, graph, image, audio, code, eye, lock, and overflow. The final chrome is selected through `style_selector/workbench_icon_button.rs`, which consumes the shared `UiPainterState` priority model for disabled, pressed, focused, selected, checked, hovered, open, drag, and drop-hover states before the painter emits quads and glyph segments. Input state and bindings remain owned by the retained bridge; this painter slice only makes icon pixels deterministic when SVG assets are missing, inconsistent, or too generic for the reference workbench.

The component drawer icon-button lane is now a separate 210 px `Icon Buttons` column rather than a compressed row inside `Buttons`. The eight panel icon buttons use 38 px frames, declared 18 px `icon_size`, and the audited 1.35 px vertical offset; `template_icon_buttons.rs` consumes that declared icon size and paint offset. The large mini icon buttons now carry the accepted edge pass directly in `.zui`: `border_color = "#171f26"` and `corner_radius = 10.0` across all eight buttons. Non-danger mini icon buttons keep the audited normal panel surface `#272d31` and glyph `#98a3a8`; `WorkbenchMiniDelete` keeps danger fill/glyph semantics while accepting the shared dark edge. The painter consumes declared radius and border values after disabled/selected/pressed/hover/focus priority, with declared danger border values taking precedence over the red fallback border. The `Toggle Buttons` segmented control is declared as `WorkbenchIconToggleSegmented` with `grid/list/columns` options, a preview change route that cycles the retained value state, `selected_segment_border_width = 0.0`, `selected_segment_underline_height = 1.0`, and `selected_segment_underline_color = "#32d3de7a"`.

List and menu rows now have their own native adornment pass. `template_list_rows.rs` draws Workbench list-row text, selected-row fill, right-side check marks, normal-row chevrons, and disabled-row markers without depending on generic `selected` surface behavior. The list-row painter now also consumes declared `background_color`, `text_color`, and `icon_color` through the existing `TemplatePaneNodeData` style fields, so `WorkbenchListSelected` can carry the audited teal selected fill, cyan label tone, and bright right-check tone from `.zui` while ordinary and disabled list rows keep theme fallbacks. The sample list stack is now declared as a transparent `WorkbenchListGroup` with zero internal gap, so the side panel's outer spacing no longer separates the three list rows. `template_popup_rows.rs` consumes the existing structured dropdown/menu rows plus declarative `menu_items` flags such as `icon=save`, `danger`, and `submenu`, so popup rows can show right-aligned command icons, destructive text/icon color, selected option checks, and submenu chevrons while keeping layout and input dispatch in the retained bridge. Final popup row chrome is selected through `style_selector/workbench_popup_row.rs`, which consumes the shared `UiPainterState` priority model for disabled, pressed, focused, hovered, selected, and checked rows before the painter emits row quads, selection marks, text, shortcuts, and adornments. The first popup primitive generalization lives in `template_bridge/popup_primitives.rs`: shared string-list parsing, menu item state parsing, and transient row-flag cleanup are no longer Workbench-only helpers. The component-drawer side-stack `WorkbenchPopupMenu` now uses a fixed 145 px `.zui` width so the native popup body matches the reference crop instead of stretching across the full side panel.

The lower component table now uses a real four-row table group instead of three direct list-column children. `workbench_component_drawer.zui` wraps `WorkbenchTableHeader`, `WorkbenchTableItem`, `WorkbenchTableSelected`, and `WorkbenchTableTail` in a transparent `WorkbenchTableGroup` with zero vertical gap; `WorkbenchTableItem` carries the `Item_01 / Mesh / 2.4 MB / 2m ago` row and the declared `layout_first_cell_offset_x = 4.0` normal-row inset. The component-lab preview state now treats `WorkbenchTableItem`, `WorkbenchTableSelected`, and `WorkbenchTableTail` as one exclusive selected group, so coordinate or direct click dispatch can move selection across all visible reference table rows without static screenshot state.

Scene tree rows now use the same native primitive route. `template_tree_rows.rs` recognizes declared `TreeRow` / `workbench-tree-row` nodes and generated `WorkbenchSceneVirtualItem*` rows before the generic Label/ListRow fallback, then paints hierarchy indentation guides, expanded/collapsed chevrons, small object-type icons, selected/hover/pressed row chrome, and right-side visibility/lock or overflow actions. The existing scene-tree snapshot sync, virtual row materialization, hit testing, and `Hierarchy.SelectEntity` route ownership remain unchanged; this slice only replaces fallback text pixels with a Workbench-specific hierarchy vocabulary. The retained projection contract now verifies both scene-tree rows and Inspector component-property overflow rows declare the same repeat metadata shape, including `node_path_namespace = "v2"`, so future template edits cannot silently move generated rows out of the Workbench v2 node-path space.

The bottom status bar now uses the same native primitive route. `template_status_controls.rs` recognizes the authored `WorkbenchStatus*` nodes before the generic Label/IconButton pass, draws left-side status marks, low-contrast grid/snap/zoom dropdown chips, and the snap/world/target icon buttons as explicit quads/text instead of relying on placeholder icon assets. The final color and interaction state are selected through `style_selector/workbench_status_control.rs`, which consumes the shared `UiPainterState` / `UiPainterResolvedState` priority model for disabled, loading, pressed, focused, open, dragging, drop-hover, selected, checked, and hovered states before the painter emits geometry and glyph segments. `WorkbenchStatusReady` now carries its measured ready declaration through `.zui`: `layout_offset_x`, `layout_offset_y`, `layout_gap`, `text_color = #8f9aa0`, `icon_fill`, and `icon_size` project into the retained host contract and are consumed by the status painter without moving the 9 px dot or following items. The `No Errors` status mark uses the audited `#58b866` fill and now projects `icon_color`/`icon_stroke` into `TemplatePaneNodeData.icon_color`, so its internal check mark consumes the declared `#112018` foreground instead of relying on the generic dark mark fallback. It also declares `layout_icon_size = 12.04`; the painter keeps the outer 14 px status icon slot for label placement and uses the declared size only for centered mark painting, matching the browser `scale(0.86)` without moving following status items. The right-side chip/icon-button normal borders use the audited `#242c32` status-right edge from the HTML/CSS reference while focused/pressed/selected states still use the shared focus-ring path. `WorkbenchStatusWarnings` carries its measured warning declaration through `.zui`: `layout_gap`, `text_color`, `icon_fill`, `icon_color`/`icon_stroke = #11181a`, `icon_stroke_width = 1.45`, `icon_size`, and `icon_offset_y` project into the retained host contract and are consumed by the status painter before theme fallbacks, keeping the internal mark color and softened mark width native without changing the 21 px warning icon slot. `WorkbenchStatusMessages` follows the same declaration path for its message row offsets, text color, info fill, icon size, and icon offset. The right-side status defaults now live on `WorkbenchWindowStatusBar`: `status_right_offset_y` is inherited into grid/snap/zoom chips and snap/world/target icon buttons, while `status_right_text_color` is inherited into grid/snap/zoom chip labels when the leaf does not override `text_color`. This keeps the low-contrast right-side labels and vertical offset tied to one `.zui` declaration instead of repeated leaf props. The `.zui` status bar remains the layout authority for item width and ordering; this painter slice only supplies the missing pixel vocabulary for the declared primitives.

Component-drawer feedback rows, the feedback tooltip, and the notification toast now have the same Workbench-specific painter route. `template_alerts.rs` recognizes `WorkbenchInfoAlert`, `WorkbenchSuccessAlert`, `WorkbenchWarningAlert`, `WorkbenchErrorAlert`, and the standalone `WorkbenchToastRoot` before the generic Material Alert fallback. It draws compact tinted alert rows, deterministic severity glyphs, the teal toast status mark, `UNDO` action, close affordance, and the audited toast surface/border pair from the HTML/CSS prototype: `rgba(21, 48, 53, 0.97)` surface with an `rgba(53, 199, 208, 0.08)` border. `template_tooltips.rs` recognizes `WorkbenchTooltipRoot` before the generic Material fallback and draws the 96 px dark bubble, 8 px declared arrow, title/body text, shadow, and cyan info mark. `workbench_component_drawer.zui`, `workbench_toast.zui`, and `workbench_tooltip.zui` remain the layout and text authority, with the latest tooltip tones declared as `#171c20` bubble/arrow fill, `#252d32` border, `#d0d9dd` title, `#a8b3b8` body, and `#259ca7` info mark. The toast root projects its declared `status_mark_size`, `status_mark_color`, and `action_color`; the tooltip root projects `arrow_size` and `arrow_color`; both paths keep geometry and action/arrow tones editable from `.zui` rather than hardcoded in the painter. The drawer structure now separates the feedback region into `WorkbenchFeedbackAlerts`, `WorkbenchTooltipRoot`, and `WorkbenchFeedbackToastColumn`; the standalone `feedback_toast` instance deliberately leaves its instance `control_id` empty so component expansion exposes exactly one `WorkbenchToastRoot`, while the four alert samples retain their own IDs.

Dedicated source assets must match that root-shell authority unless they are deliberately exercising a standalone fallback. `floating_window_source.zui` therefore uses a `57px` top spacer and `44px` rail so floating-window default/clamp frames line up with the document host. The drawer source frame recompute remains owned by `workbench_drawer_source/layout.rs`; `BuiltinHostWindowTemplateBridge` passes `WorkbenchBody` and `StatusBarRoot` anchors into that owner for real workbench projections so visible drawer frames are recomputed from the current root shell without naming a retired standalone drawer-source UI asset.

The source bridges keep their shared surfaces alive across layout recompute. Drawer-source construction still builds the initial surface from `EditorUiHostRuntime`, but subsequent standalone, workbench-model, and anchored recomputes mutate the existing `UiSurface`, mark root layout dirty when shell size is an input, and call `UiSurface::rebuild_dirty(...)`. Floating-window source recompute follows the same retained-surface pattern. This keeps node ids, render-cache state, and bridge-local surface state stable while still letting runtime layout/render rebuild only the dirty domains.

## Input Authority

All high-frequency workbench input is host-owned but shared-surface-first. The retained host uploads pointer facts and lets shared `UiSurface` / `UiPointerDispatcher` routes decide hits, capture, and semantic delivery.

Current retained bridge families include:

- menu and popup routes through `retained_host::menu_pointer` and `callback_dispatch::shared_pointer::menu`
- activity rail, host page, document tab, drawer header, and viewport toolbar routes through their retained pointer bridge modules
- shell drag/drop and splitter routes through `retained_host::shell_pointer`
- hierarchy, asset tree/content/reference, welcome recent, and scroll-only pane routes through retained list and detail pointer bridges
- viewport body and toolbar routes through `callback_dispatch::viewport` and `RetainedViewportController`

Stable editor events are produced after route resolution through template bindings and editor runtime dispatch. The host does not keep a second direct business callback path for list selection, tab activation, drawer toggles, menu selection, or pane surface actions.

Decorative Workbench scene nodes are part of the render/layout surface, not the input surface. Template hit testing only registers nodes with an authored action, binding, edit/commit route, dispatch kind, or input-field role, and the componentized Workbench pointer path must return no feedback or runtime event when the pointer lands on purely visual viewport layers such as the floor grate.

## Host Contract And Painter

`retained_host::host_contract` is the Rust-owned DTO and native-window seam. It contains the presentation data, pane/context globals, input translation, surface hit testing, native pointer dispatch, redraw decisions, presenter, and painter modules.

The painter consumes host-contract data and shared template render commands. It is allowed to provide native fallback pixels for shell chrome, text, icons, viewport images, diagnostics overlays, close prompts, and retained template nodes. It must not introduce a second layout or business-state authority; arranged frames and stable action ids come from `.zui`, shared surface projection, and editor workbench data.

The host contract also carries shared component classification tokens. `TemplatePaneNodeData` exposes `component_category` and `component_layout_role`, populated from the runtime component descriptor registry through `component_contract_metadata.rs` for both generic pane projection and the componentized Workbench window projection. Native painters and diagnostics should use these tokens for broad families such as input, selection, collection, container, flex, grid, popup, and virtual-list before adding control-id-specific detail.

`template_component_family.rs` is the first shared native consumer of those tokens. It resolves a
`TemplatePaneNodeData` node into Button, IconButton, TextInput, Slider, Checkbox, Radio, Toggle,
Dropdown, Tab, SegmentedControl, ListRow, TreeRow, TableRow, Popup, Tooltip, Alert, Drawer, Window,
or related container families using component role, host role, category/layout role, and legacy
Workbench id fallback. The individual painter entry points and the template hit-test surface now
ask for that component family before drawing or routing Workbench button, input, dropdown,
selection, slider, tab, list, tree, and table behavior.
Workbench-specific skinning remains behind `uses_workbench_visual_language`, so the classification
layer can later serve generic Material/editor components without forcing the dark Workbench shell
style onto them.

`template_input_semantics.rs` is the first input-system consumer of the same family contract. It
owns the "is this hit a text input?" and "which edit target receives text?" rules for native pointer
focus, using `TemplateComponentFamily::TextInput` plus the existing welcome-text and legacy
role-string fallbacks. This keeps input behavior aligned with component categories instead of
spreading more role-string checks through `native_pointer.rs`.

`template_activation_semantics.rs` owns the primary-click callback route for template node hits. It
classifies focus-only text inputs, Inspector controls, asset click/change controls, welcome controls,
component showcase controls, structured dropdown options, popup menu rows, binding-backed controls,
and action-backed controls before invoking `PaneSurfaceHostContext`. `native_pointer.rs` remains
responsible for pointer state, focus timing, and damage regions, but no longer owns the full
template-node activation match.

Visual asset pixels stay inside that host-contract seam. `painter/visual_assets.rs` resolves runtime `UiVisualAssetRef` values and template image/icon metadata through the editor asset tree, then rasterizes SVG sources with `resvg` at the requested host paint target size. Missing assets still fall back to native placeholder behavior, but decoded SVG or bitmap pixels are clipped and alpha-blended by the Rust-owned painter rather than by a restored generated UI layer.

## Hard Cutover From Deleted Slint Host

The old owner path was `zircon_editor::ui::slint_host` and the old source tree included `zircon_editor/ui/**/*.slint`. Those paths are historical only. They must not be restored as a compatibility module, shim, facade, re-export, generated include, build dependency, or active documentation owner.

Remaining references to Slint are allowed only as historical cutover context, no-Slint guard wording, or dependency-deletion evidence. Current code, tests, docs, and validation commands should use `retained_host`, `.zui`, and Rust-owned `host_contract` names.

## Validation

The retained viewport controller installs an editor-only `editor-viewport-default` quality profile
when it creates a render-framework viewport. That profile requests Hybrid GI without implicitly
requesting Virtual Geometry. Both world-only and world-plus-UI submissions normalize the settings-only
HGI extract by enabling it and filling zero trace/card/voxel budgets; authored nonzero budgets and
other HGI settings are preserved. A failed profile install destroys the just-created viewport before
the error is returned.

The focused controller lifecycle test records both viewport quality profiles and both submitted HGI
extracts and passed 1/1. App-level tests separately prove that Editor requests HGI by default, Runtime
does not, and an advanced provider catalog selects HGI without VG. The editor-host Cargo preset still
needs its advanced catalog feature relation from the active Frameworks 03 manifest owner, so this is
not yet recorded as an end-to-end editor WGPU product result.

The retained shell is guarded by:

- source tests that reject active deleted UI source files and generated build seams
- retained host window and boundary tests under `zircon_editor/src/tests/host/retained_window`
- retained pointer tests under `zircon_editor/src/tests/host/retained_*`
- template-runtime tests under `zircon_editor/src/tests/host/template_runtime`
- integration-contract readers for `workbench_retained*`
- editor boundary tests for `.zui` host assets and workbench projection cutover

The milestone validation target remains `cargo check -p zircon_editor --lib --locked --message-format=short`, `cargo check -p zircon_editor --lib --tests --locked --message-format=short`, and then the repository validator when unrelated active-workstream blockers are clear or classified.

The 2026-05-09 workspace rerun reached the retained-host test build after the earlier reflection blocker moved forward. The retained-host slice removed all obsolete `i_retained_backend_testing::init_no_event_loop()` calls and kept `ModelRc` construction on the Rust-owned `VecModel`/`ModelRc` path. This keeps the hard cutover honest: retained tests instantiate `UiHostWindow` directly and do not reintroduce a generated or toolkit-backed test backend dependency.

The same 2026-05-09 geometry cleanup revalidated focused retained-host authority tests after the shell moved to `24 + 32 + 1 = 57` top chrome and `44px` rail sizing. Focused passes covered `workbench_projection`, `drawer_source_projection`, `floating_window_source`, `shared_surface`, `retained_host_page_pointer`, `retained_activity_rail_pointer`, and `retained_callback_dispatch::workbench::template_bridge` using `target\codex-shared-a`; warnings remain existing unused/dead-code warnings.

The 2026-05-10 retained/template performance follow-up strengthens `drawer_source_projection` and `floating_window_source` reuse guards. They now require stable source-surface node ids, stable render command counts, layout recomputation through `rebuild_dirty(...)`, and positive render-command reuse. A bridge that replaces its surface with a newly instantiated deterministic tree no longer satisfies the test.

The later menu-pointer timeout was classified as a command-budget artifact rather than a retained menu-pointer hang. The focused `native_root_menu_pointer_click_dispatches_shared_menu_action_in_real_host` test passed in isolation, and the next visible full-suite stop points, `native_frame_request_recomputes_dirty_layout_before_presentation` and `child_window_viewport_pointer_event_focuses_source_window_before_runtime_dispatch`, also passed in isolation. A redirected serial run of `cargo test -p zircon_editor --lib --locked --target-dir target\codex-shared-a -- --test-threads=1` completed with `1162 passed; 0 failed; 4 ignored` in 2018.92s. A fresh 2026-05-09 recheck repeated the native menu-pointer case with 1 passed / 0 failed in 14.05s after compile, then repeated the full redirected serial gate with `1162 passed; 0 failed; 4 ignored` in 2126.68s.

The final retained-host workspace validator attempt used `./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -TargetDir target\codex-shared-a`. It passed `cargo build --workspace --locked`, then failed during `cargo test --workspace --locked` in `zircon_plugin_navigation_runtime --lib`. The focused reproduction `cargo test -p zircon_plugin_navigation_runtime --lib --locked --target-dir target\codex-shared-a -- --nocapture --test-threads=1` failed with `5 passed / 8 failed`; the failure cluster is world-driven navigation scans seeing missing/default dynamic authoring components after world mutation, while direct manager/navmesh checks still pass. That blocker is outside the retained editor host and does not justify restoring `slint_host`, generated UI, or backend-testing dependencies.

For Workbench visual evidence, `native_workbench_reference.rs` can write the current componentized editor render buffer as a PNG by setting `ZIRCON_WRITE_WORKBENCH_PREVIEW=1` and `ZIRCON_WORKBENCH_PREVIEW_PATH=<png>` on the focused native preview test. This produces comparable editor-window output from the real retained/native path instead of returning to the HTML prototype as the deliverable.

The latest primitive-family routing pass wrote
`target/editor-workbench-visual-check/editor-workbench-native-family-1672x941.png` from that native
test path and generated
`target/editor-workbench-visual-check/editor-workbench-design-vs-native-family-1672x941.png` as the
primary side-by-side comparison against `docs/ui-and-layout/workbench.png`.

After the input-family routing continuation, the same native test wrote
`target/editor-workbench-visual-check/editor-workbench-native-input-family-1672x941.png` and the
updated comparison
`target/editor-workbench-visual-check/editor-workbench-design-vs-native-input-family-1672x941.png`.

After the activation-family routing continuation, `native_workbench_reference` wrote
`target/editor-workbench-visual-check/editor-workbench-native-activation-family-1672x941.png` and the
updated comparison
`target/editor-workbench-visual-check/editor-workbench-design-vs-native-activation-family-1672x941.png`.

After the visible Effect module-workspace continuation, the same native path wrote
`target/editor-workbench-visual-check/editor-workbench-native-module-workspace-1672x941.png` and
`target/editor-workbench-visual-check/editor-workbench-gameplay-effect-vs-native-module-workspace-1672x941.png`.
This pass keeps the existing Workbench outer shell, inserts the module workspace under the main
band with a rail-alignment gap, and uses the shared Workbench field/list/table/property primitives
instead of returning to the browser sample as the implementation target.

The follow-up multi-module route continuation keeps that same layout strategy for Material,
Behavior, Assets, and VFX. Their workspaces now share the fixed rail gap plus three-column
left/center/right panel grammar, and representative dropdowns, text fields, graph rows, asset rows,
and VFX property rows dispatch `WorkbenchModule/*` preview actions through the shared template
binding and preview-action registry. `module_navigation.rs` owns the new row groups so the retained
bridge can switch selected rows without adding another layout rule. Current validation for this
slice covers TOML parsing, Rust formatting, targeted diff checking, whitespace scanning, a direct
reference-surface test-binary pass, and a direct `native_workbench_reference` screenshot pass. The
latest native output is
`target/editor-workbench-visual-check/editor-workbench-native-multimodule-routes-1672x941.png`, and
the refreshed comparison against the accepted AI gameplay-effect shell is
`target/editor-workbench-visual-check/editor-workbench-gameplay-effect-vs-native-multimodule-routes-1672x941.png`.
The bridge regressions for module navigation state and route projection also pass from the
refreshed test binary after correcting their runtime-node expectation to
`WorkbenchModuleWorkspace`.

The native additional-module continuation keeps the same outer Workbench shell and mounts Ability,
Tags, Perception, Render, and HUD through `workbench_additional_module_workspaces.zui`. Those panels
reuse the existing rail gap plus left/center/right grammar, so the new module count changes
content and navigation state rather than introducing another page layout. Their controls are
preview-only: every declared `WorkbenchModule/*` event maps through
`workbench_module_template_bindings.rs`, the shared preview-action registry, and
`module_navigation.rs`, which provides selected tab, row, command, and workspace feedback while
`workbench_window_template_bindings.rs` stays focused on the rest of the window shell. Real editor
data binding remains a later milestone.

The module-field edit bridge now gives those module panels a shared native field response path.
`module_field_edit.rs` handles `WorkbenchModule/*` Change/Submit bindings backed by
`EditWorkbench*` or `CommitWorkbench*` preview actions, verifies that the edited control owns the
route, updates `value` and `value_text`, refreshes retained projection, and returns paint-only
invalidation. This is still preview data, but it uses the same native edit callback lane as
Inspector fields instead of treating module fields as click-only samples. The focused native
regression `componentized_workbench_module_field_edit_updates_value_preview` passed from the
refreshed `zircon_editor` test binary with 1 passed / 1837 filtered. The refreshed native visual
evidence is
`target/editor-workbench-visual-check/editor-workbench-native-module-field-edit-1672x941.png`, with
the side-by-side comparison at
`target/editor-workbench-visual-check/editor-workbench-gameplay-effect-vs-native-module-field-edit-1672x941.png`.

The follow-up native keyboard pass keeps the same module-field bridge but proves the host window
input path reaches it. `native_workbench_module_field_focuses_edits_and_commits_from_keyboard`
switches to the Ability workspace, focuses `WorkbenchAbilityNameField`, inserts `_Preview`, and
asserts both edit and commit callbacks carry the updated module value. The refreshed test binary
passed that focused test with 1 passed / 1838 filtered, then passed the full
`native_workbench_reference` screenshot run with 16 passed / 1823 filtered. Current native visual
evidence is
`target/editor-workbench-visual-check/editor-workbench-native-module-field-keyboard-1672x941.png`;
the comparable window artifact against `ai-gameplay-effect-layout.png` is
`target/editor-workbench-visual-check/editor-workbench-ai-gameplay-vs-native-module-field-keyboard-1672x941.png`.
The same pass corrected module switching so module mode keeps `WorkbenchSceneWorkspace` visible;
the activity rail remains part of the outer Workbench shell, and only the module overlay plus
individual module workspaces change visibility.

The module-navigation shell follow-up now validates that contract directly. The refreshed editor
test binary passed `workbench_module_tabs_switch_exactly_one_module_workspace`,
`workbench_scene_tab_restores_scene_workspace_and_hides_module_workspaces`,
`componentized_workbench_module_field_edit_updates_value_preview`, and
`native_workbench_module_field_focuses_edits_and_commits_from_keyboard`. The full
`native_workbench_reference` screenshot run passed 16 tests / 1823 filtered and wrote
`target/editor-workbench-visual-check/editor-workbench-native-module-navigation-shell-1672x941.png`;
the comparable window artifact against `ai-gameplay-effect-layout.png` is
`target/editor-workbench-visual-check/editor-workbench-ai-gameplay-vs-native-module-navigation-shell-1672x941.png`.

The visible-frame follow-up separates internal shell-frame bookkeeping from bridge-facing frame
queries. The template surface keeps raw frame lookup for required controls, while the retained
bridge filters frames through render visibility before exposing them to tests and native dispatch.
This prevents collapsed module workspaces from returning visible frames without letting required
outer shell controls disappear during state refresh. Focused validation passed for both module
navigation tests, decorative viewport pointer and host hit-test regressions, the broader
`workbench_module` filter, and scene/inspector snapshot sync in
`E:\cargo-targets\zircon-editor-workbench-preview-0603`.

The module-command feedback continuation keeps the same shell and module grammar but gives module
commands a visible native response. `module_command_feedback.rs` maps preview action ids such as
Ability Playtest, Render Compile, Browse, Import, Simulate, Preview, and Validate onto existing
Workbench status items plus the relevant module output row. The bridge mutates declared `text` or
`value_text` properties on existing StatusItem/ListRow/TableRow controls, then refreshes retained
projection without changing layout or adding per-button pixel logic. Validation passed in
`D:\cargo-targets\zircon-editor-workbench-command-feedback` for the focused command feedback
regression, adjacent module navigation and field-edit regressions, and the native screenshot test.
Current visual evidence is
`target/editor-workbench-visual-check/editor-workbench-native-command-feedback-1672x941.png`; the
comparison artifact is
`target/editor-workbench-visual-check/editor-workbench-ai-gameplay-vs-native-command-feedback-1672x941.png`.

The toolbar window-menu follow-up keeps the interaction in the Rust-owned editor shell. The
window template root now owns an Overlay with one vertical workbench-content child and three
`WorkbenchPopupMenu` wrapper siblings for Main, Run Mode, and Layout menus. The top toolbar remains
a horizontal trigger component. `window_menu_state.rs` owns the small menu table, so triggering one
menu opens that menu, closes the other toolbar menus, and mirrors selected/checked state onto the
trigger control. Popup item selection still goes through the shared `popup_state.rs`
close/writeback path, and runtime `popup_menu.rs` expands the wrapped `ContextActionMenu`
`menu_items` into visible row render commands for the screenshot path. The window template imports
the Workbench popup wrapper directly so production asset governance can trace that `.zui` component
from a production `.v2.ui.toml` entry point.

The next runtime render slice applies the same component-level popup path to dropdown-style
controls. `zircon_runtime::ui::surface::render::popup_rows` now owns the shared popup row visual
description, while `popup_options.rs` expands open `Dropdown`, `ComboBox`, and `Select` nodes from
their authored `options` and option-state props into render commands below the trigger. Material
Domain, Gameplay Effect policy, Inspector selects, and component-drawer dropdowns therefore share
one open-state row renderer instead of each panel painting a custom popup.

The refreshed native Material-module dropdown evidence was generated from the real retained editor
test binary, not the browser sample. The focused test
`componentized_workbench_module_dropdown_open_paints_native_preview_pixels` passed with 1 test /
1848 filtered and wrote
`target/editor-workbench-visual-check/editor-workbench-native-module-dropdown-open-popup-options-1672x941.png`;
the AI-reference comparison is
`target/editor-workbench-visual-check/editor-workbench-ai-material-vs-native-module-dropdown-open-popup-options-1672x941.png`.

The next low-level component continuation moves selection controls into that same runtime render
extract path. `selection_controls.rs` expands authored `Checkbox`, `Radio`, `Toggle`, and `Switch`
nodes into reusable mark/tick, dot, track/thumb, and inline-label commands, reading both template
props and runtime `UiStateFlags`. This keeps checkbox/radio/toggle behavior aligned with the
componentized Workbench `.zui` primitives instead of depending on one-off retained painter branches
when the editor screenshot path consumes render commands.

The selection-control runtime extract path is covered by
`render_extract_expands_selection_control_indicators`, which passed in
`D:\cargo-targets\zircon-editor-workbench-selection-controls` with checked checkbox/radio/toggle
mark, dot, track/thumb, and single-label command assertions.

The full native workbench screenshot path also passed through
`componentized_workbench_surface_paints_native_preview_pixels_and_interaction_state` with
`ZIRCON_WRITE_WORKBENCH_PREVIEW=1`, writing
`target/editor-workbench-visual-check/editor-workbench-native-selection-controls-1672x941.png`.
The direct reference-to-native comparison image for this slice is
`target/editor-workbench-visual-check/editor-workbench-reference-vs-native-selection-controls-1672x941.png`,
using `docs/ui-and-layout/workbench.png` as the left-side reference and the native editor render
extract output as the right-side window.

The next HTML-prototype parity pass applies the same runtime extract approach to sliders.
`sliders.rs` expands `RangeField`, `Slider`, and `RangeSlider` into label, track, fill, tick,
thumb/halo, range-min value, and value-box render commands, reading the same flat props that the
component prototype and `.zui` assets use for value, min/max, labels, ticks, and state colors.
`render_extract_expands_slider_primitives` passed in
`D:\cargo-targets\zircon-editor-workbench-sliders`; the first run also exposed and fixed an
unrelated runtime test typo where non-finite dynamic-resolution scale used the `Real` time marker
instead of `f32::NAN`.
The current HTML-template/native comparison for this pass is
`target/editor-workbench-visual-check/editor-workbench-html-template-vs-native-runtime-components-1672x941.png`.

Validation for this toolbar slice passed in
`D:\cargo-targets\zircon-editor-workbench-command-feedback` with
`cargo test -p zircon_runtime --lib render_extract_expands_open_context_action_menu_items --locked
--jobs 1` and `cargo test -p zircon_editor --lib
componentized_workbench_toolbar_run_menu_paints_native_preview_pixels --locked --jobs 1` with
`ZIRCON_WRITE_WORKBENCH_PREVIEW=1`. The native screenshot is
`target/editor-workbench-visual-check/editor-workbench-native-toolbar-run-menu-open-1672x941.png`;
the side-by-side comparison against the current web component template is
`target/editor-workbench-visual-check/editor-workbench-web-template-vs-native-toolbar-run-menu-open-1672x941.png`.
