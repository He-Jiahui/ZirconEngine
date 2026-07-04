use super::*;

#[test]
fn runtime_15_review_guard_row_data_child_budgets_stay_focused() {
    for (path, budget) in [
        (REVIEW_GUARD_ROW_DATA_GUARD_PATH, 150),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/delegation.rs",
            80,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/moved_rows.rs",
            100,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/aggregation.rs",
            130,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/budgets.rs",
            90,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/status_mirrors.rs",
            120,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/typed_error_rows.rs",
            170,
        ),
        (ROOT_PATHS_PATH, 90),
        (ROOT_STATUSES_PATH, 80),
        (ROOT_CHILD_ROWS_PATH, 120),
        (ROOT_SOURCE_BLOBS_PATH, 80),
        (ROOT_INVENTORY_GUARD_PATH, 100),
        (TYPED_ERROR_ROWS_PATH, 80),
        (TYPED_ERROR_NATIVE_PLUGIN_ROWS_PATH, 130),
        (TYPED_ERROR_RUNTIME_SURFACE_ROWS_PATH, 80),
        (TYPED_ERROR_ASSET_SHADER_ROWS_PATH, 80),
        (RUNTIME_15_ROW_DATA_GUARD_PATH, 400),
        (MOVED_ROWS_GUARD_PATH, 400),
        (REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH, 800),
        (
            RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH,
            800,
        ),
        (REVIEW_GUARD_SPLITS_PATH, 800),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below its focused Runtime 15 review row-data budget of {budget}; got {line_count} lines"
        );
    }
}
