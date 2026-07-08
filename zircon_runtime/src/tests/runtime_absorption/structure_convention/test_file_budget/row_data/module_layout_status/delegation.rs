use super::*;

#[test]
fn runtime_15_module_layout_status_docs_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(MODULE_LAYOUT_STATUS_DOCS_GUARD_PATH);
    let child_inventory = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let status_inventory = read_runtime_src(ROOT_STATUSES_PATH);
    let child_sources = module_layout_status_doc_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts module-layout status-doc child",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/module_layout_status_docs.rs\"]",
            "mod module_layout_status_docs;",
        ],
    );
    assert_contains_all(
        "module-layout status-doc guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_paths;",
            "mod root_source_blobs;",
            "mod root_statuses;",
            "mod source_ownership;",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "module-layout status-doc status inventory records old and new split anchors",
        &status_inventory,
        &[
            ROW_DATA_GUARD_STATUS_NAME,
            ROW_DATA_GUARD_STATUS_ID,
            ROW_DATA_GUARD_NAME,
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
            HISTORICAL_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in MODULE_LAYOUT_STATUS_DOC_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "module-layout status-doc child inventory should list child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "module-layout status-doc child {child_path} should define {guard_name}"
        );
    }
}
