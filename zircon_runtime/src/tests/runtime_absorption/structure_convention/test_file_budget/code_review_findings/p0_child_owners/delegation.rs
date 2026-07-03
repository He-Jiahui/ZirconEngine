use super::*;

#[test]
fn runtime_15_p0_robustness_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let child_blob = folder_backed_child_source_blob();

    assert_contains_all(
        "P0 robustness structure guard parent mounts folder-backed children",
        &parent,
        &[
            "#[path = \"p0_child_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"p0_child_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"p0_child_owners/route_ownership.rs\"]",
            "mod route_ownership;",
            "#[path = \"p0_child_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            FOLDER_BACKED_SLICE,
            FOLDER_BACKED_STATUS,
        ],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            parent.contains(child_path),
            "P0 robustness structure guard parent should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "P0 robustness child source blob should contain child guard {child_guard}"
        );
    }
    assert!(
        !parent.contains(&format!("fn {GUARD}")),
        "historical P0 robustness ownership guard should live in route_ownership child"
    );
    assert!(
        !parent.contains(&format!("fn {FOLDER_BACKED_STATUS_GUARD}")),
        "P0 robustness status mirror guard should live in status_mirrors child"
    );
    assert!(
        !parent.contains(&format!("fn {BUDGET_GUARD}")),
        "P0 robustness budget guard should live in budgets child"
    );
}
