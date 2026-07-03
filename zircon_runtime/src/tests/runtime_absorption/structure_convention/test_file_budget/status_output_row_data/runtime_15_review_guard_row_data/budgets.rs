use super::*;

#[test]
fn runtime_15_review_guard_row_data_child_budgets_stay_focused() {
    for (path, source, budget) in [
        (
            REVIEW_GUARD_ROW_DATA_GUARD_PATH,
            read_runtime_src(REVIEW_GUARD_ROW_DATA_GUARD_PATH),
            150,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/delegation.rs",
            read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/delegation.rs"),
            80,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/moved_rows.rs",
            read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/moved_rows.rs"),
            100,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/aggregation.rs",
            read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/aggregation.rs"),
            130,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/budgets.rs",
            read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/budgets.rs"),
            90,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/status_mirrors.rs",
            read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/status_mirrors.rs"),
            120,
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/typed_error_rows.rs",
            read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/typed_error_rows.rs"),
            170,
        ),
        (TYPED_ERROR_ROWS_PATH, read_runtime_src(TYPED_ERROR_ROWS_PATH), 80),
        (
            TYPED_ERROR_NATIVE_PLUGIN_ROWS_PATH,
            read_runtime_src(TYPED_ERROR_NATIVE_PLUGIN_ROWS_PATH),
            130,
        ),
        (
            TYPED_ERROR_RUNTIME_SURFACE_ROWS_PATH,
            read_runtime_src(TYPED_ERROR_RUNTIME_SURFACE_ROWS_PATH),
            80,
        ),
        (
            TYPED_ERROR_ASSET_SHADER_ROWS_PATH,
            read_runtime_src(TYPED_ERROR_ASSET_SHADER_ROWS_PATH),
            80,
        ),
        (
            RUNTIME_15_ROW_DATA_GUARD_PATH,
            read_runtime_src(RUNTIME_15_ROW_DATA_GUARD_PATH),
            400,
        ),
        (MOVED_ROWS_GUARD_PATH, read_runtime_src(MOVED_ROWS_GUARD_PATH), 400),
        (
            REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH,
            read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH),
            800,
        ),
        (
            RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH,
            read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH),
            800,
        ),
        (
            REVIEW_GUARD_SPLITS_PATH,
            read_runtime_src(REVIEW_GUARD_SPLITS_PATH),
            800,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below its focused Runtime 15 review row-data budget of {budget}; got {line_count} lines"
        );
    }
}
