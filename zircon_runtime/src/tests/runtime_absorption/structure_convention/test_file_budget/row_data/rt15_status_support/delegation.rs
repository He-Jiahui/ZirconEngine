use super::*;

#[test]
fn runtime_15_status_support_row_data_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(STATUS_SUPPORT_ROW_DATA_GUARD_PATH);
    let guard_child_inventory = format!(
        "{}\n{}\n{}",
        read_runtime_src(ROOT_PATHS_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_PATH),
        read_runtime_src(ROOT_GUARD_CHILDREN_PATH)
    );
    let guard_status_inventory = read_runtime_src(ROOT_STATUSES_PATH);
    let child_sources = status_support_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts status-support row-data child",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/runtime_15_status_support_row_data.rs\"]",
            "mod runtime_15_status_support_row_data;",
        ],
    );
    assert_contains_all(
        "status-support row-data guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod anchor_mirror;",
            "mod budgets;",
            "mod delegation;",
            "mod export_chain;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_owner_paths;",
            "mod root_paths;",
            "mod root_statuses;",
            "mod row_data_and_budget;",
            "mod row_ownership;",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "status-support row-data guard status inventory stays child-owned",
        &guard_status_inventory,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
            ANCHOR_MIRROR_CHILD_SPLIT_STATUS_NAME,
            ANCHOR_MIRROR_CHILD_SPLIT_STATUS_ID,
            ANCHOR_MIRROR_CHILD_SPLIT_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in STATUS_SUPPORT_ROW_DATA_GUARD_CHILDREN {
        assert!(
            guard_child_inventory.contains(child_path),
            "status-support row-data child inventory should list child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "status-support row-data child {child_path} should define {guard_name}"
        );
    }
}
