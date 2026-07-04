use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_doc_child_budgets_stay_focused() {
    for (path, budget) in [
        (REVIEW_GUARD_ROW_DATA_GUARD_PATH, 220),
        (STATUS_DOCS_GUARD_PATH, 70),
        (ROOT_PATHS_PATH, 90),
        (ROOT_STATUSES_PATH, 90),
        (ROOT_CHILD_ROWS_PATH, 120),
        (ROOT_SOURCE_BLOBS_PATH, 80),
        (ROOT_INVENTORY_GUARD_PATH, 100),
        (STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH, 260),
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < budget,
            "{path} should stay below the focused Runtime 15 review row-data status-doc guard budget of {budget} lines; got {line_count}"
        );
    }

    for (_, child_path, _) in STATUS_DOC_GUARD_CHILDREN {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 180,
            "{child_path} should stay focused after review-guard row-data status-doc folder-backed split; got {line_count} lines"
        );
    }

    for child_path in [
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_status_docs/status_mirrors/child_split_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_status_docs/status_mirrors/review_guard_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_status_docs/status_mirrors/status_doc_status.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data_status_docs/status_mirrors/folder_backed_status.rs",
    ] {
        let source = read_runtime_src(child_path);
        let line_count = source.lines().count();
        assert!(
            line_count < 100,
            "{child_path} should stay focused after review-guard status-doc status-mirror child split; got {line_count} lines"
        );
    }
}
