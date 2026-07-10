type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M4 row-data owner child split",
        &[
            "runtime_15_m4_row_data_owner_child_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/core_rhi_dynamic.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/asset_scene_render.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_text_template.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4/ui_surface_plugin.rs",
            "runtime_15_m4_row_data_owner_is_child_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 M4 row-data children guard folder-backed split",
        &[
            "runtime_15_m4_row_data_children_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/delegation.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/row_ownership.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/status_mirrors.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/budgets.rs",
            "runtime_15_m4_row_data_children_guard_is_folder_backed",
            "runtime_15_m4_row_data_owner_is_child_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 M4 row-data children status-mirror child split",
        &[
            "runtime_15_m4_row_data_children_status_mirror_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/status_mirrors.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/status_mirrors/child_split_status.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/status_mirrors/historical_status.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/status_mirrors/folder_backed_status.rs",
            "runtime_15_m4_row_data_children_status_mirror_children_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 M4 row-data children root inventory child split",
        &[
            "runtime_15_m4_row_data_children_root_inventory_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_paths.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_statuses.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_child_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_owner_paths.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/root_inventory.rs",
            "runtime_15_m4_row_data_children_root_inventory_is_child_owned",
            "Cargo gate deferred",
        ],
    ),
];
