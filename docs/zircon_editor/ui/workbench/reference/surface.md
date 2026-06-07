---
related_code:
  - zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - zircon_editor/assets/ui/theme/editor_workbench_strict.v2.ui.toml
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_activity_rail.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\chrome\workbench_axis_value_field.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_button.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_checkbox.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\chrome\workbench_chip.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_component_drawer.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_dropdown.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_field.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_icon_button.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_inspector_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\data\workbench_list_row.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_additional_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\feedback\workbench_popup_menu.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\data\workbench_property_row.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_radio.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\chrome\workbench_rail_button.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_scene_tree_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\chrome\workbench_section_title.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_segmented_control.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_slider.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_status_bar.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\feedback\workbench_status_item.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_tab.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\data\workbench_table_row.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\feedback\workbench_toast.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_toggle.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\data\workbench_tree_row.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_viewport_panel.zui
  - zircon_runtime_interface/src/ui/v2/repeat.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime_interface/src/ui/v2/arena.rs
  - zircon_runtime/src/ui/v2/surface_tree/node.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/reference/mod.rs
  - zircon_editor/src/ui/workbench/reference/ids.rs
  - zircon_editor/src/ui/workbench/reference/metrics.rs
  - zircon_editor/src/ui/workbench/reference/tokens.rs
  - zircon_editor/src/ui/workbench/reference/surface.rs
  - zircon_editor/src/ui/workbench/reference/template_surface.rs
  - zircon_editor/src/ui/workbench/reference/builder/mod.rs
  - zircon_editor/src/ui/workbench/reference/builder/nodes.rs
  - zircon_editor/src/ui/workbench/reference/builder/panels.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/popup_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/pointer_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/property_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/transform_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_popup_actions.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/ui.rs
  - zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_menu_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_option_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_popup_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_node_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_property_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_sliders.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/support.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/support.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_inspector_property_edit.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_pointer_preview.rs
  - zircon_editor/src/tests/workbench/mod.rs
  - zircon_editor/src/tests/workbench/reference_surface.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_pointer_preview.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime_interface/src/ui/component/category.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_model.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
implementation_files:
  - zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/components/workbench\primitives\chrome\workbench_axis_value_field.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_button.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_checkbox.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_component_drawer.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_dropdown.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_inspector_panel.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_module_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench\modules\core\workbench_additional_module_workspaces.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench\primitives\inputs\workbench_radio.zui
  - zircon_editor/assets/ui/editor/components/workbench\shell\workbench_status_bar.zui
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/reference/mod.rs
  - zircon_editor/src/ui/workbench/reference/ids.rs
  - zircon_editor/src/ui/workbench/reference/metrics.rs
  - zircon_editor/src/ui/workbench/reference/tokens.rs
  - zircon_editor/src/ui/workbench/reference/surface.rs
  - zircon_editor/src/ui/workbench/reference/template_surface.rs
  - zircon_editor/src/ui/workbench/reference/builder/mod.rs
  - zircon_editor/src/ui/workbench/reference/builder/nodes.rs
  - zircon_editor/src/ui/workbench/reference/builder/panels.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/retained_adapter.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_module_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/virtual_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/popup_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/pointer_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/data_sync.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_field_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/property_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/transform_edit.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/error.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_sliders.rs
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_popup_actions.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/retained_host/ui.rs
  - zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/common/dispatch.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_keyboard.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_popup_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_status_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_dropdowns.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_node_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_property_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_selection_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/support.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/support.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_inspector_property_edit.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_module_navigation.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_pointer_preview.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_window_menus.rs
  - zircon_runtime_interface/src/ui/v2/repeat.rs
  - zircon_runtime_interface/src/ui/v2/asset.rs
  - zircon_runtime_interface/src/ui/v2/arena.rs
  - zircon_runtime/src/ui/v2/surface_tree/node.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime_interface/src/ui/component/category.rs
  - zircon_runtime_interface/src/ui/component/descriptor/component_model.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/list.rs
plan_sources:
  - user: 2026-06-01 Start approximating the zirconEngine editor effect from basic rendering, interaction response, and layout system
  - docs/ui-and-layout/workbench.png
tests:
  - zircon_editor/src/tests/workbench/reference_surface.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_dropdowns.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_selection_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_segmented_controls.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_window_menus.rs
  - Python tomllib parse of workbench_main_band.zui and workbench_module_workspace.zui (2026-06-03 after visible Effect module workspace: passed)
  - rustfmt --edition 2021 --check over template_component_family.rs and reference_surface.rs (2026-06-03 after visible Effect module workspace: passed)
  - cargo test -p zircon_editor --lib reference_workbench_componentized_window_surface_matches_reference_chrome_metrics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-preview-0603 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-03 after visible Effect module workspace: passed, 1 passed / 1834 filtered)
  - direct native_workbench_reference test binary run with ZIRCON_WRITE_WORKBENCH_PREVIEW=1 and ZIRCON_WORKBENCH_PREVIEW_PATH=target/editor-workbench-visual-check/editor-workbench-native-module-workspace-1672x941.png (2026-06-03 after visible Effect module workspace: passed, 15 passed / 1820 filtered)
  - Python tomllib parse of workbench_main_band.zui and workbench_module_workspace.zui (2026-06-03 after multi-module route expansion: passed)
  - rustfmt --edition 2021 --check over workbench_window_template_bindings.rs, workbench_preview_actions.rs, module_navigation.rs, reference_surface.rs, and workbench_projection.rs (2026-06-03 after multi-module route expansion: passed)
  - targeted git diff --check and touched-file trailing-whitespace scan for the multi-module route expansion (2026-06-03: passed)
  - copied direct zircon_editor test binary run with ZIRCON_WRITE_WORKBENCH_PREVIEW=1 and ZIRCON_WORKBENCH_PREVIEW_PATH=target/editor-workbench-visual-check/editor-workbench-native-multimodule-routes-1672x941.png (2026-06-03 after multi-module route expansion: passed, 15 passed / 1820 filtered)
  - direct zircon_editor test binary run for reference_workbench_componentized_window_surface_matches_reference_chrome_metrics (2026-06-03 after multi-module route expansion: passed, 1 passed / 1834 filtered)
  - direct zircon_editor test binary run for componentized_workbench_window_template_bridge_updates_module_navigation_state (2026-06-03 after expanded WorkbenchModuleWorkspace runtime-node assertion: passed, 1 passed / 1834 filtered)
  - direct zircon_editor test binary run for componentized_workbench_window_template_bridge_exports_surface_projection_frames_and_routes (2026-06-03 after expanded WorkbenchModuleWorkspace runtime-node assertion: passed, 1 passed / 1834 filtered)
  - Python tomllib parse of workbench_top_toolbar.zui, workbench_module_workspace.zui, workbench_additional_module_workspaces.zui, and workbench_window.v2.ui.toml (2026-06-03 after additional native module panels: passed)
  - Python coverage script for all 188 declared WorkbenchModule/* events across top toolbar, module workspace, and additional module workspace assets (2026-06-03 after module binding split: passed)
  - rustfmt --edition 2021 --check over workbench_module_template_bindings.rs, workbench_window_template_bindings.rs, module_navigation.rs, workbench_preview_actions.rs, workbench_projection.rs, and reference_surface.rs (2026-06-03 after module binding split: passed)
  - direct zircon_editor test binary run for componentized_workbench_window_template_bridge_exports_surface_projection_frames_and_routes (2026-06-03 after module binding split and host-projection assertion correction: passed, 1 passed / 1834 filtered)
  - direct zircon_editor test binary run for componentized_workbench_window_template_bridge_updates_module_navigation_state (2026-06-03 after additional native module panels: passed, 1 passed / 1834 filtered)
  - direct zircon_editor test binary run for reference_workbench_componentized_window_surface_matches_reference_chrome_metrics (2026-06-03 after host-projection assertion correction: passed, 1 passed / 1834 filtered)
  - direct native_workbench_reference test binary run with ZIRCON_WRITE_WORKBENCH_PREVIEW=1 and ZIRCON_WORKBENCH_PREVIEW_PATH=target/editor-workbench-visual-check/editor-workbench-native-expanded-modules-1672x941.png (2026-06-03 after additional native module panels: passed, 15 passed / 1820 filtered)
  - direct zircon_editor test binary run for componentized_workbench_module_field_edit_updates_value_preview (2026-06-03 after native module-field edit bridge: passed, 1 passed / 1837 filtered)
  - direct native_workbench_reference test binary run with ZIRCON_WRITE_WORKBENCH_PREVIEW=1 and ZIRCON_WORKBENCH_PREVIEW_PATH=target/editor-workbench-visual-check/editor-workbench-native-module-field-edit-1672x941.png (2026-06-03 after native module-field edit bridge: passed, 15 passed / 1823 filtered)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-command-feedback --message-format short --color never (2026-06-03 after native module-command feedback bridge: passed with existing warnings)
  - cargo test -p zircon_editor --lib workbench_module_commands_update_status_and_module_output_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-command-feedback --message-format short --color never -- --nocapture --test-threads=1 (2026-06-03 after native module-command feedback bridge: passed, 1 passed / 1839 filtered)
  - direct D:\cargo-targets\zircon-editor-workbench-command-feedback\debug\deps\zircon_editor-16c136b0ff3b6b9d.exe reruns for workbench_module_tabs_switch_exactly_one_module_workspace, workbench_scene_tab_restores_scene_workspace_and_hides_module_workspaces, componentized_workbench_module_field_edit_updates_value_preview, and native_workbench_module_field_focuses_edits_and_commits_from_keyboard (2026-06-03 after native module-command feedback bridge: passed, 1 passed each / 1839 filtered)
  - cargo test -p zircon_editor --lib componentized_workbench_module_command_feedback_paints_native_preview_pixels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-command-feedback --message-format short --color never -- --nocapture --test-threads=1 with ZIRCON_WRITE_WORKBENCH_PREVIEW=1 and ZIRCON_WORKBENCH_PREVIEW_PATH=target/editor-workbench-visual-check/editor-workbench-native-command-feedback-1672x941.png (2026-06-03 after native module-command feedback bridge: passed, 1 passed / 1840 filtered)
  - Python tomllib parse of workbench_window.v2.ui.toml and workbench_top_toolbar.zui after moving toolbar menus to the window root Overlay with direct ContextActionMenu nodes (2026-06-03: passed)
  - rustfmt --edition 2021 --check over zircon_runtime/src/ui/surface/render/extract.rs, zircon_runtime/src/ui/surface/render/mod.rs, zircon_runtime/src/ui/surface/render/popup_menu.rs, zircon_runtime/src/ui/tests/render_popup_menu.rs, zircon_runtime/src/ui/tests/mod.rs, and native_workbench_window_menus.rs (2026-06-03: passed)
  - cargo test -p zircon_runtime --lib render_extract_expands_open_context_action_menu_items --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-workbench-command-feedback -- --nocapture (2026-06-03 after runtime ContextActionMenu row rendering: passed, 1 passed / 2529 filtered)
  - cargo test -p zircon_editor --lib componentized_workbench_toolbar_run_menu_paints_native_preview_pixels --locked --jobs 1 with ZIRCON_WRITE_WORKBENCH_PREVIEW=1 and ZIRCON_WORKBENCH_PREVIEW_PATH=target/editor-workbench-visual-check/editor-workbench-native-toolbar-run-menu-open-1672x941.png (2026-06-03 after native toolbar window-menu screenshot row rendering: passed, 1 passed / 1847 filtered)
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/transform_edit.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_node_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_property_rows.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - rustfmt --edition 2021 --check over template_buttons.rs, template_dropdowns.rs, workbench_window_projection.rs, pane_component_projection/mod.rs, and workbench_projection.rs (2026-06-02 after Workbench button-row native style sync: passed)
  - Python tomllib parse plus declaration assertions for workbench_component_drawer.zui, workbench_button.zui, workbench_dropdown.zui, and editor_workbench_strict.v2.ui.toml (2026-06-02 after Workbench button-row native style sync: passed)
  - stale-doc scan plus git diff --check over the button-row native style Rust/ZUI/docs/session files (2026-06-02: passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - focused Cargo validation for the button-row native style sync stayed deferred on 2026-06-02 because active cargo/rustc lanes were compiling
  - rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs (2026-06-02 after Workbench Icon/Delete native style sync: passed)
  - Python tomllib parse plus declaration assertions for WorkbenchButtonIcon/Delete foreground_color and corner_radius in workbench_component_drawer.zui (2026-06-02 after Workbench Icon/Delete native style sync: passed)
  - stale-doc scan, git diff --check, and trailing-whitespace scan over the Icon/Delete native style Rust/ZUI/docs/session files (2026-06-02: passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - focused Cargo validation for the Icon/Delete native style sync stayed deferred on 2026-06-02 because active cargo/rustc lanes were compiling
  - rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs (2026-06-02 after Workbench Tertiary/Outline native style sync: passed)
  - Python tomllib parse plus declaration assertions for WorkbenchTertiaryButton/WorkbenchOutlineButton foreground_color, border_color, corner_radius, and layout_offset_x in workbench_component_drawer.zui (2026-06-02 after Workbench Tertiary/Outline native style sync: passed)
  - stale-doc scan, git diff --check, and trailing-whitespace scan over the Tertiary/Outline native style Rust/ZUI/docs/session files (2026-06-02: passed; Git reported LF-to-CRLF working-tree warnings for existing tracked files)
  - focused Cargo validation for the Tertiary/Outline native style sync stayed deferred on 2026-06-02 because active cargo/rustc lanes were compiling
  - 2026-06-02 Workbench Icon Buttons edge native sync: the eight 38 px component-drawer mini icon buttons now declare #171f26 borders and corner_radius = 10.0; template_icon_buttons.rs consumes declared radius and declared danger border values before fallback constants; workbench_projection.rs asserts the projected mini icon edge values
  - 2026-06-02 Workbench Icon Buttons edge native sync: rustfmt --edition 2021 --check over template_icon_buttons.rs, template_icon_buttons_tests.rs, and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; focused Cargo stayed deferred because an existing editor bridge Cargo/rustc lane was already compiling
  - 2026-06-03 Workbench Primary button color native sync: WorkbenchPrimaryButton declares background_color = "#29a4b8", border_color = "#1c8798", and visual_brightness = 1.0; template_buttons.rs consumes those declared colors before fallback Primary constants; workbench_projection.rs asserts the projected Primary style values
  - 2026-06-03 Workbench Primary button color native sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; focused Cargo stayed deferred because other Cargo/rustc lanes were active
  - 2026-06-03 Workbench Add Component button border native sync: WorkbenchAddComponent declares border_color = "#364047"; template_buttons.rs consumes the declared border before the Inspector-specific text/glyph override; workbench_projection.rs asserts the projected Add Component style value
  - 2026-06-03 Workbench Add Component button border native sync: rustfmt --edition 2021 --check over template_buttons.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_inspector_panel.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench Inputs bottom-control height native sync: WorkbenchInputDropdown and WorkbenchInputStepper declare fixed 30.5 heights; template_dropdowns.rs and template_fields.rs preserve half-pixel Workbench paint heights; workbench_projection.rs asserts both expanded leaf frames stay at 30.5
  - 2026-06-03 Workbench Inputs bottom-control height native sync: rustfmt --edition 2021 --check over template_dropdowns.rs, template_fields.rs, and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench Warning icon stroke-width native sync: WorkbenchStatusWarnings declares icon_stroke_width = 1.45; both retained projection paths map it into TemplatePaneNodeData.icon_stroke_width; template_status_controls.rs consumes it for the warning exclamation mark while preserving the 21 px icon slot; workbench_projection.rs and status painter tests assert the projected width and mark geometry
  - 2026-06-03 Workbench Warning icon stroke-width native sync: rustfmt --edition 2021 --check over template_nodes.rs, workbench_window_projection.rs, pane_component_projection/mod.rs, template_node_conversion.rs, template_status_controls.rs, workbench_projection.rs, template_assets.rs, component_showcase.rs, and pane_component_projection/tests.rs passed; Python tomllib declaration assertions for workbench_status_bar.zui passed; focused Cargo stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench Labs tabs background native sync: WorkbenchLabsTabs declares background_color = "#1c2226"; retained projection exposes it through button_style.element.background_color; template_segmented_controls.rs consumes declared idle tab-strip backgrounds while preserving hover/pressed/focused state priority; workbench_projection.rs and the segmented-control painter tests assert the projected color and painted pixel
  - 2026-06-03 Workbench Labs tabs background native sync: rustfmt --edition 2021 --check over template_segmented_controls.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; focused Cargo stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench Sliders thumb native sync: WorkbenchInputSlider, WorkbenchInputRangeSlider, and WorkbenchInputStepsSlider declare thumb_color = "#b7f1f8", thumb_outline_color = "#2ab1bc33", and thumb_halo_color = "#32d3de3d"; both retained projection paths map those aliases into icon_color, button_style.element.border_color, and state_layer_color; template_sliders.rs consumes them for native thumb fill, outline, and soft halo while preserving track/fill geometry
  - 2026-06-03 Workbench Sliders thumb native sync: rustfmt --edition 2021 --check over template_sliders.rs, workbench_window_projection.rs, pane_component_projection/mod.rs, pane_component_projection/tests.rs, and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; focused Cargo stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench Toggle tone native sync: editor_workbench_strict.v2.ui.toml now exposes idle workbench_toggle_track/border/thumb tokens and checked workbench_toggle_checked_track/border/thumb tokens; template_selection_controls.rs consumes declared toggle background, foreground, and border colors before fallback constants; workbench_projection.rs asserts the checked projected colors
  - 2026-06-03 Workbench Toggle tone native sync: rustfmt --edition 2021 --check over template_selection_controls.rs and workbench_projection.rs passed; Python tomllib declaration assertions for editor_workbench_strict.v2.ui.toml and workbench_component_drawer.zui passed; stale marker scan, git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active Cargo/rustc lanes were compiling
  - 2026-06-03 Workbench table-tail Modified-column native sync: WorkbenchTableTail declares fourth_cell_text_color = "#aab5ba"; the retained projection exposes it as TemplatePaneNodeData.value_color; template_table_rows.rs consumes the same tone for the tail fourth-cell fallback; workbench_projection.rs asserts the projected color
  - 2026-06-03 Workbench table-tail Modified-column native sync: rustfmt --edition 2021 --check over template_table_rows.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_component_drawer.zui passed; stale marker scan, targeted git diff --check, and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active Cargo/rustc lanes were compiling
  - 2026-06-02 Workbench tooltip tone follow-up: workbench_tooltip.zui, the drawer-mounted WorkbenchTooltipRoot, template_tooltips.rs fallback constants, pane_component_projection/tests.rs, and workbench_projection.rs now mirror the latest tooltip tones: border #252d32, secondary text #a8b3b8, and info mark #259ca7
  - 2026-06-02 Workbench tooltip tone follow-up: rustfmt --edition 2021 --check over the touched tooltip painter/projection test files passed; Python tomllib parsing/assertions for workbench_tooltip.zui and workbench_component_drawer.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused cargo test -p zircon_editor --lib template_tooltips --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 timed out after 184 seconds without a pass/fail result, and the post-timeout process scan showed only unrelated active Cargo lanes
  - 2026-06-02 Workbench Inspector Materials-row tone sync: WorkbenchMaterialRow now declares the counted Materials label, #9aa5ab label/count tone, #8f9aa0 select value, #20272c select border, and #13181b native field fill; template_inspector_rows.rs consumes declared resource label/count colors plus field background/border before fallback constants
  - 2026-06-02 Workbench Inspector Materials-row tone sync: rustfmt --edition 2021 --check over template_inspector_rows.rs and workbench_projection.rs passed; Python tomllib parse/declaration assertions for workbench_inspector_panel.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused Cargo was not started because unrelated render-main-chain and editor bridge Cargo lanes were already active
  - 2026-06-02 Workbench Inspector Lighting checkbox native sync: WorkbenchComponentPropertySlot03Row now declares layout_content_offset_x = 34.0; template_inspector_rows.rs consumes that declared offset for Receive Shadows checkbox placement before the old 28 px fallback; workbench_projection.rs asserts the projected Slot03 offset
  - 2026-06-02 Workbench Inspector Lighting checkbox native sync: rustfmt --edition 2021 --check over template_inspector_rows.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_inspector_panel.zui passed; tracked git diff --check and touched-file trailing whitespace scan passed; focused cargo test -p zircon_editor --lib template_inspector_rows was attempted twice but timed out and later exited without captured pass/fail output, so no Cargo pass is claimed
  - 2026-06-03 Workbench No Errors icon visual scale native sync: WorkbenchStatusErrors declares layout_icon_size = 12.04; template_status_controls.rs keeps the outer 14 px status icon layout slot for label placement and uses the declared size only for the centered success mark paint rect; workbench_projection.rs asserts the projected value
  - 2026-06-03 Workbench No Errors icon visual scale native sync: rustfmt --edition 2021 --check over template_status_controls.rs and workbench_projection.rs passed; Python tomllib declaration assertions for workbench_status_bar.zui passed; tracked git diff --check and touched-file trailing-whitespace scan passed; focused Cargo stayed deferred because active cargo/rustc lanes were compiling
  - 2026-06-03 Workbench Warning icon mark native sync: WorkbenchStatusWarnings declares icon_color/icon_stroke = #11181a; template_status_controls.rs consumes the declared warning foreground for the triangle exclamation mark while preserving the 21 px icon slot; workbench_projection.rs asserts the projected color
  - 2026-06-03 Workbench Checkboxes/Radios unchecked mark native sync: WorkbenchCheckboxOff and WorkbenchRadioOff declare background_color = #13191d and border_color = #374148; template_selection_controls.rs consumes those declared idle mark tones; workbench_projection.rs asserts both projected style colors
  - 2026-06-03 Workbench Cast Shadows select native sync: data_sync.rs applies background_color = #282e32, border_color = #343d43, and value_color = #b5c0c5 only to dynamic cast_shadows component-property rows and clears those style props for non-Cast rows; template_inspector_rows.rs consumes the declared nested-select field/value tones; workbench_projection.rs asserts both the dynamic style colors and clearing behavior
  - 2026-06-03 Workbench Ready status text native sync: WorkbenchStatusReady declares text_color = #8f9aa0; template_status_controls.rs consumes the projected Ready label tone without changing dot geometry or status-item spacing; workbench_projection.rs asserts the projected value color
  - cargo test -p zircon_editor --lib reference_workbench --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check and RUSTFLAGS=-Awarnings (2026-06-01: passed, 4 passed)
  - cargo test -p zircon_editor --lib componentized_workbench --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check and RUSTFLAGS=-Awarnings (2026-06-01: passed, 6 passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check and RUSTFLAGS=-Awarnings (2026-06-01: passed)
  - cargo test -p zircon_runtime_interface --lib ui_image_control_paint_elements_preserve_background_image_and_border_order --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check and RUSTFLAGS=-Awarnings (2026-06-01: passed, 1 passed)
  - cargo test -p zircon_runtime --lib surface_property_mutation_keeps_template_visibility_metadata_in_sync --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 latest: passed, 1 passed)
  - cargo test -p zircon_runtime --lib ui_v2_surface_property_mutation_updates_runtime_style_baseline_metadata --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 latest: passed, 1 passed)
  - cargo test -p zircon_runtime --lib ui_v2_surface_property_mutation_restyles_focused_pseudo_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01: passed, 1 passed)
  - cargo test -p zircon_editor --lib componentized_workbench --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 latest: passed, 22 passed)
  - cargo test -p zircon_editor --lib native_workbench_reference --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 latest: passed, 2 passed)
  - cargo test -p zircon_runtime_interface --lib render_contracts --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 latest: passed, 29 passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-check-b and RUSTFLAGS=-Awarnings (2026-06-01 latest: passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01: passed)
  - cargo test -p zircon_editor --lib componentized_workbench_window_template_bridge_syncs_scene_and_inspector_snapshot_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01: passed, 1 passed)
  - cargo test -p zircon_editor --lib componentized_workbench --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01: passed, 23 passed)
  - Python tomllib parse of zircon_editor/assets/ui/editor/components/workbench\shell\workbench_scene_tree_panel.zui (2026-06-01 latest: passed, 10 scene-tree row slots with 07-10 collapsed by default)
  - cargo test -p zircon_editor --lib componentized_workbench_window_template_bridge_syncs_scene_and_inspector_snapshot_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01 latest after ten scene-tree slots: passed, 1 passed)
  - cargo test -p zircon_editor --lib componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01: passed, 1 passed)
  - cargo test -p zircon_editor --lib componentized_workbench --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01 latest after virtual scene-tree rows: passed, 24 passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01 after template bridge virtual row helper extraction: passed)
  - cargo test -p zircon_editor --lib componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-virtual-rows-1435 and RUSTFLAGS=-Awarnings (2026-06-01 after template bridge virtual row helper extraction: passed, 1 passed)
  - cargo test -p zircon_editor --lib componentized_workbench --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-virtual-rows-1435 and RUSTFLAGS=-Awarnings (2026-06-01 after template bridge virtual row helper extraction: passed, 24 passed)
  - cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat declaration schema: passed)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01 after repeat declaration integration: passed)
  - cargo test -p zircon_runtime --lib ui_v2_repeat_declaration_is_preserved_in_compiled_surface_metadata --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01: timed out after 604 seconds during zircon_runtime test binary link; no compiler diagnostic returned)
  - cargo test -p zircon_editor --lib componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-decl-1603 and RUSTFLAGS=-Awarnings (2026-06-01: timed out after 605 seconds during zircon_runtime test binary link; no compiler diagnostic returned)
  - rustfmt --edition 2021 --check over the touched repeat/runtime/editor Rust files (2026-06-01 after repeat declaration integration: passed)
  - Python tomllib parse of zircon_editor/assets/ui/editor/components/workbench\shell\workbench_scene_tree_panel.zui (2026-06-01 after repeat declaration integration: passed, repeat table present and 10 authored scene-tree children)
  - cargo fmt -p zircon_editor --check (2026-06-01 latest after ten scene-tree slots: passed)
  - git diff --check over the touched bridge Rust, test, docs, and session note files (2026-06-01 latest after virtual scene-tree rows: no whitespace errors; Git reported LF-to-CRLF warnings for workbench_projection.rs, error.rs, and mod.rs)
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-reference-sync-1208 and RUSTFLAGS=-Awarnings (2026-06-01 latest after virtual scene-tree rows: passed)
  - rustfmt --edition 2021 --check over the touched workbench input-focus Rust files (2026-06-01 after WorkbenchField pointer-focus bridge: passed)
  - git diff --check -- zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs zircon_editor/src/ui/retained_host/callback_dispatch/workbench/pointer.rs zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection.rs (2026-06-01 after WorkbenchField pointer-focus bridge: passed; reported only LF-to-CRLF working-tree warning for the existing test file)
  - cargo test -p zircon_editor --lib componentized_workbench_pointer_focuses_input_fields_without_authored_binding --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-input-focus-1918 and RUSTFLAGS=-Awarnings (2026-06-01 after WorkbenchField pointer-focus bridge: timed out after 614 seconds while compiling; no compiler diagnostic returned)
  - cargo check -p zircon_editor --lib --tests --locked --jobs 1 --message-format short --color never with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after WorkbenchField pointer-focus bridge: blocked by unrelated active runtime plugin compile errors in zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_package_module_validation.rs, E0308, before editor workbench code was checked)
  - rustfmt --edition 2021 --check over the touched hover-feedback bridge and test Rust files (2026-06-01 after componentized hover bridge: passed)
  - git diff --check over the touched hover-feedback bridge, theme, test, docs, and session-note files (2026-06-01 after componentized hover bridge: passed; Git reported LF-to-CRLF working-tree warnings for template_bridge/mod.rs and template_bridge/support.rs)
  - Python tomllib parse of zircon_editor/assets/ui/theme/editor_workbench_strict.v2.ui.toml (2026-06-01 after componentized hover bridge: passed, 138 selectors and required hover selectors present)
  - cargo test -p zircon_editor --lib componentized_workbench_pointer_hover_updates_icon_button_preview_without_authored_binding --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-hover-2005 and RUSTFLAGS=-Awarnings (2026-06-01 after componentized hover bridge: timed out after 604 seconds while compiling; no compiler diagnostic returned)
  - rustfmt --edition 2021 --check over the touched hover/press-feedback bridge and test Rust files (2026-06-01 after componentized press bridge: passed)
  - Python tomllib parse of zircon_editor/assets/ui/theme/editor_workbench_strict.v2.ui.toml (2026-06-01 after componentized press bridge: passed, 150 selectors and required hover/pressed selectors present)
  - git diff --check over the touched hover/press-feedback bridge, theme, and test files (2026-06-01 after componentized press bridge: passed; Git reported LF-to-CRLF working-tree warnings for template_bridge/mod.rs, template_bridge/support.rs, and workbench_projection.rs)
  - cargo test for the new pressed regression was not rerun after the press bridge because the previous focused editor regression had already timed out after 604 seconds and concurrent workspace/Hub Cargo validation was still compiling in other active sessions; this leaves the compile/test gate open for the milestone testing stage.
  - rustfmt --edition 2021 --check over the touched componentized workbench pointer-feedback Rust files (2026-06-01 after slider pointer feedback: passed)
  - Python tomllib parse of zircon_editor/assets/ui/theme/editor_workbench_strict.v2.ui.toml (2026-06-01 after slider pointer feedback: passed, 153 selectors and slider hover/pressed selectors present)
  - git diff --check over the touched slider pointer-feedback bridge, theme, docs, and session-note files (2026-06-01 after slider pointer feedback: passed; Git reported LF-to-CRLF working-tree warnings for support.rs and workbench/mod.rs)
  - trailing-whitespace scan over the touched slider pointer-feedback Rust, theme, docs, and session-note files (2026-06-01 after slider pointer feedback: passed)
  - cargo test for componentized_workbench_pointer_drag_updates_slider_value_without_authored_binding was deferred on 2026-06-01 because unrelated cargo/rustc processes from other active sessions were still compiling; this leaves the focused compile/test gate open for the milestone testing stage.
  - rustfmt --edition 2021 --check over the touched workbench host-contract projection Rust files (2026-06-01 after structured dropdown/menu projection: passed)
  - Python tomllib parse of zircon_editor/assets/ui/editor/components/workbench\shell\workbench_component_drawer.zui (2026-06-01 after structured dropdown/menu projection: passed, confirmed dropdown option state metadata)
  - git diff --check over the touched structured dropdown/menu projection files (2026-06-01 after structured dropdown/menu projection: passed; Git reported LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the touched structured dropdown/menu projection files (2026-06-01 after structured dropdown/menu projection: passed)
  - cargo test -p zircon_editor --lib componentized_workbench_window_projection_exports_dropdown_and_popup_rows --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-repeat-validation-1755 and RUSTFLAGS=-Awarnings (2026-06-01 after structured dropdown/menu projection: timed out after 718 seconds while compiling dependencies; no compiler diagnostic returned, and matching cargo processes were stopped)
  - rustfmt --edition 2021 --check over the touched Workbench dropdown-selection Rust files (2026-06-01 after dropdown option selection bridge: passed)
  - Python tomllib parse of zircon_editor/assets/ui/editor/components/workbench\shell\workbench_component_drawer.zui (2026-06-01 after dropdown option selection bridge: passed, confirmed ComponentLab/InputDropdownSelect Change event plus dropdown option state metadata)
  - git diff --check over the touched dropdown-selection Rust/ZUI files (2026-06-01 after dropdown option selection bridge: passed; Git reported LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the touched dropdown-selection Rust/ZUI files (2026-06-01 after dropdown option selection bridge: passed)
  - focused Cargo for componentized_workbench_dropdown_option_selection_updates_value_and_projection deferred on 2026-06-01 because unrelated cargo/rustc processes from another active validation lane were still compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over the touched Workbench popup-menu selection Rust files (2026-06-01 after popup menu item selection bridge: passed)
  - git diff --check over the touched popup-menu selection Rust/docs files (2026-06-01 after popup menu item selection bridge: passed; Git reported LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the touched popup-menu selection Rust/docs files (2026-06-01 after popup menu item selection bridge: passed)
  - focused Cargo for componentized_workbench_popup_menu_item_selection_updates_value_and_projection deferred on 2026-06-01 because unrelated cargo/rustc processes from another active validation lane were still compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over the touched Workbench popup-row hit-test Rust files (2026-06-01 after popup row hit-test bridge: passed)
  - git diff --check over the touched popup-row hit-test Rust/docs files (2026-06-01 after popup row hit-test bridge: passed; Git reported LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the touched popup-row hit-test Rust/docs files (2026-06-01 after popup row hit-test bridge: passed)
  - focused Cargo for workbench_hit_test_routes_open_dropdown_option_rows and workbench_hit_test_routes_open_popup_menu_rows deferred on 2026-06-01 because unrelated cargo/rustc processes from another active validation lane were still compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over the touched popup-row painter, layout, and hit-test Rust files (2026-06-01 after popup-row painter bridge and painter module split: passed)
  - git diff --check over the tracked touched popup-row painter and hit-test Rust files (2026-06-01 after popup-row painter/layout bridge: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over all touched popup-row painter, layout, hit-test, docs, and session-note files including new modules (2026-06-01 after popup-row painter/layout bridge: passed)
  - focused Cargo for template_nodes_paint_open_dropdown_option_rows_below_control, template_nodes_paint_open_popup_menu_rows_inside_menu_frame, workbench_hit_test_routes_open_dropdown_option_rows, and workbench_hit_test_routes_open_popup_menu_rows deferred on 2026-06-01 because unrelated cargo/rustc processes from another active validation lane were still compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over the touched popup-row hover, painter, layout, and hit-test Rust files (2026-06-01 after popup-row native hover bridge: passed)
  - git diff --check over the tracked touched popup-row hover, painter, layout, and hit-test Rust files (2026-06-01 after popup-row native hover bridge: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over all touched popup-row hover, painter, layout, hit-test, docs, and session-note files including new modules (2026-06-01 after popup-row native hover bridge: passed)
  - focused Cargo for native_workbench_dropdown_option_row_hover_updates_structured_row_state and native_workbench_popup_menu_row_hover_updates_structured_row_state deferred on 2026-06-01 because unrelated cargo/rustc processes from another active validation lane were still compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over native_keyboard.rs, host_contract/mod.rs, host_contract/window.rs, and native_workbench_reference.rs (2026-06-01 after popup-row keyboard navigation bridge: passed)
  - git diff --check over the tracked popup-row keyboard navigation files (2026-06-01 after popup-row keyboard navigation bridge: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the popup-row keyboard navigation files (2026-06-01 after popup-row keyboard navigation bridge: passed)
  - focused Cargo for native_workbench_dropdown_keyboard_moves_row_hover_and_enter_dispatches_option and native_workbench_popup_menu_keyboard_moves_row_hover_and_enter_dispatches_menu_item deferred on 2026-06-01 because unrelated cargo/rustc processes from another active validation lane were still compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over the touched popup-cancel Rust files (2026-06-02 after Escape popup cancel bridge: passed)
  - git diff --check over the tracked popup-cancel Rust/docs files (2026-06-02 after Escape popup cancel bridge: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over all touched popup-cancel Rust/docs/session files including new modules (2026-06-02 after Escape popup cancel bridge: passed)
  - focused Cargo for componentized_workbench_popup_cancel_closes_dropdown_without_value_dispatch, componentized_workbench_popup_cancel_closes_menu_without_selecting_item, native_workbench_dropdown_escape_dispatches_popup_cancel, and native_workbench_popup_menu_escape_dispatches_popup_cancel deferred on 2026-06-02 because unrelated cargo/rustc processes from another active validation lane were still compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over native_popup_dismiss.rs, host_contract/mod.rs, native_pointer.rs, and native_workbench_reference.rs (2026-06-02 after outside-click popup dismissal: passed)
  - git diff --check over the tracked outside-click popup-dismiss Rust/docs files (2026-06-02 after outside-click popup dismissal: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over all touched outside-click popup-dismiss Rust/docs/session files including new modules (2026-06-02 after outside-click popup dismissal: passed)
  - focused Cargo for native_workbench_dropdown_option_primary_press_keeps_selection_path, native_workbench_popup_menu_item_primary_press_keeps_menu_selection_path, native_workbench_dropdown_outside_primary_press_dispatches_popup_cancel, and native_workbench_popup_menu_outside_primary_press_dispatches_popup_cancel was attempted with cargo test -p zircon_editor --lib primary_press --locked --jobs 1 --message-format short --color never -- --nocapture and CARGO_TARGET_DIR=D:\cargo-targets\zircon-editor-workbench-popup-dismiss; the command timed out after 1204 seconds without a compiler diagnostic, and the owned cargo/rustc child processes were stopped while unrelated workspace/runtime validation lanes remained active
  - rustfmt --edition 2021 --check over template_popup_layout.rs, surface_hit_test/template_node.rs, painter/template_popup_rows.rs, painter/template_nodes.rs, native_keyboard.rs, and native_popup_dismiss.rs (2026-06-02 after bounded dropdown popup geometry: passed)
  - git diff --check over the tracked bounded dropdown popup geometry Rust/docs files (2026-06-02 after bounded dropdown popup geometry: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over all touched bounded dropdown popup geometry Rust/docs/session files (2026-06-02 after bounded dropdown popup geometry: passed)
  - focused Cargo for dropdown_option_popup_frame_within_opens_above_when_below_overflows, workbench_hit_test_routes_dropdown_option_rows_above_control_when_bottom_clipped, and template_nodes_paint_open_dropdown_option_rows_above_control_when_below_clipped was deferred on 2026-06-02 because an unrelated workspace Cargo validation lane was still running; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over workbench_popup_geometry.rs, host_contract/mod.rs, native_keyboard.rs, native_popup_dismiss.rs, surface_hit_test/template_node.rs, and native_workbench_reference.rs (2026-06-02 after shared Workbench popup geometry and disabled/separator native press coverage: passed)
  - git diff --check over the tracked shared-popup-geometry Rust/docs/session files (2026-06-02 after shared Workbench popup geometry: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over all touched shared-popup-geometry Rust/docs/session files including the new geometry module (2026-06-02 after shared Workbench popup geometry: passed)
  - focused Cargo for workbench_hit_test_blocks_popup_menu_separator_row, native_workbench_disabled_dropdown_option_primary_press_is_ignored_without_cancel, and native_workbench_popup_menu_separator_primary_press_is_ignored_without_cancel was deferred on 2026-06-02 because unrelated workspace/runtime Cargo and rustc lanes remained active; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over native_workbench_reference.rs (2026-06-02 after native Workbench text edit/commit lane: passed)
  - git diff --check over workbench_component_drawer.zui, native_workbench_reference.rs, docs, and session note (2026-06-02 after native Workbench text edit/commit lane: passed; Git reported only an LF-to-CRLF working-tree warning for the existing Rust test file)
  - Python tomllib parse of zircon_editor/assets/ui/editor/components/workbench\shell\workbench_component_drawer.zui (2026-06-02 after native Workbench text edit/commit lane: passed)
  - trailing-whitespace scan over workbench_component_drawer.zui, native_workbench_reference.rs, docs, and session note (2026-06-02 after native Workbench text edit/commit lane: passed)
  - focused Cargo for native_workbench_text_input_focuses_edits_and_commits_from_keyboard was deferred on 2026-06-02 because unrelated workspace/runtime Cargo and rustc lanes remained active; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over the Inspector property edit bridge Rust files (2026-06-02 after local edit/commit state bridge: passed)
  - git diff --check over the tracked Inspector property edit Rust/docs/session files (2026-06-02 after local edit/commit state bridge: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the Inspector property edit Rust/docs/session files, including new untracked test and bridge modules (2026-06-02: passed)
  - focused Cargo for componentized_workbench_inspector_property_edit_updates_row_preview deferred on 2026-06-02 because unrelated workspace/runtime Cargo and rustc lanes remained active; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over template_node_labels.rs, painter/mod.rs, and template_nodes.rs (2026-06-02 after PropertyRow label/value painter extraction: passed)
  - git diff --check over the tracked PropertyRow label/value painter Rust/docs/session files (2026-06-02 after PropertyRow label/value painter extraction: passed; Git reported LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the PropertyRow label/value painter Rust/docs/session files, including the new untracked label module: passed on 2026-06-02
  - focused Cargo for property_row_label_keeps_label_and_value_visible was deferred on 2026-06-02 because unrelated workspace/runtime Cargo and rustc lanes remained active; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over template_property_rows.rs, painter/mod.rs, and template_nodes.rs (2026-06-02 after native PropertyRow axis/value field painting: passed)
  - git diff --check over the tracked native PropertyRow field-painter Rust/docs/session files (2026-06-02 after native PropertyRow axis/value field painting: passed; Git reported LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the native PropertyRow field-painter Rust/docs/session files, including the new untracked painter module: passed on 2026-06-02
  - focused Cargo for property_axis_values_group_units_with_axis_value deferred on 2026-06-02 because unrelated workspace/runtime Cargo and rustc lanes remained active; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over data_sync.rs, workbench_projection.rs, reference_surface.rs, and the retained painter modules (2026-06-02 after declarative Transform axis field primitive: passed)
  - Python tomllib parse of workbench_axis_value_field.zui, workbench_inspector_panel.zui, and editor_workbench_strict.v2.ui.toml (2026-06-02 after declarative Transform axis field primitive: passed)
  - git diff --check over the tracked declarative Transform axis field Rust/ZUI/theme/docs/session files (2026-06-02: passed; Git reported LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the declarative Transform axis field Rust/ZUI/theme/docs/session files, including the new untracked axis field component: passed on 2026-06-02
  - focused Cargo for reference_workbench_componentized_window_surface_matches_reference_chrome_metrics and componentized_workbench_window_template_bridge_syncs_scene_and_inspector_snapshot_state deferred on 2026-06-02 because unrelated workspace/runtime Cargo and rustc lanes remained active; no new compiler diagnostic was produced by this slice
  - rustfmt --check over transform_edit.rs, Workbench callback dispatch, and focused Workbench Inspector tests (2026-06-02 after Transform axis edit bridge: passed)
  - Python tomllib parse of workbench_axis_value_field.zui, workbench_inspector_panel.zui, and editor_workbench_strict.v2.ui.toml (2026-06-02 after Transform axis edit bridge: passed)
  - git diff --check over the Transform axis edit Rust/ZUI/test files (2026-06-02: passed; Git reported only LF-to-CRLF working-tree warnings for existing Rust files)
  - trailing-whitespace scan over the Transform axis edit Rust/ZUI/test files (2026-06-02: passed)
  - rustfmt --edition 2021 --check over component_property_rows.rs, data_sync.rs, property_edit.rs, workbench template bindings, and focused Workbench Inspector tests (2026-06-02 after Inspector component property row pool: passed)
  - Python tomllib parse of workbench_axis_value_field.zui, workbench_inspector_panel.zui, and editor_workbench_strict.v2.ui.toml (2026-06-02 after Inspector component property row pool: passed)
  - git diff --check over the Inspector component property row-pool Rust/ZUI/test files (2026-06-02: passed; Git reported only LF-to-CRLF working-tree warnings for existing tracked Rust files)
  - focused Cargo for componentized_workbench_window_template_bridge_syncs_scene_and_inspector_snapshot_state and componentized_workbench_inspector_property_edit_updates_row_preview remains deferred on 2026-06-02 because active zircon_editor and zircon_hub Cargo/rustc validation lanes are already compiling; no new compiler diagnostic has been produced by this slice
  - cargo test -p zircon_editor --lib workbench_inspector_property_edit --locked --jobs 1 --message-format short --color never -- --nocapture with CARGO_TARGET_DIR=E:\cargo-targets\zircon-editor-workbench-reference (2026-06-02: timed out after 904 seconds while compiling; the orphaned matching Cargo/rustc processes were stopped and no compiler diagnostic or test result was produced; further Cargo remains deferred while other active workspace validation lanes are compiling)
  - rustfmt --edition 2021 --check over template_selection_controls.rs and workbench_projection.rs (2026-06-02 after Checkboxes & Radios native tone/gap sync: passed)
  - Python tomllib parse of workbench_checkbox.zui, workbench_radio.zui, workbench_component_drawer.zui, and editor_workbench_strict.v2.ui.toml (2026-06-02 after Checkboxes & Radios native tone/gap sync: passed)
  - git diff --check over the touched selection-control Rust/ZUI/theme/docs files (2026-06-02 after Checkboxes & Radios native tone/gap sync: passed; Git reported only LF-to-CRLF working-tree warnings for docs/editor-and-tooling/editor-workbench-shell.md and the existing workbench_projection.rs test file)
  - focused Cargo for the selection-control slice stayed deferred on 2026-06-02 because active cargo/rustc lanes from other sessions were still compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over template_sliders.rs, workbench_window_projection.rs, pane_component_projection/mod.rs, and workbench_projection.rs (2026-06-02 after Slider row/proportion native sync: passed)
  - Python tomllib parse plus slider declaration assertions for workbench_component_drawer.zui, workbench_slider.zui, and editor_workbench_strict.v2.ui.toml, including Range range_min and Steps step_tick_count (2026-06-02 after Slider row/proportion native sync: passed)
  - stale-doc marker scan, git diff --check, and trailing-whitespace scan over the Slider row/proportion Rust/ZUI/docs/session files (2026-06-02: passed; Git reported only LF-to-CRLF working-tree warnings for existing tracked files)
  - focused cargo test -p zircon_editor --lib template_sliders --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 for the Slider row/proportion native sync timed out after 304 seconds on 2026-06-02 without compiler diagnostics; no cargo/rustc processes remained afterward
  - rustfmt --edition 2021 --check over workbench_projection.rs (2026-06-02 after Labs structural ownership sync: passed)
  - Python tomllib parse of workbench_component_drawer.zui (2026-06-02 after Labs structural ownership sync: passed, confirmed input_segmented and toggle_on are Labs children)
  - tracked git diff --check over tracked touched files, plus trailing-whitespace scan over the Labs structural ownership ZUI/test/docs/session files (2026-06-02: passed; Git reported only LF-to-CRLF working-tree warnings for existing tracked files)
  - focused Cargo for the Labs structural ownership sync was deferred on 2026-06-02 because active cargo/rustc lanes were compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over workbench_projection.rs (2026-06-02 after Sliders structural column sync: passed)
  - Python tomllib parse of workbench_component_drawer.zui (2026-06-02 after Sliders structural column sync: passed, confirmed WorkbenchComponentSliders owns the three slider rows)
  - tracked git diff --check over tracked touched files, plus trailing-whitespace scan over the Sliders structural column ZUI/test/docs/session files (2026-06-02: passed; Git reported only LF-to-CRLF working-tree warnings for existing tracked files)
  - focused Cargo for the Sliders structural column sync was deferred on 2026-06-02 because active cargo/rustc lanes were compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over workbench_projection.rs (2026-06-02 after component drawer top/lower row split: passed)
  - Python tomllib parse of workbench_component_drawer.zui (2026-06-02 after component drawer top/lower row split: passed, confirmed top-row/lower-row, List/Menu, and Table ownership)
  - tracked git diff --check over tracked touched files, plus trailing-whitespace scan over the component drawer row-split ZUI/test/docs/session files (2026-06-02: passed; Git reported only LF-to-CRLF working-tree warnings for existing tracked files)
  - focused Cargo for the component drawer row split was deferred on 2026-06-02 because active cargo/rustc lanes were compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over workbench_projection.rs (2026-06-02 after component drawer feedback/toast layout sync: passed)
  - Python tomllib parse of workbench_component_drawer.zui (2026-06-02 after component drawer feedback/toast layout sync: passed, confirmed WorkbenchFeedbackAlerts, WorkbenchTooltipRoot, WorkbenchFeedbackToastColumn, and a single uncontrolled feedback_toast instance)
  - tracked git diff --check over tracked touched files, plus trailing-whitespace scan over the feedback/toast layout ZUI/test/docs/session files (2026-06-02: passed; Git reported only LF-to-CRLF working-tree warnings for existing tracked files)
  - focused Cargo for the component drawer feedback/toast layout sync was deferred on 2026-06-02 because active cargo/rustc lanes were compiling; no new compiler diagnostic was produced by this slice
  - rustfmt --edition 2021 --check over menu action, retained adapter, componentized workbench projection, hit-test, and showcase action input Rust files (2026-06-05 action-id normalization and projection support: passed)
  - git diff --check over the touched action-id normalization, retained projection, runtime diagnostics, ECS query, showcase asset, and docs files (2026-06-05: passed with only Windows LF-to-CRLF working-tree warnings)
  - Get-ChildItem/Select-String guard for component_lab.button_dropdown_option.select and component_lab.input_dropdown_option.select under zircon_editor/src and zircon_editor/assets (2026-06-05: passed, no legacy dropdown-option action ids remain)
  - cargo test -p zircon_editor --lib componentized_workbench_dropdown_option_selection_updates_value_and_projection --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 after explicit route action-id projection: passed, 1 passed)
  - cargo test -p zircon_editor --lib runtime_component_projection_preserves_primary_click_binding_id --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 after primary-click fallback change: passed, 1 passed)
  - cargo test -p zircon_editor --lib workbench_hit_test_routes_componentized_text_input_center --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 after route action-id projection: passed, 1 passed)
  - cargo test -p zircon_editor --lib workbench_hit_test_routes_open_popup_menu_rows --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 after popup menu-row action normalization: passed, 1 passed)
  - cargo test -p zircon_editor --lib apply_presentation_carries_componentized_workbench_window_nodes_separately --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 after raw binding-id fallback restore: passed, 1 passed)
  - cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never -- --test-threads=1 (2026-06-05 after workbench/showcase fixes: attempted twice; first failed before editor tests on runtime ECS query_many_iter.rs borrow error, fixed by cached read-only many local-field borrow split; second cargo/rustc exited -1 without a Rust diagnostic while concurrent editor/runtime cargo lanes were compiling)
doc_type: module-detail
---

# Workbench Reference Surface

## Purpose

The workbench reference surface is the first engine-side slice toward the target editor visual shown in `docs/ui-and-layout/workbench.png`. It is not a static HTML mock and it is now consumed through the retained host workbench-window projection path. It builds a real `UiSurface` from runtime UI tree primitives so the editor direction can be validated through the same layout, render extraction, hit testing, pointer dispatch, and host presentation contracts that production editor windows use.

This module gives the editor a stable visual baseline for the large shell proportions: top toolbar, activity rail, scene hierarchy, viewport, inspector, component gallery, and status bar. The proportions intentionally match the 1672 x 941 reference frame closely enough to keep later screenshot comparisons meaningful while leaving room to refine art direction without touching low-level runtime contracts. The componentized `workbench_window.v2.ui.toml` template is kept aligned to the same chrome metrics so the declarative component route and the direct runtime reference route do not drift. `template_surface.rs` exposes that declarative route as a reusable editor runtime surface entry point instead of leaving it as a test-only file load.

## Related Files

`metrics.rs` owns target dimensions and fixed chrome proportions. `tokens.rs` owns the color palette used by this reference layer. `ids.rs` assigns stable node IDs for the major regions and interactive controls that tests and diagnostic tools need to address directly.

`surface.rs` exposes `EditorWorkbenchReferenceSurface` and the public constructor. The folder-backed `builder` module owns the actual tree creation. `builder/mod.rs` assembles the root, top bar, upper shell, and status bar. `builder/panels.rs` assembles the activity rail, hierarchy, viewport, inspector, and component gallery. `builder/nodes.rs` centralizes node construction, style metadata, input policy, widget contracts, and sizing helpers.

`template_surface.rs` exposes `EditorWorkbenchTemplateSurface`, `EditorWorkbenchTemplateFrames`, and `build_editor_workbench_template_surface(...)`. It uses the editor template runtime's built-in `editor.window.workbench` document, computes layout at the reference metrics, extracts the host-visible wrapper control frames that retained/native presentation code can address, and builds a retained host projection from the same arranged surface.

`workbench_window.v2.ui.toml` is the declarative assembly target for the same visual language. It imports the `workbench_*.zui` primitive components and `editor_workbench_strict.v2.ui.toml` theme tokens, then composes them into the top toolbar, scene tree, viewport chrome, inspector, component drawer, and status bar.

`workbench_axis_value_field.zui` is the low-level Transform value primitive for Inspector vector rows. It wraps an `InputField` with compact height, fixed width, Workbench field chrome, and interactive/focusable input metadata so Position/Rotation/Scale can be assembled from real Taffy children rather than a single string-valued property row.

`workbench_checkbox.zui` and `workbench_radio.zui` own the component-lab selection-control baseline. Their declaration now carries the accepted 16 px mark size, 9 px mark-to-label gap, muted `#828c93` label tone, and the 7 px `#43d8e2` radio dot. `editor_workbench_strict.v2.ui.toml` supplies the matching selected mark shell tokens, while `WorkbenchCheckboxOff` and `WorkbenchRadioOff` declare the latest audited unchecked fill/border directly as `#13191d` / `#374148` so idle showcase marks can diverge from the reusable root fallback without moving rows. `template_selection_controls.rs` consumes declared label colors and resolved strict-theme or leaf background/border colors before falling back to local Workbench constants, so native rendering follows the same authored style path as the HTML/CSS reference instead of keeping a separate hard-coded selection palette.

`workbench_component_drawer.zui` also owns the component-lab button-row baseline. The first row declares Primary/Secondary label size as 12.22 px with their accepted offsets; Primary now declares the accepted `#29a4b8` native surface, `#1c8798` border, and neutral `visual_brightness = 1.0`, while Secondary keeps its `#1a1f23` surface and 1.01 brightness. The second row declares the 1 px x offset, Tertiary `#171c20` surface, `#879299` content tone, `#252e35` Tertiary/Outline border, and 9 px radius; the third row declares `WorkbenchButtonIcon` `#20262a` surface, `#303840` border, `#7f8a91` content tone, and 9 px radius, while `WorkbenchButtonDelete` declares `#d05a50` content tone and the same radius; Disabled declares `#2d3337` background, `#343d44` border, `#747f86` label, and `0.72` opacity; and the Button Dropdown declares `#7f8a91` label, `#67737a` chevron, and `#1f272e` border. The retained projection maps `arrow_color` into native icon color, `template_buttons.rs` consumes declared Primary/Secondary surfaces, disabled style plus style opacity, foreground colors, and button radius, and `template_dropdowns.rs` consumes declared background, border, value, and chevron colors before fallback defaults.

The component-drawer Inputs bottom row is also declaration-owned. `WorkbenchInputDropdown` and `WorkbenchInputStepper` keep the accepted `layout_offset_x = -4.0`, `layout_offset_y = 8.0`, 95 px dropdown width, 67 px stepper width, and 8 px row gap, and now declare the audited `30.5` visual height directly on the leaf controls. `template_dropdowns.rs` and `template_fields.rs` preserve half-pixel Workbench control heights while still aligning x/y and width to pixels, so the native painter consumes the HTML/CSS `30.5px` height instead of rounding it away. The main text/focused/disabled field rows, colors, fill tones, and stepper divider/arrow styling stay unchanged.

The component-drawer `Icon Buttons` lane also owns the large mini icon-button edge. All eight 38 px buttons declare `border_color = "#171f26"` and `corner_radius = 10.0`; the seven non-danger buttons keep `#272d31` fill and `#98a3a8` glyph tone, while `WorkbenchMiniDelete` keeps the danger fill/glyph branch and only shares the dark edge. `template_icon_buttons.rs` consumes the declared radius and lets declared danger borders override the red fallback edge so the native painter now matches the HTML/CSS edge pass.

`workbench_window_template_bindings.rs` registers the event IDs emitted by the componentized workbench template. Tool buttons map to viewport commands, hierarchy selection maps to a selection command, drawer/rail controls map to dock commands where a stable dock target exists, and visual-only lab controls map to editor operations or menu actions so retained projection has route metadata instead of anonymous clickable nodes.

`workbench_window_projection.rs` converts the retained workbench surface projection into the native host `TemplatePaneNodeData` contract. The component drawer's `WorkbenchInputDropdown` now declares selected, special, focused, hovered, and disabled option metadata, and the workbench-window projection reuses the same structured option and menu parsers used by pane/component projection. That means dropdowns and popup menus expose both the raw string lists and typed `structured_options` / `structured_menu_items` rows, including separator, disabled, focused, hovered, and selected state. Native rendering can therefore consume explicit row state instead of reparsing display strings.

The projection keeps authored binding identity and dispatch action identity separate. `retained_adapter.rs` carries each route's explicit action id alongside the binding id. `workbench_window_projection.rs` prefers that explicit action id for native route callbacks, but falls back to the raw binding id when no route action exists, so legacy tests and host contracts still see `ComponentLab/...` binding ids on primary clicks. Generic pane/component projection follows the same rule: it only emits a primary-click action id when the template binding authored one, rather than deriving a snake-case command id from every binding id. The componentized Workbench dropdown routes are hard-cut to `component_lab.input_dropdown.select` and `component_lab.button_dropdown.select`; the older `*_dropdown_option.select` ids are not kept as compatibility aliases.

The same projection path now carries component-system metadata into the editor host contract. `TemplatePaneNodeData.component_category` and `TemplatePaneNodeData.component_layout_role` are resolved through `component_contract_metadata.rs`, which combines the runtime editor-showcase and Material foundation descriptor registries before falling back by component role. This gives native painter and diagnostic code stable tokens such as `input`, `selection`, `collection`, `container`, `leaf`, `flex`, `grid`, `popup`, and `virtual-list`, so Button/Input/List/Table/Drawer families can be grouped from a shared contract rather than from one-off control-id paint rules.

Dropdown selection now has a Workbench-specific host bridge path. The `WorkbenchInputDropdown` template exposes `ComponentLab/InputDropdownSelect` as a Change route. When a structured option callback selects an enabled option, the bridge validates the option id against the authored `options` and `disabled_options`, writes the selected id into `value` and `value_text`, moves `special_options` to the selected id, clears transient option focus/hover/press arrays, closes `popup_open`, and refreshes the retained projection. Disabled or unknown option ids are swallowed as no-ops so native option rows cannot mutate the preview state accidentally.

Popup-menu row selection uses the same preview-only boundary. When `dispatch_pane_surface_control_clicked(...)` receives a Workbench control id plus a non-binding action id, it first asks the componentized Workbench bridge whether the action matches an authored `menu_items` row. A matched enabled row updates the popup menu's `value` and `value_text`, removes transient `focused` / `hovered` / `pressed` row flags from the authored menu list, closes the popup, refreshes projection, and requests paint-only invalidation. It deliberately does not invoke real editor menu actions such as Delete, because the component drawer menu is a visual interaction sample rather than an editor command surface.

Native hit testing now gives those structured rows real pointer targets. `surface_hit_test/template_node.rs` checks open popup rows before the normal template-node surface hit-test: dropdown options synthesize `workbench_option` hits below the dropdown frame and carry the Change binding id plus option id; popup menu rows synthesize `workbench_menu_item` hits across the menu frame and carry a normalized row action id. Hand-authored sample rows such as `Delete` are normalized to `menu.item.delete`, while already canonical `menu.item.*` rows pass through unchanged. Points inside an open popup but on a disabled option, separator row, or other non-activatable popup area are blocked from falling through to the popup parent or underlying controls. `native_pointer.rs` routes `workbench_option` through the structured-option callback lane, which the retained host already redirects to the active Workbench bridge. Menu row hits continue through `surface_control_clicked`, where the Workbench bridge handles them as preview-only row selection before any generic menu command fallback runs.

Native painting now renders the same structured rows instead of leaving them as invisible hit zones. `template_nodes.rs` remains the orchestration boundary for retained template-node painting, while `template_popup_rows.rs` draws open dropdown options below the control frame and popup-menu rows inside the menu frame. Selected and special rows receive the selected background plus a narrow accent marker; hovered, focused, and pressed rows share the hover surface; disabled rows keep disabled text and no active row fill; separators and shortcuts are painted from structured menu row metadata. Both painter and hit-test consume `template_popup_layout.rs` for dropdown option and menu row frames, so visible rows and pointer targets are derived from the same geometry.

Dropdown popup geometry now has a bounded variant in `template_popup_layout.rs`. The default geometry still opens below the control, but the bounded layout opens above the control when the below position would cross the workbench or pane bounds and the popup fully fits above. It also clamps the popup's right edge to the available bounds. `template_popup_rows.rs`, `surface_hit_test/template_node.rs`, `native_keyboard.rs`, and `native_popup_dismiss.rs` all use that bounded dropdown geometry, so painting, pointer hit testing, keyboard row focus, Escape/outside cancel damage, and popup containment share the same responsive placement. Shared frame conversion, native-window popup bounds, point containment, and damage-frame union logic live in `workbench_popup_geometry.rs`, keeping hit-test, keyboard, and outside-dismiss behavior on one geometry contract.

The retained painter also splits surface style resolution into `template_style.rs`. That keeps disabled-state detection, button/material color selection, border width/radius resolution, and elevation shadow geometry shared by `template_nodes.rs` and `material_state_layer.rs` without pushing more style logic into the already large node painter.

Native pointer hover now carries row identity for those synthetic popup rows. `HostPaneInteractionStateData` stores the hovered template control plus optional row dispatch kind, row action id, and row value text. During `UiHostWindow::get_host_presentation()`, `window.rs` applies that pointer-only state to the cloned presentation: dropdown option hover rewrites only the cloned `structured_options` hover/focus/press flags for the currently hovered option id, and popup-menu hover rewrites the cloned `structured_menu_items` transient flags for the current menu action. The underlying componentized Workbench surface remains unchanged until a click/select action occurs, so hover feedback is transient and clears when the pointer leaves.

`BuiltinWorkbenchWindowTemplateSurfaceBridge` is the retained-host bridge for the componentized workbench window. It wraps `EditorWorkbenchTemplateSurface`, keeps the editor template runtime alive for recompute, exposes the same `surface`, `frames`, `host_projection`, `control_frame(...)`, and `binding_for_control(...)` style used by other retained template bridges, and is included in the startup built-in template runtime document set.

`pointer_feedback.rs` owns the bridge-local pointer preview lane for the componentized workbench. It keeps hover, pressed, text-input focus, and range-slider feedback close to the retained surface route, while `componentized_window.rs` stays focused on constructing, refreshing, and exposing the workbench bridge.

`data_sync.rs` owns the first live-data binding step for that bridge. It maps `EditorChromeSnapshot::scene_entries` into the scene tree: the first ten rows use authored `WorkbenchTreeRow` slots, and entries beyond that are projected through `scene_tree_rows.rs` as virtual rows cloned from the authored row prototype declared by `WorkbenchSceneTree.repeat`. The sync updates row labels, depth, indentation, expanded state, scene-node id, selection state, and hides unused authored slots. It also maps `EditorChromeSnapshot::inspector` into the inspector title, Transform rows, and a repeat-backed component property row pool. The first four property rows remain authored controls for stable screenshot proportions, while additional properties are reconciled by `component_property_rows.rs` as virtual rows cloned from `WorkbenchComponentPropertySlot04Row`. Missing rows have their property metadata cleared and collapse, so stale property names and values do not survive a later no-selection or no-component snapshot.

`UiV2Repeat` carries the first declaration-level repeat metadata through v2 asset parsing, arena compilation, component instancing, and surface metadata projection. The workbench scene tree uses `repeat = { kind = "virtual_rows", prototype = "WorkbenchSceneSlot10Item", virtual_control_prefix = "WorkbenchSceneVirtualItem", authored_count = 10, node_path_namespace = "v2" }` so dynamic-row policy starts from the template asset instead of bridge-local constants.

`template_bridge/virtual_rows.rs` provides the reusable retained-template overflow row helper. It owns the shared mechanics for finding a parent/prototype control from repeat metadata, cloning a prototype row, assigning deterministic generated control IDs and node paths, detaching stale rows into `UiSurface`'s node pool, and reinserting pooled rows when the live count grows again.

`scene_tree_rows.rs` is the current workbench-specific dynamic-row bridge layered on that helper. It reconciles the row count before snapshot sync, creates `WorkbenchSceneVirtualItemNN` nodes for live hierarchy entries beyond the authored ten-row baseline, and supplies the scene-tree default metadata. Dynamic rows reuse an authored scene-row selection binding so the retained host projection resolves existing route metadata while each virtual row keeps its own control id and `scene_node_id`.

For state-response validation, the bridge can dispatch a control binding in tests and apply the visible state that a native host would expect to see: tool buttons, activity rail entries, panel tabs, component drawer tabs, checkboxes, radios, toggles, segmented controls, lists, and table rows update their `selected`, `checked`, `popup_open`, `value`, or `visibility` properties, then the dirty surface is rebuilt and projected again. `EditorWorkbenchTemplateSurface` remembers its latest layout size so a state-only refresh after resizing preserves the resized viewport frame instead of snapping back to the 1672 x 941 reference size.

The bridge intentionally applies component drawer visibility to every exact control-id node and restores legacy `visible=true` when showing a body that was previously collapsed. That keeps expanded component surfaces, template metadata, older visibility flags, and the v2 runtime-style baseline aligned during responsive layout refreshes.

Focused preview controls use the same runtime component-state channel as pointer focus. The bridge can still mutate authored-looking `focused` metadata for inspection, but `UiSurface::mutate_property(...)` mirrors retained pseudo-state keys such as `focused`, `hovered`, `active`, `popup_open`, and `selected` into `UiSurfaceComponentStateStore` so v2 runtime style refreshes keep the interactive state active instead of stripping it as stale metadata.

Componentized `WorkbenchField` controls now use the same low-level pointer route for live focus feedback even when the input has no authored click binding. `BuiltinWorkbenchWindowTemplateSurfaceBridge::route_pointer_event(...)` lets `UiSurface` choose the focused text-input owner on primary pointer down. When that route focused a text-input control and dirtied runtime component-state styling, `refresh_text_input_pointer_feedback(...)` rebuilds the dirty surface at the current shell size, regenerates retained projection from the same arranged tree, and the retained pointer dispatcher returns a paint-only host invalidation. The path is intentionally limited to text-input controls so button, tab, checkbox, and tool releases keep their existing binding-dispatch semantics while the input lane starts behaving like a real editable field.

The Workbench component drawer now gives its editable text field samples explicit Change and Submit routes. `WorkbenchInputText` and `WorkbenchInputFocused` project edit and commit action IDs into the retained host contract, so a native primary press can establish `HostTextInputFocusData`, keyboard/IME text can update the focus value through `surface_control_edited`, and Enter can dispatch the matching commit action while repainting only the focused field frame.

The Inspector component-property summary area now declares a row pool. `WorkbenchMeshRow`, `WorkbenchMaterialRow`, `WorkbenchComponentPropertySlot03Row`, and `WorkbenchComponentPropertySlot04Row` expose explicit Change and Submit routes for authored slots 01-04. `WorkbenchInspectorMesh.repeat` declares a `virtual_rows` overflow policy with `WorkbenchComponentPropertySlot04Row` as the prototype and `WorkbenchComponentPropertyVirtualRowNN` as the generated control id prefix. Snapshot sync writes the matched property field id, name, value kind, editable flag, and raw value into every visible row, authored or virtual.

`WorkbenchMaterialRow` also carries the latest native resource-row visual contract for the reference Inspector: the counted `Materials` label uses `#9aa5ab` for label and count, the select value uses `#8f9aa0`, the select border uses `#20272c`, and the native select field fill uses `#13181b` while the swatch stays on the fixed cyan material token. `template_inspector_rows.rs` consumes those declarations before fallback constants so the reference row no longer depends on Rust-only resource defaults.

`WorkbenchComponentPropertySlot03Row` carries the measured `Receive Shadows` checkbox spacing from the reference pass through `layout_content_offset_x = 34.0`. The value survives snapshot sync because property metadata updates only text/value/field fields, and `template_inspector_rows.rs` reads the positive declared offset only for recognized `Receive Shadows` checkbox rows. Rows without that declaration keep the old 28 px fallback, and `Cast Shadows` select geometry continues to use its separate 14 px nested-select inset.

`WorkbenchAddComponent` carries the latest Inspector button edge as a declarative `border_color = "#364047"` instead of a painter-only special case. The button painter still applies the accepted 1.5 px y offset plus Inspector text/glyph tones, but the border now flows through the normal resolved-button-style path and is asserted in retained projection.

`WorkbenchStatusErrors` carries the accepted browser `scale(0.86)` as `layout_icon_size = 12.04`. In the native status painter this is intentionally a visual paint size, not the text-layout icon slot: `status_signal_icon_rect(...)` still returns the 14 px status slot used to place the `No Errors` label, and `status_signal_icon_paint_rect(...)` centers the 12.04 px success mark inside it. This preserves the status item layout width and adjacent status rows.

`WorkbenchStatusWarnings` carries the softened warning exclamation mark as `icon_stroke_width = 1.45` alongside `icon_color`/`icon_stroke = #11181a`. The retained host contract stores this as `TemplatePaneNodeData.icon_stroke_width`; both projection paths map the `.zui` property, and `template_status_controls.rs` applies it only to the two internal mark segments so the 21 px warning icon slot, label gap, text tone, and fill stay layout-stable.

`property_edit.rs` handles Inspector property edit and commit routes as local Workbench preview state transitions. Authored rows resolve through their slot-specific binding ids; virtual rows reuse the slot-04 edit and commit bindings while carrying their own generated control id. A recognized editable row strips the synced label/name prefix when the native host sends back the current combined display text, updates the row's raw `value`, rebuilds `value_text` from the label/name plus the new value, refreshes the componentized workbench surface, regenerates retained projection, and returns only paint invalidation. Unknown routes still fall through to the existing pane-surface edit path, so component drawer text fields and non-Workbench edit bindings keep their previous dispatch behavior.

`template_node_labels.rs` now owns retained template-node label resolution. Generic text-input nodes still prefer the focused native edit value, but Workbench `PropertyRow` nodes combine `text` and `value_text` into one visible native label. This is an interim layout bridge: Transform rows such as Position/Rotation/Scale can show both the row label and the live vector string in the native painter until the Inspector receives split label/value field primitives.

`template_property_rows.rs` handles the interim native-painter step for remaining Workbench `PropertyRow` nodes. Instead of letting the generic single-label path draw the whole row, it draws the property label at the left and parses axis-prefixed values such as `X 128.4   Y 64.2   Z -32.7` into compact right-side value fields. The Inspector Transform section has now moved to real declarative axis fields, but the painter fallback still protects any authored property rows that have not yet been migrated.

The Transform section in `workbench_inspector_panel.zui` is now composed bottom-up from `HorizontalGroup`, `Label`, and `WorkbenchAxisValueField` nodes. The row controls still keep the stable aggregate IDs `WorkbenchTransformPosition`, `WorkbenchTransformRotation`, and `WorkbenchTransformScale` for snapshot metadata, while each editable value exports its own retained/native control ID such as `WorkbenchTransformPositionX`. Snapshot sync writes the aggregate Position value for compatibility and also writes X/Y/Z values into those split field controls.

`transform_edit.rs` handles retained/native edit and commit callbacks for those split Transform fields. It recognizes `Inspector/TransformPosition*`, `Inspector/TransformRotation*`, and `Inspector/TransformScale*` Change/Submit routes, strips a native axis prefix such as `X ` when present, updates the edited field's raw `value`, recomposes the aggregate row value as `X ...   Y ...   Z ...`, refreshes the componentized surface, and returns paint-only invalidation. This keeps the first split Inspector fields interactive without pretending that preview edits have already been committed back into the scene world.

`module_field_edit.rs` handles retained/native edit and commit callbacks for module workspace
fields. The bridge recognizes `WorkbenchModule/*` bindings only when the binding resolves to an
`.edit` or `.commit` preview action and the edited control owns the matching
Change/Submit route. A matched field updates `value` and `value_text`, refreshes the componentized
surface, regenerates retained projection, and requests paint-only invalidation. That makes Ability,
Tags, Perception, Render, HUD, and the earlier Effect/Material/Behavior/Assets/VFX field samples
share one field-response rule instead of accumulating module-specific handlers.

Componentized hover, press, and range feedback now use the same route surface instead of depending on authored click bindings for visual response. `refresh_pointer_hover_feedback(...)` mirrors `UiPointerRoute.entered` and `left` into the `hovered` pseudo-state only for enabled workbench controls that declare hoverable input or a known hover-styled workbench class. `refresh_pointer_press_feedback(...)` captures the pressed owner before routing, mirrors primary press/release into component `pressed` state for enabled clickable non-text controls, clears a stale pressed owner when a new press starts, and keeps release binding dispatch unchanged. `refresh_pointer_range_feedback(...)` handles `RangeField`/slider controls without authored change events: primary press captures the pointer, click/drag/release calculates a clamped stepped value from the arranged slider frame and pointer x position, mutates the slider `value`, rebuilds dirty projection, and returns paint-only feedback. The strict workbench theme carries matching `:hovered` and `:pressed` rules for icon buttons, rail buttons, tabs, rows, fields, sliders, toggles, radios, and other component drawer controls. Hover-only, press-only, and slider-drag movement refresh dirty projection without recording an editor event; pointer release on button-like controls still dispatches the authored click/toggle/change binding.

The component-drawer slider preview now has explicit native rows instead of one abstract slider sample. `workbench_component_drawer.zui` declares a dedicated 260 px `WorkbenchComponentSliders` column with `WorkbenchSlidersTitle`, then mounts `WorkbenchInputSlider`, `WorkbenchInputRangeSlider`, and `WorkbenchInputStepsSlider` under that column instead of under Inputs. The surrounding reference columns now use the HTML/CSS proportions for this band: Inputs 214 px, Checkboxes/Radios 168 px, and Labs 236 px. The slider rows keep the `Value`, `Range`, and `Steps` labels, authored value-chip text, shared track/fill/label/thumb tokens, `track_offset_x = -10.0`, `track_width_delta = 18.0`, Range `range_min = 20.0`, and Steps `step_tick_count = 5.0`. They also carry the audited thumb fill/outline/halo as `thumb_color = "#b7f1f8"`, `thumb_outline_color = "#2ab1bc33"`, and `thumb_halo_color = "#32d3de3d"`. `workbench_window_projection.rs` and the generic pane component projection map those aliases into `TemplatePaneNodeData`, and `template_sliders.rs` uses them to paint the left label, declared value chip, corrected track rectangle, Range dual-thumb span, Steps ticks, and declared thumb chrome before falling back to generic template painting.

The component-drawer Labs stack now owns the Labs-only segmented and switch samples from the HTML reference. `workbench_component_drawer.zui` keeps the stable `WorkbenchInputSegmented` and `WorkbenchToggleOn` control ids plus their existing `ComponentLab.*` preview routes, but moves both nodes under `WorkbenchComponentLabs`. The `WorkbenchLabsTabs` strip owns the audited `#1c2226` container background directly, while `WorkbenchLabsTabOne/Two/Three` keep only text, selected state, paint offsets, and click bindings. The switch now resolves its track/thumb/border tones through `.workbench-toggle` and `.workbench-toggle:checked`, so the native selection-control painter consumes strict-theme declarations instead of the older fixed accent/text palette. `workbench_projection.rs` asserts both moved controls are framed inside the Labs column and now also asserts the projected tab-strip background plus checked toggle style colors, so the retained/native projection catches structural and style drift before visual QA.

The lower component table tail keeps its earlier measured shell, content, and cell offsets but now follows the latest Modified-column tone from the HTML/CSS audit. `WorkbenchTableTail.fourth_cell_text_color` is `#aab5ba`, the retained projection maps it into `TemplatePaneNodeData.value_color`, and `template_table_rows.rs` uses that declared value for the fourth cell while leaving the tail deep surface, first three cell tones, overflow glyph, and table selection route unchanged.

The component-drawer body now has a coarse top/lower split. `WorkbenchComponentDrawerBody` is a vertical container with `WorkbenchComponentTopRow` for Buttons, Icon Buttons, Inputs, Checkboxes/Radios, Sliders, Labs, and the side List/Menu stack. `WorkbenchComponentLowerRow` owns `WorkbenchComponentTable` and `WorkbenchComponentFeedback`; table rows therefore live under a lower `Table` title instead of inside the side List column. The List column keeps `WorkbenchListGroup` plus a `Menu` title and `WorkbenchPopupMenu`. `WorkbenchComponentFeedback` is a three-column native feedback layout: a fixed `WorkbenchFeedbackAlerts` alert stack, the fixed `WorkbenchTooltipRoot`, and a fixed `WorkbenchFeedbackToastColumn` that mounts the standalone `feedback_toast` instance without an instance control id so the expanded component root remains `WorkbenchToastRoot`. That keeps the four inline alert ids separate from the notification toast sample.

Native keyboard popup navigation now uses the same structured popup row identities as pointer hover and click. `native_keyboard.rs` maps ArrowDown and ArrowUp to the next enabled dropdown option or popup menu item, skipping disabled options and separator rows, then writes the row identity through the existing transient host interaction state so the cloned presentation highlights the keyboard row without mutating the bridge. Enter activates the current row through the same callback lanes as pointer selection: dropdown rows use `component_showcase_option_selected`, and menu rows use `surface_control_clicked`. Escape emits the shared `WorkbenchPopupCancel` action through `surface_control_clicked`; the retained app dispatch path routes that action into `BuiltinWorkbenchWindowTemplateSurfaceBridge::close_popup(...)`, clears transient option/menu flags, closes the popup/focus/selection state, and requests paint-only refresh without recording a value-selection event.

Native outside-click popup dismissal is split into `native_popup_dismiss.rs` instead of expanding the already large pointer dispatcher. On primary press, `native_pointer.rs` first lets native window-menu clicks resolve, then asks the Workbench popup dismiss helper whether an open structured dropdown or popup menu owns the current interaction. The helper treats the dropdown trigger frame and popup rows as inside the active popup, and treats a menu's full menu frame as inside. A press outside those frames emits the same shared `WorkbenchPopupCancel` action as Escape, clears transient row hover identity, and requests a frame update over the union of the trigger and popup frames. Presses inside option/menu rows continue through the row hit-test path so selection and menu-item preview dispatch are not replaced by cancellation.

## Behavior Model

The constructor creates a `zircon_runtime::ui::surface::UiSurface` with a fixed tree ID of `editor.workbench.reference`. The root is a vertical layout with a fixed 60 px top bar, a fixed 428 px upper editor region, a computed component gallery band, and a fixed 46 px status bar.

The upper region is a horizontal layout. Its fixed columns are the 72 px activity rail, the 332 px hierarchy panel, and the 404 px inspector. The viewport column stretches into the remaining width. With the default 1672 px target width, that leaves an 864 px viewport, matching the reference workbench composition.

The declarative template mirrors these same fixed bands: 60 px toolbar, 428 px main band, 407 px component drawer, and 46 px status bar. At the default width, the main band resolves to 72 px activity rail, 332 px scene tree, 864 px viewport, and 404 px inspector. Because `workbench_window.v2.ui.toml` now mounts large components through wrapper nodes, the host-visible chrome IDs are the wrapper controls such as `WorkbenchWindowTopToolbarRegion`, `WorkbenchWindowMainBandRegion`, `WorkbenchMainBandViewportPanel`, and `WorkbenchWindowStatusBarRegion`. Lower-level controls such as `WorkbenchPrimaryButton` remain directly addressable through the expanded component tree.

Interactive controls are regular runtime UI nodes. Buttons, tree rows, text fields, toggles, slider thumb, and list rows receive `UiInputPolicy::Receive`, pointer state flags, and widget contracts. Button-like nodes also receive click bindings so pointer dispatch can produce component events, not just hit-test results.

The template surface entry point also projects retained host nodes from the same arranged `UiSurface`. This gives the next preview/native-host layer a single object containing the surface render data, the major chrome frames, and the retained host projection with route metadata for controls such as `WorkbenchPrimaryButton` and `WorkbenchToolMove`.

Route IDs are registered when the componentized surface source projection is built. This keeps retained host projection route records actionable for callback dispatch or native host handoff instead of only carrying binding IDs.

## Design And Rationale

The slice deliberately lives under `zircon_editor::ui::workbench::reference` rather than modifying `WorkbenchChromeMetrics::default()`. The existing defaults are used by many retained host and pointer routing tests, so changing them would mix visual modernization with legacy shell behavior. The reference layer lets the new editor proportions be tested independently first.

The implementation reuses runtime UI metadata instead of adding a parallel editor-only widget format. Style is expressed as template metadata because the current render extractor already resolves background, border, radius, foreground, font, and text values from that contract. Layout uses `UiContainerKind::VerticalBox`, `HorizontalBox`, and `GridBox` so the Taffy-backed layout path is exercised by the editor reference surface.

The retained host contract now resolves broad component families through
`template_component_family.rs` before individual Workbench chrome or input semantics are applied.
That resolver prefers declarative component roles, then host roles, then category/layout-role pairs
such as `collection/grid` or `container/editor-dock`, and finally legacy Workbench control ids.
Button, IconButton, TextInput, Slider, Checkbox, Radio, Toggle, Dropdown, Tab, SegmentedControl,
ListRow, TreeRow, TableRow, Popup, Tooltip, Alert, Drawer, and Window therefore share one
classification point. Workbench visual styling is still gated by explicit Workbench ids or
`workbench-*` variants, so generic Material or future editor components can use the same family
tokens without inheriting the Workbench shell skin by accident. `template_input_semantics.rs` then
uses the resolved `TextInput` family for native focus and edit-target selection, keeping those
rules out of the oversized pointer router.

`template_activation_semantics.rs` is the companion routing layer for primary template-node
activation. It turns a `TemplateNodePointerHit` into a focused callback route for text inputs,
Inspector controls, asset click/change controls, welcome controls, component showcase controls,
structured dropdown option rows, popup menu rows, binding-backed controls, or action-backed
controls before `native_pointer.rs` invokes the host. The native pointer path still decides pointer
state, damage, and focus timing; the semantic module owns the component-family-aligned route choice.

The builder is split by responsibility to keep the module maintainable. Panel assembly can grow as the visual target becomes richer, while node construction remains the single place that defines how editor reference controls become runtime UI nodes.

## Control Flow

`build_editor_workbench_reference_surface()` creates a `ReferenceSurfaceBuilder`, which inserts the root and then adds the major workbench bands. Callers then run `compute_reference_layout()`, which delegates to `UiSurface::compute_layout()` using the default target size from `EditorWorkbenchReferenceMetrics`.

`build_editor_workbench_template_surface(runtime, metrics)` asks `EditorUiHostRuntime` for the registered `editor.window.workbench` v2 document, builds its shared surface, computes layout, extracts `EditorWorkbenchTemplateFrames`, and calls `build_retained_host_projection_with_surface(...)` so retained/native presentation can consume the same expanded component tree.

`BuiltinWorkbenchWindowTemplateSurfaceBridge::new_with_runtime(...)` is the retained-host entry point for callers that already own an `EditorUiHostRuntime`, including startup. It builds the componentized workbench surface, recomputes it to the requested shell size when different from the 1672 x 941 reference size, and exposes bindings through the same `binding_for_control(...)` helper used by the legacy host-window bridge.

After layout, the normal runtime rebuild path produces arranged nodes, hit-test data, and render commands. The tests inspect the same `surface_frame()`, `render_extract`, and pointer dispatch APIs that production runtime consumers use.

When a bridge-level state change is applied, `UiSurface::mutate_property(...)` updates the expanded component node metadata and component-state store, `refresh_after_state_change(...)` rebuilds only the dirty surface using the current shell size, and retained host projection is regenerated from that same arranged surface. This keeps visual response, layout frames, and callback-dispatch route metadata synchronized.

Component-lab preview menu actions are recorded as transient preview feedback instead of editor history operations. The recorded `PressNode` path preserves the full binding path such as `ComponentLab/CheckboxOffToggle`, not only the short control id, so preview feedback remains namespaced even when different panels reuse similar control names.

Icon-bearing controls rely on the runtime interface paint-element split documented in `docs/zircon_runtime_interface/ui/surface/render.md`. A selected toolbar icon now emits background chrome, the icon image, optional text, and border as separate paint elements, so native preview pixels can show the selected button frame rather than only the icon glyph.

During `RetainedEditorHost::recompute_if_dirty`, the workbench-window bridge is recomputed to the current shell size and then synchronized from the final chrome snapshot immediately before `apply_presentation(...)`. This ordering matters because viewport resize can rebuild the chrome snapshot mid-recompute. The bridge sync therefore consumes the final scene hierarchy and inspector data for the same presentation frame that native host painting receives.

## Edge Cases And Constraints

This module does not yet render the real engine scene into the central viewport. The viewport is represented as styled UI panels and a grid-backed canvas placeholder so the layout and rendering path can be validated first, while the surrounding workbench shell is already projected through the retained host workbench-window node path.

The node IDs are intentionally stable. Do not convert them to allocation-only IDs unless the screenshot and hit-test tooling has another stable lookup key.

Label text must be written to both `label` and `text` metadata attributes. Buttons can render from `label`, but plain `Label` nodes resolve text through the generic text/value path.

## Test Coverage

`reference_workbench_surface_lays_out_target_editor_chrome` verifies the target frame proportions and confirms that the layout report selected Taffy-backed containers.

`reference_workbench_surface_extracts_renderable_panels_and_controls` verifies that the surface emits renderable quad and text commands for the viewport, primary control, labels, and status bar text.

`reference_workbench_surface_routes_primary_button_pointer_response` verifies that a real pointer down/up sequence hits the primary button, sets the pressed target, and emits a click component event through runtime dispatch.

`reference_workbench_componentized_window_surface_matches_reference_chrome_metrics` verifies that the componentized v2 template expands through the editor built-in template runtime, computes the same Taffy-backed chrome frames as the direct reference surface, preserves clickable/focusable metadata on the primary button component, exports retained host projection nodes and click routes for key controls, and recomputes the viewport width plus retained projection frame when the window width changes.

`componentized_workbench_window_template_bridge_exports_surface_projection_frames_and_routes` verifies the retained-host bridge exposes the arranged surface, major chrome frames, retained host projection frames, route IDs, primary-button editor operation binding, viewport tool binding, and resized viewport projection.

`componentized_workbench_window_template_bridge_updates_tool_selection_state`, `componentized_workbench_window_template_bridge_updates_activity_rail_selection_state`, and `componentized_workbench_window_template_bridge_updates_scene_tree_selection_state` verify bridge-dispatched clicks update selected state, checked state, render backgrounds, and retained projection frames for the tool strip, activity rail, and scene tree. The tool test also locks the resized viewport frame after a state-only refresh so interaction response cannot accidentally reset shell layout.

`componentized_workbench_window_template_bridge_updates_panel_tab_state`, `componentized_workbench_window_template_bridge_updates_component_drawer_tab_state`, `componentized_workbench_pointer_dispatch_hits_component_drawer_tab_visibility`, `componentized_workbench_window_template_bridge_updates_component_drawer_input_state`, `componentized_workbench_pointer_focuses_input_fields_without_authored_binding`, `componentized_workbench_pointer_hover_updates_icon_button_preview_without_authored_binding`, `componentized_workbench_pointer_press_updates_icon_button_preview_before_release_binding`, `componentized_workbench_pointer_drag_updates_slider_value_without_authored_binding`, and `componentized_workbench_window_template_bridge_updates_component_drawer_selection_state` cover componentized tab/body visibility and component-lab interaction feedback. The input-focus test targets the no-authored-binding path: clicking `WorkbenchInputText` focuses its runtime component state, redraws the strict focused field background/border, returns only `PAINT_ONLY`, and then clicking `WorkbenchInputStepper` blurs the first field while focusing the second without recording an editor event. The hover-preview test targets movement without activation: entering `WorkbenchToolMove` flips `hovered`, redraws the strict hover background, returns only `PAINT_ONLY`, and leaving the surface clears the hover state without recording an editor event. The press-preview test targets primary down before activation: pressing `WorkbenchToolMove` flips component `pressed`, redraws the active button background, returns only `PAINT_ONLY`, records no event, and then release clears pressed state while dispatching the original viewport tool binding. The slider-preview test targets the range path: pressing `WorkbenchInputSlider` captures the pointer, clicking at 25 percent changes `value` to `25`, dragging to 80 percent changes it to `80`, and release clears pressed state without recording an editor event.

`componentized_workbench_window_projection_exports_dropdown_and_popup_rows` verifies the new host-contract row projection for the component drawer. It asserts that `WorkbenchInputDropdown` exports three structured option rows with selected, special, focused, hovered, and disabled state, locks the bottom-row `WorkbenchInputDropdown` and `WorkbenchInputStepper` frames to the declared 30.5 height, and verifies that `WorkbenchPopupMenu` exports parsed menu rows with a real separator row plus the hovered Delete item.

`componentized_workbench_dropdown_option_selection_updates_value_and_projection` covers the first option-selection state transition. It opens the dropdown, selects `option_a` through the Workbench callback-dispatch path, verifies the value text, popup/focus closure, typed host-contract selected/special row state, paint-only invalidation, and `ComponentLabPreview` event recording, then verifies disabled `option_b` does not change the value or record a second event.

`componentized_workbench_popup_cancel_closes_dropdown_without_value_dispatch` covers the dropdown cancel transition. It opens `WorkbenchInputDropdown`, dispatches the shared popup-cancel action, verifies popup/focus/selection closure, preserves the existing dropdown value, clears transient option focus/hover/press state in the retained projection, requests paint-only invalidation, and confirms a repeated cancel on a closed dropdown is a no-op.

`componentized_workbench_popup_menu_item_selection_updates_value_and_projection` covers popup menu row selection. It selects the hovered `Delete` row in `WorkbenchPopupMenu`, verifies the popup value, popup/focus closure, paint-only invalidation, transient menu-row flag cleanup in `structured_menu_items`, and that no real editor event is recorded for the visual-only component sample.

`componentized_workbench_popup_cancel_closes_menu_without_selecting_item` covers the popup-menu cancel transition. It dispatches the shared popup-cancel action against `WorkbenchPopupMenu`, verifies popup/focus/selection closure, clears the authored hovered `Delete` flag from structured menu projection, requests paint-only invalidation, and rejects unrelated action IDs so normal menu item dispatch stays isolated.

`workbench_hit_test_routes_open_dropdown_option_rows` and `workbench_hit_test_routes_open_popup_menu_rows` cover the native hit-test bridge for those structured rows. They assert that an open dropdown option row produces a `workbench_option` pointer hit with the Change binding and option id, while an open popup menu row produces a `workbench_menu_item` hit with the menu action id. `workbench_hit_test_blocks_popup_menu_separator_row` verifies that a non-activatable row inside an open popup blocks fallback to the popup parent node.

`template_nodes_paint_open_dropdown_option_rows_below_control` and `template_nodes_paint_open_popup_menu_rows_inside_menu_frame` cover the retained native painter side of the same row contract. They paint open structured dropdown and menu nodes into an RGBA buffer and assert that the expected popup-row regions are no longer blank.

`dropdown_option_popup_frame_within_opens_above_when_below_overflows`, `dropdown_option_popup_frame_within_keeps_default_when_above_also_overflows`, and `dropdown_option_popup_frame_within_clamps_right_edge` cover the bounded dropdown geometry primitive directly. `workbench_hit_test_routes_dropdown_option_rows_above_control_when_bottom_clipped` verifies the same bounded geometry feeds native pointer hit testing, while `template_nodes_paint_open_dropdown_option_rows_above_control_when_below_clipped` verifies native painting uses the above-control popup frame instead of drawing rows past the bottom edge.

`native_workbench_dropdown_option_row_hover_updates_structured_row_state` and `native_workbench_popup_menu_row_hover_updates_structured_row_state` cover the native pointer-move path for synthetic popup rows. They install the componentized Workbench projection into a host presentation, move the pointer over dropdown/menu rows, and assert that the cloned presentation changes the hovered structured row while clearing the previous authored demo hover/focus flags.

`native_workbench_dropdown_keyboard_moves_row_hover_and_enter_dispatches_option` and `native_workbench_popup_menu_keyboard_moves_row_hover_and_enter_dispatches_menu_item` cover the matching native keyboard activation path. They open or use an open Workbench popup, move the highlighted row with ArrowDown, verify the cloned presentation row state, then press Enter and assert the host callback receives the same option/menu identity that pointer selection would dispatch. `native_workbench_dropdown_escape_dispatches_popup_cancel` and `native_workbench_popup_menu_escape_dispatches_popup_cancel` cover the cancellation path: ArrowDown first establishes a transient highlighted row, Escape then emits `WorkbenchPopupCancel` for the active popup control, clears the transient hover identity, and requests a frame update for the popup region.

`native_workbench_dropdown_option_primary_press_keeps_selection_path` and `native_workbench_popup_menu_item_primary_press_keeps_menu_selection_path` cover the inside-row guard for outside dismissal: a press inside an open dropdown option still dispatches `component_showcase_option_selected`, and a press inside an open popup menu row still dispatches the row action id. `native_workbench_disabled_dropdown_option_primary_press_is_ignored_without_cancel` and `native_workbench_popup_menu_separator_primary_press_is_ignored_without_cancel` cover the disabled/separator lane: a primary press inside a disabled dropdown option or menu separator is treated as inside the popup but does not dispatch selection, does not cancel, and keeps the popup open. `native_workbench_dropdown_outside_primary_press_dispatches_popup_cancel` and `native_workbench_popup_menu_outside_primary_press_dispatches_popup_cancel` cover the outside-click path by establishing transient row hover, pressing at a point outside the popup frames, asserting `WorkbenchPopupCancel`, and verifying the transient hover identity is cleared.

`native_workbench_text_input_focuses_edits_and_commits_from_keyboard` covers the first native keyboard editing lane for the component drawer text field. It clicks `WorkbenchInputText`, verifies the projected edit/commit action IDs, inserts text through the host text input path, and presses Enter to assert the host callback receives both `ComponentLab/InputTextEdit` and `ComponentLab/InputTextCommit` with the updated field value.

`native_workbench_module_field_focuses_edits_and_commits_from_keyboard` extends the same native
keyboard lane to module workspaces. It switches the componentized Workbench into the Ability panel,
clicks `WorkbenchAbilityNameField`, verifies the `WorkbenchModule/AbilityNameEdit` and
`WorkbenchModule/AbilityNameCommit` routes, inserts `_Preview` through the host text path, and
asserts Enter dispatches the updated module field value.

`componentized_workbench_window_template_bridge_syncs_scene_and_inspector_snapshot_state` verifies the live-data projection slice: scene snapshot rows update text/depth/selection, the reusable seventh and eighth slots accept live entries, the next unused slot collapses, inspector title and Transform position update from the selected entity snapshot, the first four plugin component properties populate authored Inspector property rows, the fifth property materializes as `WorkbenchComponentPropertyVirtualRow05`, and the virtual row exports the retained slot-04 edit route.

`componentized_workbench_inspector_property_edit_updates_row_preview` covers the Inspector property edit state transition. It syncs five live component properties into the authored plus virtual row pool, dispatches authored row edit/commit routes and a generated virtual-row edit route through the Workbench surface edit dispatcher using the same combined label/value text the native focus path currently returns, verifies raw values and summary text update in bridge metadata and retained host projection, and confirms the response is paint-only.

`componentized_workbench_transform_axis_edit_updates_field_and_row_preview` covers the first Transform split-field edit state transition. It verifies that `WorkbenchTransformPositionX` exports Change/Submit retained routes, edits Position X through the native edit dispatcher, commits Rotation Y with an axis-prefixed native value, edits Scale Z, and checks that each split field plus its aggregate row value refreshes with paint-only invalidation.

`componentized_workbench_module_field_edit_updates_value_preview` covers the module-field edit
state transition. It verifies that `WorkbenchAbilityNameField` exports Change/Submit routes, edits
the Ability field through the same native surface edit dispatcher, commits the Render pipeline
field, checks `value` and `value_text` projection refresh, and confirms a known module edit binding
is swallowed without repaint when sent to the wrong control.

`reference_workbench_componentized_window_surface_matches_reference_chrome_metrics` now also verifies that `WorkbenchTransformPositionX/Y/Z` are projected as retained `InputField` nodes with `workbench-axis-value-field` styling and stable left-to-right Taffy frames. The bridge snapshot sync regression also verifies the live selected entity translation is split into the Position X/Y/Z field values while preserving the aggregate `WorkbenchTransformPosition` value string.

The same reference surface regression now covers the first native Effect module workspace sample.
It verifies `WorkbenchMainBandModuleWorkspace` is visible in the main-band host projection,
`WorkbenchModuleEffectWorkspace` spans that band after the rail-alignment gap, and the Effect
search field, selected `GE_HealthRegen` row, modifier table row, and Magnitude field project as
typed retained controls with their representative `WorkbenchModule/*` Change, Submit, or Click
bindings. This keeps the new module panel in the editor-native template path rather than treating
the browser prototype as the deliverable.

The same route coverage now extends to the other module panels without changing the outer shell.
Material, Behavior, Assets, and VFX workspaces share the same rail-aligned three-column layout as
Effect. The regression checks representative routes for `WorkbenchMaterialDomainDropdown`,
`WorkbenchMaterialNodeRow02`, `WorkbenchBehaviorNodeRow03`, `WorkbenchAssetsTableRow03`, and
`WorkbenchVfxSystemField`; the bridge navigation regression switches to Material, verifies the
72 px rail gap in the visible module workspace, selects a material graph row, and dispatches a
Material domain edit action. These checks keep the new panels interactive as preview surfaces while
real editor data binding remains a later milestone.

The additional-module native preview now mounts `workbench_additional_module_workspaces.zui` under
the same module overlay. Ability, Tags, Perception, Render, and HUD reuse the fixed rail gap plus
left/center/right grammar from the earlier module panels; route coverage asserts representative
tabs, rows, commands, and fields for those panels without claiming real gameplay, AI, renderer, or
UI-authoring behavior. The bridge state-response test switches across those module tabs and checks
the expanded `WorkbenchModuleWorkspace` state plus visible child workspace changes while the outer
Workbench shell projection stays unchanged.

`property_row_label_keeps_label_and_value_visible` covers the retained painter label helper directly. It verifies that a `property-row` node with `text = "Position"` and `value_text = "X 12.0   Y 3.5   Z -8.0"` paints as the combined native label instead of dropping the value behind the generic text-first fallback.

`property_axis_values_group_units_with_axis_value` covers the native PropertyRow field splitter. It verifies that axis-prefixed values with units, for example `X 0 deg   Y 90 deg   Z -12.5 deg`, are grouped into three field payloads instead of treating `deg` as a new label or dropping it during painting.

`componentized_workbench_scene_tree_grows_and_reuses_virtual_rows_for_live_snapshot_state` verifies the dynamic hierarchy bridge beyond the authored ten-row baseline. It syncs thirteen live entries, confirms virtual row text, scene-node id, selected state, retained host projection output, and authored binding resolution, then shrinks the hierarchy to recycle virtual rows and grows it again to prove the node pool reuses them.

The same scene-tree regression now also checks that `WorkbenchSceneTree` carries a `repeat` metadata table with `kind = "virtual_rows"`, `prototype = "WorkbenchSceneSlot10Item"`, `virtual_control_prefix = "WorkbenchSceneVirtualItem"`, and `authored_count = 10`. `ui_v2_repeat_declaration_is_preserved_in_compiled_surface_metadata` covers the lower runtime path that preserves repeat metadata from TOML through the compiled arena and into surface metadata.

`componentized_workbench_surface_paints_native_preview_pixels_and_interaction_state` paints the componentized workbench render extract into an RGBA buffer through the native test painter. It verifies nonblank chrome and viewport pixels, dispatches a tool click through the bridge, then checks that pixels inside the selected tool frame changed. This catches regressions where render commands carry state changes but image-bearing native paint output still hides the selected chrome.

`template_component_family` unit coverage locks the shared host-contract classification layer. It verifies
declared component roles take precedence over control ids, collection layout roles become list/table
families, Workbench visual language can be declared without a Workbench control prefix, and
`range-field` plus `WorkbenchInputSlider` classify as Slider instead of TextInput.

`template_input_semantics` unit coverage locks the first input-system consumer of that family
contract. It verifies a `TextInput` family hit can use its binding as the edit target, while
structured popup rows never inherit text-input focus from their source control. The template
hit-test coverage also verifies a `text-field` node without the legacy `input-field` role still
enters the hit surface as `TemplateComponentFamily::TextInput`.

`template_activation_semantics` unit coverage locks the primary-click route layer. It verifies
TextInput family hits stay focus-only, structured dropdown options do not fall back to inherited
bindings, popup menu rows route as surface actions, and asset dispatch kinds classify click versus
change controls with the expected source.

The same native preview test can now write the current editor workbench render to a PNG for visual comparison. Set `ZIRCON_WRITE_WORKBENCH_PREVIEW=1` and `ZIRCON_WORKBENCH_PREVIEW_PATH=<png>` when running the focused test to emit the 1672 x 941 native screenshot without adding a production screenshot dependency.

On 2026-06-03 the focused screenshot run passed with `cargo test -p zircon_editor --lib componentized_workbench_surface_paints_native_preview_pixels_and_interaction_state --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-preview-0603 -- --nocapture`. It wrote `target/editor-workbench-visual-check/editor-workbench-native-1672x941.png`; the comparison artifact against the HTML component template is `target/editor-workbench-visual-check/editor-workbench-web-vs-native-1672x941.png`.

The later primitive-family routing pass reran the same native preview test after recompilation and
wrote `target/editor-workbench-visual-check/editor-workbench-native-family-1672x941.png`. Its main
visual evidence artifact is
`target/editor-workbench-visual-check/editor-workbench-design-vs-native-family-1672x941.png`, which
pairs the user-provided `docs/ui-and-layout/workbench.png` design reference with the current
retained/native render. The browser-prototype comparison remains available as
`target/editor-workbench-visual-check/editor-workbench-web-vs-native-family-1672x941.png`, but it is
secondary because the browser mock can now show module-specific panels that do not match the
Workbench window layout.

The input-family routing continuation reran the native preview test after moving component-family
resolution into the shared host contract and adding `template_input_semantics.rs`. It wrote
`target/editor-workbench-visual-check/editor-workbench-native-input-family-1672x941.png`; the updated
design-reference comparison is
`target/editor-workbench-visual-check/editor-workbench-design-vs-native-input-family-1672x941.png`.

The activation-family routing continuation reran `native_workbench_reference` after extracting
template primary-click dispatch into `template_activation_semantics.rs`. It wrote
`target/editor-workbench-visual-check/editor-workbench-native-activation-family-1672x941.png`; the
updated design-reference comparison is
`target/editor-workbench-visual-check/editor-workbench-design-vs-native-activation-family-1672x941.png`.

The visible Effect module workspace pass reran the native screenshot path with
`ZIRCON_WRITE_WORKBENCH_PREVIEW=1` and wrote
`target/editor-workbench-visual-check/editor-workbench-native-module-workspace-1672x941.png`.
The side-by-side comparison against the accepted AI gameplay-effect shell reference is
`target/editor-workbench-visual-check/editor-workbench-gameplay-effect-vs-native-module-workspace-1672x941.png`.

The multi-module route continuation after that screenshot keeps the same native workbench target.
It adds aligned inactive workspaces plus preview routes for Material, Behavior, Assets, and VFX
rows/fields/dropdowns, with light validation through TOML parsing, Rust formatting, targeted diff
checking, whitespace scanning, a direct reference-surface test-binary pass, and a direct
`native_workbench_reference` screenshot pass. It wrote
`target/editor-workbench-visual-check/editor-workbench-native-multimodule-routes-1672x941.png`;
the refreshed comparison artifact is
`target/editor-workbench-visual-check/editor-workbench-gameplay-effect-vs-native-multimodule-routes-1672x941.png`.
The bridge regressions now also pass from the refreshed test binary after correcting the test
expectations from the composite instance id to the expanded `WorkbenchModuleWorkspace` runtime node.

The module-field keyboard continuation reran the refreshed `zircon_editor` test binary directly.
`native_workbench_module_field_focuses_edits_and_commits_from_keyboard` passed with 1 passed /
1838 filtered, and the full `native_workbench_reference` screenshot run passed 16 tests / 1823
filtered. It wrote
`target/editor-workbench-visual-check/editor-workbench-native-module-field-keyboard-1672x941.png`;
the current comparison against the accepted `ai-gameplay-effect-layout.png` shell reference is
`target/editor-workbench-visual-check/editor-workbench-ai-gameplay-vs-native-module-field-keyboard-1672x941.png`.
That rerun also fixed the module-switching shell contract: selecting a module now keeps
`WorkbenchSceneWorkspace` visible so the activity rail remains part of the outer Workbench shell,
while `WorkbenchMainBandModuleWorkspace`, `WorkbenchModuleWorkspace`, and the concrete module
workspace controls own module content visibility.

The module-navigation shell follow-up reran the focused navigation regressions after locking that
outer-shell behavior. `workbench_module_tabs_switch_exactly_one_module_workspace` and
`workbench_scene_tab_restores_scene_workspace_and_hides_module_workspaces` both passed from the
refreshed editor test binary, alongside reruns of
`componentized_workbench_module_field_edit_updates_value_preview` and
`native_workbench_module_field_focuses_edits_and_commits_from_keyboard`. The full
`native_workbench_reference` screenshot run passed 16 tests / 1823 filtered and wrote
`target/editor-workbench-visual-check/editor-workbench-native-module-navigation-shell-1672x941.png`;
the current side-by-side comparison against `ai-gameplay-effect-layout.png` is
`target/editor-workbench-visual-check/editor-workbench-ai-gameplay-vs-native-module-navigation-shell-1672x941.png`.

The visible-frame follow-up splits two frame lookup contracts that previously shared one helper.
`EditorWorkbenchTemplateSurface::control_frame(...)` is now the raw template-surface lookup used by
`EditorWorkbenchTemplateFrames::from_surface(...)` for required shell frame refresh. The retained
bridge calls `visible_control_frame(...)`, which walks render visibility through arranged/tree
ancestors before returning a frame. This keeps collapsed module workspaces out of bridge-facing
projection checks and hit-test expectations, while required shell controls such as
`WorkbenchMainBandActivityRail` cannot disappear during a dirty rebuild.

Validation for that split passed on 2026-06-03 in
`E:\cargo-targets\zircon-editor-workbench-preview-0603`: the two focused module navigation tests,
the decorative viewport pointer-dispatch test, the decorative viewport host hit-test unit, the
broader `workbench_module` filter with 4 passed tests, and
`componentized_workbench_window_template_bridge_syncs_scene_and_inspector_snapshot_state`. The
runtime/app side of the same asset-flow continuation also passed scoped library checks, but full M6
workspace acceptance remains open outside this module document.

The module-command feedback follow-up adds
`module_command_feedback.rs` as the shared preview response table for module commands. Top-level
Save/Browse/Compile/Diff/Simulate and panel commands such as Ability Playtest, Render Compile,
Assets Import, VFX Simulate, Tags Add/Rename, Perception Simulate, and HUD Preview now update the
existing Workbench status items plus the relevant module output row through normal retained
surface property mutations. The outer shell, module workspace layout, and component declarations
stay unchanged; the bridge only maps command action ids to `text` or `value_text` updates on
already-declared StatusItem/ListRow/TableRow/PropertyRow controls. The regression
`workbench_module_commands_update_status_and_module_output_rows` covers Ability Playtest, Render
Compile, and Browse-to-Assets through the normal module dispatch path, while
`componentized_workbench_module_command_feedback_paints_native_preview_pixels` renders the
post-command native surface to prove the feedback appears in the real editor paint path.

The command-feedback screenshot was generated on 2026-06-03 with
`ZIRCON_WRITE_WORKBENCH_PREVIEW=1`, selecting the Ability module and dispatching Playtest before
paint. The native output is
`target/editor-workbench-visual-check/editor-workbench-native-command-feedback-1672x941.png`; the
side-by-side comparison against `docs/ui-and-layout/ai-workbench-style/ai-gameplay-effect-layout.png`
is
`target/editor-workbench-visual-check/editor-workbench-ai-gameplay-vs-native-command-feedback-1672x941.png`.

The module-dropdown follow-up extends the same `WorkbenchDropdown` component behavior from the
component drawer into the module workspaces without adding per-control layout code. When a module
field action ends with `.edit` and the source control exposes authored `options`, the componentized
Workbench bridge treats the Change dispatch as a shared dropdown activation and opens the popup on
that control. Option selection continues through `popup_state.rs`, so module
dropdowns reuse the existing option validation, `value` / `value_text` writeback, transient
option-state clearing, popup closure, and retained projection refresh path. The regression
`workbench_module_dropdowns_open_select_and_close_with_shared_dropdown_path` covers the Material
Domain dropdown opening through its module field binding, selecting `post_process`, rejecting an
unknown option as a no-op, and repainting only the affected Workbench surface state.

The module-dropdown screenshot evidence was generated on 2026-06-03 with
`ZIRCON_WRITE_WORKBENCH_PREVIEW=1`. The native open-state output is
`target/editor-workbench-visual-check/editor-workbench-native-module-dropdown-open-1672x941.png`;
the selected-state output is
`target/editor-workbench-visual-check/editor-workbench-native-module-dropdown-selected-1672x941.png`.
The comparison captures against
`docs/ui-and-layout/ai-workbench-style/ai-material-editor-layout.png` are
`target/editor-workbench-visual-check/editor-workbench-ai-material-vs-native-module-dropdown-open-1672x941.png`
and
`target/editor-workbench-visual-check/editor-workbench-ai-material-vs-native-module-dropdown-selected-1672x941.png`.

The toolbar window-menu follow-up keeps the top toolbar in the native componentized editor window
instead of returning to the browser sample. `workbench_window.v2.ui.toml` now owns the menu overlay
at the window root: the normal workbench content stays in one vertical content child, while Main,
Run Mode, and Layout are sibling `WorkbenchPopupMenu` wrapper nodes above it.
`workbench_top_toolbar.zui` remains a horizontal toolbar component and only owns the trigger
controls. The retained bridge routes `OpenMainMenu`, `OpenRunModeMenu`, and `OpenLayoutMenu`
through `window_menu_state.rs`, which opens the requested menu, closes the other toolbar menus,
updates the trigger selected/checked state, and reuses `popup_state.rs` for item selection and
cancellation. Runtime `popup_menu.rs` still expands the wrapped `ContextActionMenu.menu_items`
into visible render commands, so the native screenshot path shows menu rows from the same template
state while production `.v2.ui.toml` imports reach the Workbench `.zui` wrapper assets directly.
This keeps the response model component-level and table-driven instead of adding per-button pixel
tweaks.

The toolbar menu screenshot evidence was generated on 2026-06-03 with
`ZIRCON_WRITE_WORKBENCH_PREVIEW=1`. The native open-state output is
`target/editor-workbench-visual-check/editor-workbench-native-toolbar-run-menu-open-1672x941.png`.
The comparable window artifact against the current web component template screenshot is
`target/editor-workbench-visual-check/editor-workbench-web-template-vs-native-toolbar-run-menu-open-1672x941.png`.

`surface_property_mutation_keeps_template_visibility_metadata_in_sync` belongs to the shared runtime surface layer, but it is part of this workbench acceptance chain: it proves that a runtime write from `collapsed` to `visible` updates both the retained node and template metadata before the next layout refresh.

`startup_template_runtime_loads_componentized_workbench_window_bridge_source` verifies the startup built-in template runtime includes the componentized workbench window document, so the bridge is available from the same runtime bundle as the existing host shell bridges.

## Open Issues Or Follow-up

The next layer should move repeat materialization from `TemplateBridgeVirtualRowSequence` into a reusable runtime/editor list primitive, then replace the interim dropdown-backed Inspector component-property summaries and remaining native `PropertyRow` painting with split label/value field primitives for labels, values, validation state, and type-specific editors. After that, the viewport placeholder should be replaced by engine-rendered scene content and verified through a running-window capture path.
