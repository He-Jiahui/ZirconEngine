use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_child_budgets_stay_focused() {
    for (path, budget) in [
        (DIRECT_ASSERTION_GUARD_PATH, 70),
        (ROOT_PATHS_PATH, 100),
        (ROOT_STATUSES_PATH, 80),
        (ROOT_CHILD_ROWS_PATH, 120),
        (ROOT_SOURCE_BLOBS_PATH, 80),
        (ROOT_INVENTORY_GUARD_PATH, 100),
        (REVIEW_GUARD_ROW_DATA_AGGREGATION_PATH, 400),
        (CODE_REVIEW_ROWS_PATH, 400),
        (DIRECT_ASSERTION_ROWS_PATH, 400),
        (PLUGIN_IMPORTER_ROWS_PATH, 400),
        (STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH, 240),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the focused Runtime 15 direct-assertion row-data budget of {budget} lines; got {line_count}"
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
