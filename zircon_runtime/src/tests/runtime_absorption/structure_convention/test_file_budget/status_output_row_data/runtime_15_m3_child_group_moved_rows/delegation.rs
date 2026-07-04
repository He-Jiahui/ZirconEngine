use super::*;

#[test]
fn runtime_15_status_output_m3_child_group_moved_rows_are_child_owner() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let child_groups_guard = read_runtime_src(CHILD_GROUPS_GUARD_PATH);
    let moved_rows_status_inventory = read_runtime_src(ROOT_STATUSES_PATH);

    assert_contains_all(
        "status-output row-data guard mounts M3 child-group moved-row child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_m3_child_group_moved_rows.rs\"]",
            "mod runtime_15_m3_child_group_moved_rows;",
        ],
    );
    assert!(
        !child_groups_guard
            .contains("fn runtime_15_status_output_m3_child_group_moved_rows_are_child_owner"),
        "runtime_15_m3_child_groups.rs should delegate moved-row ownership checks"
    );
    assert_contains_all(
        "M3 child-group moved-row guard records this split",
        &moved_rows_status_inventory,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
        ],
    );
}

#[test]
fn runtime_15_m3_child_group_moved_rows_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let child_groups_guard = read_runtime_src(CHILD_GROUPS_GUARD_PATH);
    let moved_rows_guard = read_runtime_src(MOVED_ROWS_GUARD_PATH);
    let moved_rows_child_inventory = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let moved_rows_status_inventory = read_runtime_src(ROOT_STATUSES_PATH);
    let child_sources = moved_row_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts M3 child-group moved-row child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_m3_child_group_moved_rows.rs\"]",
            "mod runtime_15_m3_child_group_moved_rows;",
        ],
    );
    assert!(
        !child_groups_guard
            .contains("fn runtime_15_status_output_m3_child_group_moved_rows_are_child_owner"),
        "runtime_15_m3_child_groups.rs should delegate moved-row ownership checks"
    );
    assert_contains_all(
        "M3 child-group moved-row parent mounts folder-backed children",
        &moved_rows_guard,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod lock_poison_rows;",
            "mod module_convention_rows;",
            "mod review_top_rows;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_paths;",
            "mod root_source_blobs;",
            "mod root_statuses;",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "M3 child-group moved-row guard records folder-backed split anchors",
        &moved_rows_status_inventory,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in MOVED_ROW_GUARD_CHILDREN {
        assert!(
            moved_rows_child_inventory.contains(child_path),
            "M3 child-group moved-row child inventory should list child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "M3 child-group moved-row child {child_path} should define {guard_name}"
        );
    }
}
