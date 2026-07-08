type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 UI architecture test folder split",
        &[
            "runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/ui_architecture.rs",
            "tests/runtime_absorption/ui_architecture/architecture_boundaries.rs",
            "tests/runtime_absorption/ui_architecture/legacy_renames.rs",
            "runtime_15_ui_architecture_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI v2 asset test folder split",
        &[
            "runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked",
            "ui/tests/v2_asset.rs",
            "ui/tests/v2_asset/style_runtime.rs",
            "runtime_15_ui_v2_asset_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI v2 style-runtime test folder split",
        &[
            "runtime_15_ui_v2_style_runtime_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/v2_asset/style_runtime.rs",
            "ui/tests/v2_asset/style_runtime/static_resolution.rs",
            "ui/tests/v2_asset/style_runtime/runtime_pseudo_state.rs",
            "ui/tests/v2_asset/style_runtime/resolved_pseudo_state.rs",
            "ui/tests/v2_asset/style_runtime/property_mutation.rs",
            "ui/tests/v2_asset/style_runtime/style_overrides.rs",
            "runtime_15_ui_v2_style_runtime_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core test folder split",
        &[
            "runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked",
            "ui/tests/shared_core.rs",
            "ui/tests/shared_core/layout_surface.rs",
            "runtime_15_ui_shared_core_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core guard child-owner split",
        &[
            "runtime_15_ui_shared_core_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/ui_shared_core.rs",
            "structure_convention/test_file_budget/ui_shared_core/layout_surface.rs",
            "runtime_15_ui_shared_core_guard_child_owners_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core input visibility child folder split",
        &[
            "runtime_15_ui_shared_core_input_visibility_child_folder_split_static_passed_cargo_deferred",
            "ui/tests/shared_core/input_visibility.rs",
            "ui/tests/shared_core/input_visibility/hit_visibility.rs",
            "ui/tests/shared_core/input_visibility/pointer_routes.rs",
            "runtime_15_ui_shared_core_input_visibility_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core scroll mutation child folder split",
        &[
            "runtime_15_ui_shared_core_scroll_mutation_child_folder_split_static_passed_cargo_deferred",
            "ui/tests/shared_core/scroll_mutation.rs",
            "ui/tests/shared_core/scroll_mutation/property_mutation.rs",
            "ui/tests/shared_core/scroll_mutation/virtual_scroll.rs",
            "runtime_15_ui_shared_core_scroll_mutation_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core layout surface child folder split",
        &[
            "runtime_15_ui_shared_core_layout_surface_child_folder_split_static_passed_cargo_deferred",
            "ui/tests/shared_core/layout_surface.rs",
            "ui/tests/shared_core/layout_surface/layout_measurement.rs",
            "ui/tests/shared_core/layout_surface/render_extract.rs",
            "runtime_15_ui_shared_core_layout_surface_children_are_folder_backed",
        ],
    ),
];
