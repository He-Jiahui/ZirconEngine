use super::*;

#[test]
fn runtime_15_m3_child_groups_row_data_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(M3_CHILD_GROUPS_GUARD_PATH);
    let child_sources = m3_child_group_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts M3 child-groups guard",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_m3_child_groups.rs\"]",
            "mod runtime_15_m3_child_groups;",
        ],
    );
    assert_contains_all(
        "M3 child-groups guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod exports;",
            "mod row_ownership;",
            "mod status_mirrors;",
            HISTORICAL_CHILD_OWNER_STATUS_NAME,
            HISTORICAL_CHILD_OWNER_STATUS_ID,
            HISTORICAL_CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in M3_CHILD_GROUP_GUARD_CHILDREN {
        assert! {
            guard_parent.contains(child_path),
            "M3 child-groups guard should mount child path {child_path}"
        };
        assert! {
            child_sources.contains(guard_name),
            "M3 child-groups child {child_path} should define {guard_name}"
        };
    }
}
