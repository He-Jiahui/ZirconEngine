use super::*;

#[test]
fn runtime_15_review_guard_code_review_rows_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(CODE_REVIEW_ROWS_GUARD_PATH);
    let child_sources = code_review_rows_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts review-guard code-review row guard",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_review_guard_code_review_rows.rs\"]",
            "mod runtime_15_review_guard_code_review_rows;",
        ],
    );
    assert_contains_all(
        "review-guard code-review row guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod export_chain;",
            "mod row_ownership;",
            "mod root_and_children;",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "review-guard code-review row guard records folder-backed split anchors",
        &guard_parent,
        &[
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );

    for (_, child_path, guard_name) in CODE_REVIEW_ROWS_GUARD_CHILDREN {
        assert!(
            guard_parent.contains(child_path),
            "review-guard code-review row guard should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "review-guard code-review row guard child {child_path} should define {guard_name}"
        );
    }
}
