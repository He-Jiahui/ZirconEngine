use super::*;

#[test]
fn runtime_15_m3_child_groups_status_docs_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let status_docs_parent = read_runtime_src(STATUS_DOCS_GUARD_PATH);
    let guard_child_inventory = [
        read_runtime_src(ROOT_CHILD_ROWS_PATH),
        read_runtime_src(ROOT_PATHS_PATH),
    ]
    .join("\n");
    let guard_status_inventory = read_runtime_src(ROOT_STATUSES_PATH);
    let child_sources = status_docs_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts M3 child-group status-doc children",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/runtime_15_m3_child_group_status_docs.rs\"]",
            "mod runtime_15_m3_child_group_status_docs;",
            "#[path = \"row_data/runtime_15_m3_child_group_status_row_docs.rs\"]",
            "mod runtime_15_m3_child_group_status_row_docs;",
        ],
    );
    assert_contains_all(
        "M3 child-group status-doc parent mounts focused children",
        &status_docs_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_paths;",
            "mod root_source_blobs;",
            "mod root_statuses;",
            "mod source_ownership;",
            "mod status_maps;",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "M3 child-group status-doc guard records folder-backed split anchors",
        &guard_status_inventory,
        &[
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
            HISTORICAL_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in STATUS_DOCS_GUARD_CHILDREN {
        assert! {
            guard_child_inventory.contains(child_path),
            "M3 child-group status-doc child inventory should list child path {child_path}"
        };
        assert! {
            child_sources.contains(guard_name),
            "M3 child-group status-doc child {child_path} should define {guard_name}"
        };
    }
}
