use super::*;

#[test]
fn runtime_15_review_guard_row_data_root_and_expected_budgets_stay_focused() {
    assert_runtime_15_review_guard_row_data_budgets(&[
        (ROOT_PATHS_PATH, 80),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths/delegation.rs",
            50,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths/folder_backed.rs",
            120,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths/foundation.rs",
            50,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths/root_child_rows.rs",
            60,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths/status_outputs.rs",
            80,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths/status_support_rows.rs",
            70,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/root_paths/typed_error_rows.rs",
            60,
        ),
        (ROOT_STATUSES_PATH, 130),
        (ROOT_CHILD_ROWS_PATH, 40),
        (ROOT_CHILD_ROWS_TOP_LEVEL_CHILD_PATH, 70),
        (ROOT_CHILD_ROWS_DELEGATION_CHILD_PATH, 45),
        (ROOT_CHILD_ROWS_TYPED_ERROR_ROWS_CHILD_PATH, 70),
        (ROOT_CHILD_ROWS_AGGREGATION_CHILD_PATH, 70),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_CHILD_PATH, 40),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH, 90),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH, 120),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_BUDGETS_CHILD_PATH, 75),
        (ROOT_CHILD_ROWS_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH, 170),
        (ROOT_SOURCE_BLOBS_PATH, 125),
        (ROOT_INVENTORY_GUARD_PATH, 100),
        (RUNTIME_15_ROW_DATA_GUARD_PATH, 400),
        (MOVED_ROWS_GUARD_PATH, 400),
        (RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH, 800),
        (REVIEW_GUARD_SPLITS_PATH, 800),
    ]);

    for (_, child_path, _) in REVIEW_GUARD_ROW_DATA_AGGREGATION_CHILDREN {
        assert_runtime_15_review_guard_row_data_budgets(&[(child_path, 100)]);
    }
}
