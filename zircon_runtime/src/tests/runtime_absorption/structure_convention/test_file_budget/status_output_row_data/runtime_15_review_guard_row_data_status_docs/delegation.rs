use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_docs_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(STATUS_DOCS_GUARD_PATH);
    let child_sources = status_doc_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts review-guard status-doc child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_review_guard_row_data_status_docs.rs\"]",
            "mod runtime_15_review_guard_row_data_status_docs;",
        ],
    );
    assert_contains_all(
        "review-guard row-data status-doc guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod row_sources;",
            "mod status_maps;",
            "mod status_mirrors;",
            STATUS_DOC_CHILD_OWNER_STATUS_NAME,
            STATUS_DOC_CHILD_OWNER_STATUS_ID,
            STATUS_DOC_CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in STATUS_DOC_GUARD_CHILDREN {
        assert!(
            guard_parent.contains(child_path),
            "review-guard row-data status-doc guard should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "review-guard row-data status-doc child {child_path} should define {guard_name}"
        );
    }
}
