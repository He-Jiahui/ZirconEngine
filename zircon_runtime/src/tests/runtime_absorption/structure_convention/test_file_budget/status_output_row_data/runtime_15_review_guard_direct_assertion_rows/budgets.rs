use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_child_budgets_stay_focused() {
    for path in [
        DIRECT_ASSERTION_GUARD_PATH,
        REVIEW_GUARD_ROW_DATA_AGGREGATION_PATH,
        CODE_REVIEW_ROWS_PATH,
        DIRECT_ASSERTION_ROWS_PATH,
        PLUGIN_IMPORTER_ROWS_PATH,
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused Runtime 15 direct-assertion row-data budget; got {line_count} lines"
        );
    }

    for (_, child_path, _) in DIRECT_ASSERTION_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 170,
            "{child_path} should stay focused after direct-assertion folder-backed split; got {line_count} lines"
        );
    }

    for child_path in [
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows/status_mirrors/child_owner_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows/status_mirrors/child_split_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_direct_assertion_rows/status_mirrors/folder_backed_status.rs",
    ] {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 100,
            "{child_path} should stay focused after direct-assertion status-mirror child split; got {line_count} lines"
        );
    }
}
