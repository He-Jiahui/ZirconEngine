type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 production guard review-guard row-data folder-backed split",
        &[
            "runtime_15_production_guard_review_guard_row_data_folder_backed_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/base_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/moved_row_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/code_review_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/row_data_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/status_doc_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/direct_assertion_rows.rs",
            "Cargo gate deferred active Render Plan08 lane",
        ],
    ),
    (
        "Runtime 15 M3 status output review-guard row-data guard child-owner split",
        &[
            "runtime_15_status_output_review_guard_row_data_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
            "runtime_15_status_output_m3_review_guard_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 review-guard row-data status-doc guard child-owner split",
        &[
            "runtime_15_review_guard_row_data_status_docs_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_status_docs.rs",
            "runtime_15_status_output_review_guard_row_data_status_docs_are_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 review-guard row-data moved-row guard child-owner split",
        &[
            "runtime_15_review_guard_row_data_moved_rows_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_moved_rows.rs",
            "runtime_15_status_output_m3_review_guard_row_data_moved_rows_are_child_owner",
            "Cargo gate deferred",
        ],
    ),
];
