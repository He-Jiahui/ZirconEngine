use super::*;

#[test]
fn runtime_15_m2_row_data_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(M2_ROW_DATA_GUARD_PATH);
    let root_statuses = read_runtime_src(ROOT_STATUSES_PATH);
    let root_child_rows = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let guard_source = format!("{guard_parent}{root_statuses}{root_child_rows}");
    let child_sources = m2_row_data_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts M2 row-data child",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/runtime_15_m2_row_data.rs\"]",
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
            "mod root_child_rows;",
            "mod root_inventory;",
            "mod root_owner_paths;",
            "mod root_paths;",
            "mod root_statuses;",
            "pub(super) use root_child_rows::*;",
            "pub(super) use root_owner_paths::*;",
            "pub(super) use root_paths::*;",
            "pub(super) use root_statuses::*;",
        ],
    );
    assert_contains_all(
        "Runtime 15 M2 row-data root statuses preserve historical status anchors",
        &guard_source,
        &[
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
            guard_source.contains(child_path),
            "M2 row-data root child inventory should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "M2 row-data child {child_path} should define {guard_name}"
        );
    }
}
