use super::*;

#[test]
fn runtime_15_m2_row_data_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(M2_ROW_DATA_GUARD_PATH);
    let child_sources = m2_row_data_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts M2 row-data child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_m2_row_data.rs\"]",
            "mod runtime_15_m2_row_data;",
        ],
    );
    assert_contains_all(
        "M2 row-data guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod row_ownership;",
            "mod status_mirrors;",
            ROW_DATA_SPLIT_STATUS_NAME,
            ROW_DATA_SPLIT_STATUS_ID,
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in M2_ROW_DATA_GUARD_CHILDREN {
        assert!(
            guard_parent.contains(child_path),
            "M2 row-data guard should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "M2 row-data child {child_path} should define {guard_name}"
        );
    }
}
