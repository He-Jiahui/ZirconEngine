use super::*;

#[test]
fn runtime_15_module_layout_child_summary_status_docs_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let status_doc_parent = read_runtime_src(CHILD_SUMMARY_STATUS_DOCS_GUARD_PATH);
    let child_inventory = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let status_inventory = read_runtime_src(ROOT_STATUSES_PATH);

    assert_contains_all(
        "status-output row-data guard mounts module-layout child-summary status-doc child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/module_layout_child_summary_status_docs.rs\"]",
            "mod module_layout_child_summary_status_docs;",
        ],
    );
    for (module_name, path, guard_name) in CHILD_SUMMARY_STATUS_DOC_CHILDREN {
        assert_contains_all(
            "module-layout child-summary status-doc parent mounts focused children",
            &status_doc_parent,
            &[
                &format!("#[path = \"module_layout_child_summary_status_docs/{module_name}.rs\"]"),
                &format!("mod {module_name};"),
            ],
        );
        assert_contains_all(
            "module-layout child-summary status-doc child inventory lists focused children",
            &child_inventory,
            &[path, guard_name],
        );
        assert_contains_all(
            "module-layout child-summary status-doc child keeps expected guard anchor",
            &read_runtime_src(path),
            &[guard_name],
        );
    }
    assert_contains_all(
        "module-layout child-summary status-doc route parent records historical and folder-backed status",
        &status_inventory,
        &[
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
}
