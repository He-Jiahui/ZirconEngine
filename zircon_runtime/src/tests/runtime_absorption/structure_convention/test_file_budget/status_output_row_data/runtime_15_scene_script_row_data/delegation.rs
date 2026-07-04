use super::*;

#[test]
fn runtime_15_scene_script_row_data_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(SCENE_SCRIPT_ROW_DATA_GUARD_PATH);
    let guard_child_inventory = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let guard_status_inventory = read_runtime_src(ROOT_STATUSES_PATH);
    let child_sources = scene_script_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts scene-script row-data child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_scene_script_row_data.rs\"]",
            "mod runtime_15_scene_script_row_data;",
        ],
    );
    assert_contains_all(
        "scene-script row-data guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod export_chain;",
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_owner_paths;",
            "mod root_paths;",
            "mod root_statuses;",
            "mod row_ownership;",
            "mod status_mirrors;",
        ],
    );
    assert_contains_all(
        "scene-script row-data guard status inventory stays child-owned",
        &guard_status_inventory,
        &[
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in SCENE_SCRIPT_ROW_DATA_GUARD_CHILDREN {
        assert!(
            guard_child_inventory.contains(child_path),
            "scene-script row-data child inventory should list child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "scene-script row-data child {child_path} should define {guard_name}"
        );
    }
}
