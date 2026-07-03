use super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[(
    "Runtime 15 M3 code-review row-data owner child split",
    &[
        "runtime_15_code_review_rows_row_data_owner_child_split_static_passed_cargo_deferred",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs",
        "runtime_15_code_review_rows_row_data_owner_is_child_backed",
        "Cargo gate deferred",
    ],
),
(
    "Runtime 15 M3 typed-error structure row-data child split",
    &[
        "runtime_15_typed_error_structure_row_data_child_split_static_passed_cargo_deferred",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/top_level.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/folder_backed.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/structure_assertions.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/status_docs.rs",
        "runtime_15_review_guard_code_review_rows_child_budgets_stay_focused",
        "Cargo gate deferred",
    ],
)];
