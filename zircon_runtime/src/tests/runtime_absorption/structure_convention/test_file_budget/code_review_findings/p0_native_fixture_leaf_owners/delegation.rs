use super::*;

#[test]
fn runtime_15_p0_native_fixture_leaf_owner_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let child_inventory = read_runtime_src(P0_NATIVE_FIXTURE_ROOT_CHILD_ROWS_CHILD);
    let status_inventory = read_runtime_src(P0_NATIVE_FIXTURE_ROOT_STATUSES_CHILD);
    let child_sources = folder_backed_child_source_blob();

    assert_contains_all(
        "P0 native fixture leaf-owner structure guard delegates focused children",
        &parent,
        &[
            "#[path = \"p0_native_fixture_leaf_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"p0_native_fixture_leaf_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"p0_native_fixture_leaf_owners/leaf_ownership.rs\"]",
            "mod leaf_ownership;",
            "#[path = \"p0_native_fixture_leaf_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"p0_native_fixture_leaf_owners/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"p0_native_fixture_leaf_owners/root_statuses.rs\"]",
            "mod root_statuses;",
            "#[path = \"p0_native_fixture_leaf_owners/root_child_rows.rs\"]",
            "mod root_child_rows;",
            "#[path = \"p0_native_fixture_leaf_owners/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"p0_native_fixture_leaf_owners/root_inventory.rs\"]",
            "mod root_inventory;",
        ],
    );
    assert_contains_all(
        "P0 native fixture root status child preserves folder-backed status anchors",
        &status_inventory,
        &[],
    );
    for moved_guard in [format!("fn {GUARD}")] {
        assert!(
            !parent.contains(&moved_guard),
            "P0 native fixture leaf-owner guard `{moved_guard}` should stay in child files"
        );
    }
    for (_, child_path, guard_name) in FOLDER_BACKED_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "P0 native fixture root child inventory should include child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "P0 native fixture leaf-owner child {child_path} should define {guard_name}"
        );
    }
}
