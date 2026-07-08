type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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
];
