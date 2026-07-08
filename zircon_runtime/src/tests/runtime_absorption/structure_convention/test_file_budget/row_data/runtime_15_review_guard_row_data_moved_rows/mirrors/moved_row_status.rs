use super::*;

#[test]
fn runtime_15_review_guard_moved_row_status_rows_are_current() {
    let status_rows = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH);
    let status_support_expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let status_support_expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );

    assert_contains_all(
        "Runtime 15 M3 production-support row data records review-guard moved-row splits",
        &status_rows,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/delegation.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/typed_error_rows.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/status_mirrors.rs",
            "runtime_15_status_output_m3_review_guard_row_data_moved_rows_are_child_owner",
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected status map records review-guard moved-row splits",
        &status_support_expected_status_map,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records review-guard moved-row splits",
        &status_support_expected_date_map,
        &[
            CHILD_OWNER_STATUS_NAME,
            "2026-06-30",
            FOLDER_BACKED_STATUS_NAME,
            "2026-07-02",
        ],
    );
}
