use super::*;

#[test]
fn runtime_15_m4_row_data_children_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(M4_ROW_DATA_CHILDREN_GUARD_PATH);
    let child_sources = m4_row_data_children_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts M4 row-data children guard",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_m4_row_data_children.rs\"]",
            "mod runtime_15_m4_row_data_children;",
        ],
    );
    assert_contains_all(
        "M4 row-data children guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod row_ownership;",
            "mod status_mirrors;",
            ROW_DATA_OWNER_STATUS_NAME,
            ROW_DATA_OWNER_STATUS_ID,
            ROW_DATA_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in M4_ROW_DATA_CHILDREN_GUARD_CHILDREN {
        assert!(
            guard_parent.contains(child_path),
            "M4 row-data children guard should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "M4 row-data children guard child {child_path} should define {guard_name}"
        );
    }
}
