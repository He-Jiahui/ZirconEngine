use super::super::super::*;

#[test]
fn runtime_15_m3_child_group_owner_paths_are_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_m3_groups/owner_paths/m3_child_group_owner_paths.rs",
    );
    for child in [
        "group_owner_paths/root_guard_paths.rs",
        "group_owner_paths/owner_path_routes.rs",
        "group_owner_paths/plan_status_row_paths.rs",
        "group_owner_paths/folder_backed.rs",
    ] {
        assert!(
            parent.contains(child),
            "M3 child-group owner-path route parent must mount {child}"
        );
    }
    for moved_owner in [
        "M3_CHILD_GROUPS_GUARD_PATH,",
        "PRODUCTION_GUARD_RUNTIME_ROW_DATA_GUARD_PATH,",
        "TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH,",
        "PRODUCTION_GUARD_SUPPORT_EXPECTED_SLICE_GUARDS_ROWS_PATH,",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "M3 child-group owner-path route parent must not own moved budget tuple {moved_owner}"
        );
    }

    let grouped_paths: usize = super::M3_CHILD_GROUP_OWNER_PATH_GROUPS
        .iter()
        .map(|group| group.len())
        .sum();
    assert!(
        grouped_paths >= 30,
        "M3 child-group owner-path budget groups should retain the full inventory; got {grouped_paths}"
    );
}
