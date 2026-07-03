use super::*;

#[test]
fn runtime_15_lock_poison_status_row_data_children_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_lock_poison_status_row_data_children.rs",
    );
    let child_sources = lock_poison_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts lock-poison status row-data child",
        &status_output_row_data_parent,
        &[
            "#[path = \"status_output_row_data/runtime_15_lock_poison_status_row_data_children.rs\"]",
            "mod runtime_15_lock_poison_status_row_data_children;",
        ],
    );
    assert_contains_all(
        "lock-poison status row-data guard mounts folder-backed children",
        &guard_parent,
        &[
            "mod budgets;",
            "mod delegation;",
            "mod export_chain;",
            "mod row_ownership;",
            "mod status_mirrors;",
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in LOCK_POISON_STATUS_ROW_DATA_GUARD_CHILDREN {
        assert!(
            guard_parent.contains(child_path),
            "lock-poison status row-data guard should mount child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "lock-poison status row-data child {child_path} should define {guard_name}"
        );
    }
}
