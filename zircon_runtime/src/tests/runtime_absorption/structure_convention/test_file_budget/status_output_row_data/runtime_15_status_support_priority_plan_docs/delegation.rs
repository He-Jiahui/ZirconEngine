use super::*;

#[test]
fn runtime_15_priority_plan_docs_row_data_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(PRIORITY_GUARD_PATH);
    let child_inventory = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let status_inventory = read_runtime_src(ROOT_STATUSES_PATH);
    let child_sources = priority_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts priority-plan-doc guard",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_status_support_priority_plan_docs.rs\"]",
            "mod runtime_15_status_support_priority_plan_docs;",
        ],
    );
    assert_contains_all(
        "priority-plan-doc guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod export_chain;",
            "mod row_sources;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_paths;",
            "mod root_source_blobs;",
            "mod root_statuses;",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "priority-plan-doc status inventory records split anchors",
        &status_inventory,
        &[
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
            HISTORICAL_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in PRIORITY_GUARD_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "priority-plan-doc guard should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "priority-plan-doc guard child {child_path} should define {guard_name}"
        );
    }
}
