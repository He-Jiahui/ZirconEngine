type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 status-support row-data guard folder-backed split",
        &[
            "runtime_15_status_support_row_data_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/delegation.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/row_ownership.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/export_chain.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/status_mirrors.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/budgets.rs",
            "runtime_15_status_support_row_data_guard_is_folder_backed",
            "runtime_15_status_support_row_data_owner_is_child_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 status-support row-data status-mirror child split",
        &[
            "runtime_15_status_support_row_data_status_mirror_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/status_mirrors.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/status_mirrors/child_split_status.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/status_mirrors/historical_status.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/status_mirrors/folder_backed_status.rs",
            "runtime_15_status_support_row_data_status_mirror_children_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 status-support row-data root inventory child split",
        &[
            "runtime_15_status_support_row_data_root_inventory_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_paths.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_statuses.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_child_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_owner_paths.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_inventory.rs",
            "runtime_15_status_support_row_data_root_inventory_is_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 status-support row-data root owner paths folder-backed split",
        &[
            "runtime_15_status_support_row_data_root_owner_paths_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_owner_paths.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_owner_paths/root_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_owner_paths/row_data_and_budget.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_owner_paths/expected_slice_maps.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_owner_paths/runtime_index_anchors.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data/root_owner_paths/priority_plan_docs.rs",
            "runtime_15_status_support_row_data_root_owner_paths_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
];
