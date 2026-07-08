use super::*;

#[test]
fn runtime_15_m3_child_groups_row_data_guard_is_folder_backed() {
    let status_output_row_data_parent = read_runtime_src(STATUS_OUTPUT_ROW_DATA_PARENT_PATH);
    let guard_parent = read_runtime_src(M3_CHILD_GROUPS_GUARD_PATH);
    let guard_child_inventory = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let guard_status_inventory = [
        read_runtime_src(ROOT_STATUSES_PATH),
        m3_child_group_core_status_source_blob(),
        read_runtime_src(ROOT_STATUSES_UI_PATH),
    ]
    .join("\n");
    let child_sources = m3_child_group_guard_child_source_blob();

    assert_contains_all(
        "status-output row-data guard mounts M3 child-groups guard",
        &status_output_row_data_parent,
        &[
            "#[path = \"row_data/runtime_15_m3_child_groups.rs\"]",
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
            "mod module_convention_status;",
            "mod production_guard_runtime_row_data;",
            "mod production_guard_support;",
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
        "M3 child-groups status inventory owns split names",
        &guard_status_inventory,
        &[
            HISTORICAL_CHILD_OWNER_STATUS_NAME,
            HISTORICAL_CHILD_OWNER_STATUS_ID,
            HISTORICAL_CHILD_OWNER_GUARD_NAME,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
            FOLDER_BACKED_GUARD_NAME,
            PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_STATUS_NAME,
            PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_STATUS_ID,
            PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_GUARD_NAME,
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_ID,
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
            ROOT_INVENTORY_CHILD_SPLIT_STATUS_NAME,
            ROOT_INVENTORY_CHILD_SPLIT_STATUS_ID,
            ROOT_INVENTORY_CHILD_SPLIT_GUARD_NAME,
            EXPORTS_CHILD_SPLIT_STATUS_NAME,
            EXPORTS_CHILD_SPLIT_STATUS_ID,
            EXPORTS_CHILD_SPLIT_GUARD_NAME,
        ],
    );
    for (_, child_path, guard_name) in M3_CHILD_GROUP_GUARD_CHILDREN {
        assert! {
            guard_child_inventory.contains(child_path),
            "M3 child-groups child inventory should list child path {child_path}"
        };
        assert! {
            child_sources.contains(guard_name),
            "M3 child-groups child {child_path} should define {guard_name}"
        };
    }
}
