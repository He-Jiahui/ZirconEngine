type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 child-group moved-row guard child-owner split",
        &[
            "runtime_15_m3_child_group_moved_row_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows.rs",
            "runtime_15_status_output_m3_child_group_moved_rows_are_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 child-group moved-row guard folder-backed split",
        &[
            "runtime_15_m3_child_group_moved_row_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/delegation.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/lock_poison_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/module_convention_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/review_top_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/budgets.rs",
            "runtime_15_m3_child_group_moved_rows_guard_is_folder_backed",
            "runtime_15_status_output_m3_child_group_moved_rows_are_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 child-group moved-row status-mirror child split",
        &[
            "runtime_15_m3_child_group_moved_row_status_mirror_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/status_mirrors/child_split_status.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/status_mirrors/historical_status.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/status_mirrors/folder_backed_status.rs",
            "runtime_15_m3_child_group_moved_row_status_mirror_children_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 child-group moved-row root inventory child split",
        &[
            "runtime_15_m3_child_group_moved_row_root_inventory_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/root_paths.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/root_statuses.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/root_child_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/root_source_blobs.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_moved_rows/root_inventory.rs",
            "runtime_15_m3_child_group_moved_rows_root_inventory_is_child_owned",
            "Cargo gate deferred",
        ],
    ),
];
