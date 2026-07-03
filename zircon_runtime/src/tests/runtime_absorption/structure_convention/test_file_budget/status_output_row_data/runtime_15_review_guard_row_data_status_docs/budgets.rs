use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_doc_child_budgets_stay_focused() {
    for path in [REVIEW_GUARD_ROW_DATA_GUARD_PATH, STATUS_DOCS_GUARD_PATH] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 220,
            "{path} should stay below the focused Runtime 15 review row-data status-doc guard budget; got {line_count} lines"
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
