use super::Slice;

pub(super) const ROWS: [Slice; 4] = [
    (
        "Runtime 15 M3 production guard status-doc row-data folder-backed split",
        &[
            "runtime_15_production_guard_status_docs_row_data_folder_backed_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs/foundation_m2_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs/child_group_status_doc_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs/child_group_status_row_doc_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs/child_group_moved_row_rows.rs",
            "Cargo gate deferred active Render Plan08/text lanes",
        ],
    ),
    (
        "Runtime 15 M3 foundation row-data status-doc guard child-owner split",
        &[
            "runtime_15_foundation_row_data_status_docs_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status_docs.rs",
            "runtime_15_status_output_foundation_row_data_status_docs_are_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 M2 row-data guard child-owner split",
        &[
            "runtime_15_m2_row_data_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_row_data.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m2_row_data.rs",
            "runtime_15_status_output_runtime_15_m2_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 M2 row-data guard folder-backed split",
        &[
            "runtime_15_m2_row_data_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/row_data/runtime_15_m2_row_data.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m2_row_data/delegation.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m2_row_data/row_ownership.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m2_row_data/status_mirrors.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m2_row_data/budgets.rs",
            "runtime_15_m2_row_data_guard_is_folder_backed",
            "runtime_15_status_output_runtime_15_m2_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
];
