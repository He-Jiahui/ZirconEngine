use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_rows_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(DIRECT_ASSERTION_GUARD_PATH);
    let child_sources = direct_assertion_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts review-guard direct-assertion row-data child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_review_guard_direct_assertion_rows.rs\"]",
            "mod runtime_15_review_guard_direct_assertion_rows;",
        ],
    );
    assert_contains_all(
        "review-guard direct-assertion guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod export_chain;",
            "mod row_ownership;",
            "mod status_mirrors;",
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in DIRECT_ASSERTION_GUARD_CHILDREN {
        assert!(
            guard_parent.contains(child_path),
            "review-guard direct-assertion guard should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "review-guard direct-assertion child {child_path} should define {guard_name}"
        );
    }
}
