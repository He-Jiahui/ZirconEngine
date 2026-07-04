type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 foundation row-data 73-row docs sync",
        &[
            "runtime_15_foundation_row_data_71_row_docs_sync_static_passed_cargo_deferred",
            "21/23/18/11",
            "73 Runtime 15 foundation status rows",
            "runtime_15_foundation_row_data_docs_record_current_row_count",
        ],
    ),
    (
        "Runtime 15 M3 foundation row-data stale-count prose guard",
        &[
            "runtime_15_foundation_row_data_stale_count_prose_guard_static_passed_cargo_deferred",
            "runtime_15_foundation_row_data_docs_record_current_row_count",
            "stale Runtime 15 foundation row-count prose",
            "current 73-row foundation row-data facts",
            "73 Runtime 15 foundation status rows",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 row data split",
        &[
            "runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "runtime_15_status_output_runtime_15_row_data_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 15 row-data guard folder-backed split",
        &[
            "runtime_15_runtime_15_row_data_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/delegation.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/budgets.rs",
            "runtime_15_runtime_15_row_data_guard_is_folder_backed",
            "runtime_15_status_output_runtime_15_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 15 row-data status-mirror child split",
        &[
            "runtime_15_runtime_15_row_data_status_mirror_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/status_mirrors.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/status_mirrors/child_split_status.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/status_mirrors/historical_status.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/status_mirrors/folder_backed_status.rs",
            "runtime_15_runtime_15_row_data_status_mirror_children_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 15 row-data row-ownership child split",
        &[
            "runtime_15_runtime_15_row_data_row_ownership_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/group_exports.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/foundation_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/status_support.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/row_ownership/owner_budgets.rs",
            "runtime_15_runtime_15_row_data_row_ownership_children_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 Runtime 15 row-data root inventory child split",
        &[
            "runtime_15_runtime_15_row_data_root_inventory_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/root_paths.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/root_statuses.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/root_child_rows.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/root_owner_paths.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data/root_inventory.rs",
            "runtime_15_runtime_15_row_data_root_inventory_is_child_owned",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 foundation row data split",
        &[
            "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
            "runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 foundation row-data topic child-owner split",
        &[
            "runtime_15_foundation_row_data_topic_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_runtime_rows.rs",
            "runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 status output Runtime 15 M2 row data split",
        &[
            "runtime_15_status_output_runtime_15_m2_row_data_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
            "runtime_15_status_output_runtime_15_m2_row_data_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 status-support row-data-and-budget child split",
        &[
            "runtime_15_status_support_row_data_and_budget_child_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/runtime_row_data.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/hub_editor_support.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/render_shader_support.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/m3_m4_row_data.rs",
            "runtime_15_status_support_row_data_and_budget_children_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
];
