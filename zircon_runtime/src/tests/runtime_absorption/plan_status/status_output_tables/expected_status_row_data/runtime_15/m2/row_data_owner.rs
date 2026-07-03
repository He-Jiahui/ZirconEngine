type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M2 row-data owner child split",
        &[
            "runtime_15_m2_row_data_owner_child_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/core_scene_asset_dynamic.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/render_graphics.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2/ui_platform_editor.rs",
            "runtime_15_m2_row_data_owner_is_child_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 M2 row-data children guard folder-backed split",
        &[
            "runtime_15_m2_row_data_children_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/delegation.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/row_ownership.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m2_row_data_children/budgets.rs",
            "runtime_15_m2_row_data_children_guard_is_folder_backed",
            "runtime_15_m2_row_data_owner_is_child_backed",
            "Cargo gate deferred",
        ],
    ),
];
