use super::*;

#[test]
fn runtime_15_f8_child_owner_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let child_inventory = read_runtime_src(F8_ROOT_CHILD_ROWS_CHILD);
    let child_blob = folder_backed_child_source_blob();

    assert_contains_all(
        "F8 structure guard parent mounts folder-backed children",
        &parent,
        &[
            "#[path = \"f8_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"f8_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"f8_owners/route_ownership.rs\"]",
            "mod route_ownership;",
            "#[path = \"f8_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"f8_owners/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"f8_owners/root_statuses.rs\"]",
            "mod root_statuses;",
            "#[path = \"f8_owners/root_child_rows.rs\"]",
            "mod root_child_rows;",
            "#[path = \"f8_owners/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"f8_owners/root_inventory.rs\"]",
            "mod root_inventory;",
        ],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "F8 structure guard root child inventory should list {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "F8 child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !parent.contains(&format!("fn {GUARD}")),
        "historical F8 ownership guard should live in route_ownership child"
    );
}
