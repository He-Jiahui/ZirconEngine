use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 M3 UI component catalog test folder split",
        &[
            "runtime_15_ui_component_catalog_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/component_catalog.rs",
            "ui/tests/component_catalog/catalog_inventory.rs",
            "runtime_15_ui_component_catalog_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI boundary test folder split",
        &[
            "runtime_15_ui_boundary_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/boundary.rs",
            "ui/tests/boundary/template_namespace.rs",
            "runtime_15_ui_boundary_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI boundary ZUI surface projection guard sync",
        &[
            "runtime_15_ui_boundary_zui_surface_projection_guard_sync_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/ui_boundary.rs",
            "zui_surface_projection_does_not_call_template_tree_builder",
            "ui/tests/boundary/asset_fixture_projection.rs",
            "runtime_15_ui_boundary_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI component state test folder split",
        &[
            "runtime_15_ui_component_catalog_component_state_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/component_catalog/component_state.rs",
            "ui/tests/component_catalog/component_state/reference_sources.rs",
            "runtime_15_ui_component_catalog_component_state_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI component state keyboard test folder split",
        &[
            "runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/component_catalog/component_state/keyboard.rs",
            "ui/tests/component_catalog/component_state/keyboard/action_selection.rs",
            "runtime_15_ui_component_catalog_component_state_keyboard_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI Material foundation test folder split",
        &[
            "runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/component_catalog/material_foundation/mod.rs",
            "ui/tests/component_catalog/material_foundation/planned_layers.rs",
            "runtime_15_ui_component_catalog_material_foundation_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI asset test folder split",
        &[
            "runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/asset.rs",
            "ui/tests/asset/style_rule_ids.rs",
            "runtime_15_ui_asset_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI asset surface index test folder split",
        &[
            "runtime_15_ui_asset_surface_index_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/asset_surface_index.rs",
            "ui/tests/asset_surface_index/surface_edges.rs",
            "ui/tests/asset_surface_index/dirty_targets.rs",
            "runtime_15_ui_asset_surface_index_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI asset MUI web form style test folder split",
        &[
            "runtime_15_ui_asset_mui_web_form_style_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/asset_mui_web_form_style.rs",
            "ui/tests/asset_mui_web_form_style/form_controls.rs",
            "runtime_15_ui_asset_mui_web_form_style_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI asset MUI X web style test folder split",
        &[
            "runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/asset_mui_web_mui_x_style.rs",
            "ui/tests/asset_mui_web_mui_x_style/data_grid.rs",
            "runtime_15_ui_asset_mui_web_mui_x_style_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI asset MUI web style test folder split",
        &[
            "runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/asset_mui_web_style.rs",
            "ui/tests/asset_mui_web_style/state_icons.rs",
            "runtime_15_ui_asset_mui_web_style_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI taffy layout pass test folder split",
        &[
            "runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/taffy_layout_pass.rs",
            "ui/tests/taffy_layout_pass/routing_diagnostics.rs",
            "runtime_15_ui_taffy_layout_pass_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI runtime window input pump test folder split",
        &[
            "runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_window_input_pump.rs",
            "ui/tests/runtime_window_input_pump/lifecycle.rs",
            "runtime_15_ui_runtime_window_input_pump_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI runtime window event ABI child folder split",
        &[
            "runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_ui_window_event_routes/abi.rs",
            "ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs",
            "runtime_15_ui_runtime_window_event_abi_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 test file budget root-layout UI child split",
        &[
            "runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/root_layout.rs",
            "structure_convention/test_file_budget/root_layout/ui_children.rs",
            "runtime_15_test_file_budget_root_layout_ui_child_scan_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 UI widget text input keyboard test folder split",
        &[
            "runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/widget_text_input_keyboard.rs",
            "ui/tests/widget_text_input_keyboard/basic_editing.rs",
            "runtime_15_ui_widget_text_input_keyboard_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI focus navigation test folder split",
        &[
            "runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/focus_navigation.rs",
            "ui/tests/focus_navigation/focus_state.rs",
            "runtime_15_ui_focus_navigation_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI runtime input manager test folder split",
        &[
            "runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_input_manager.rs",
            "ui/tests/runtime_input_manager/route_matrix.rs",
            "runtime_15_ui_runtime_input_manager_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI runtime input ownership test folder split",
        &[
            "runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_input_ownership.rs",
            "ui/tests/runtime_input_ownership/input_method.rs",
            "runtime_15_ui_runtime_input_ownership_tests_are_folder_backed",
        ],
    ),
];
