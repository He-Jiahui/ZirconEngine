use super::*;

#[test]
fn runtime_15_f8_child_owner_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let child_blob = folder_backed_child_source_blob();

    assert_contains_all(
        "F8 structure guard parent mounts folder-backed children",
        &parent,
        &[
            "#[path = \"f8_child_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"f8_child_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"f8_child_owners/route_ownership.rs\"]",
            "mod route_ownership;",
            "#[path = \"f8_child_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            FOLDER_BACKED_SLICE,
            FOLDER_BACKED_STATUS,
        ],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            parent.contains(child_path),
            "F8 structure guard parent should inventory child path {child_path}"
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
    assert!(
        !parent.contains(&format!("fn {FOLDER_BACKED_STATUS_GUARD}")),
        "F8 status mirror guard should live in status_mirrors child"
    );
    assert!(
        !parent.contains(&format!("fn {BUDGET_GUARD}")),
        "F8 budget guard should live in budgets child"
    );
}
