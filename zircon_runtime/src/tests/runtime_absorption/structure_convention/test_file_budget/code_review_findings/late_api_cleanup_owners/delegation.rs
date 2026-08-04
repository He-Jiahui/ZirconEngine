use super::*;

#[test]
fn runtime_15_late_api_cleanup_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let root_paths = read_runtime_src(LATE_API_CLEANUP_ROOT_PATHS_CHILD);
    let child_inventory = read_runtime_src(LATE_API_CLEANUP_ROOT_CHILD_ROWS_CHILD);
    let status_inventory = read_runtime_src(LATE_API_CLEANUP_ROOT_STATUSES_CHILD);
    let child_blob = folder_backed_child_source_blob();

    assert_contains_all(
        "late API cleanup structure guard parent delegates to folder-backed children",
        &parent,
        &[
            "#[path = \"late_api_cleanup_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"late_api_cleanup_owners/route_ownership.rs\"]",
            "mod route_ownership;",
            "#[path = \"late_api_cleanup_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"late_api_cleanup_owners/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"late_api_cleanup_owners/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"late_api_cleanup_owners/root_statuses.rs\"]",
            "mod root_statuses;",
            "#[path = \"late_api_cleanup_owners/root_child_rows.rs\"]",
            "mod root_child_rows;",
            "#[path = \"late_api_cleanup_owners/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"late_api_cleanup_owners/root_inventory.rs\"]",
            "mod root_inventory;",
        ],
    );
    assert_contains_all(
        "late API cleanup root path/status children preserve parent and folder-backed anchors",
        &(root_paths + "\n" + &status_inventory),
        &[STRUCTURE_GUARD_OWNER],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "late API cleanup root child inventory should list {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "{child_path} should own guard {child_guard}"
        );
    }
    for moved_fn in [format!("fn {GUARD}")] {
        assert!(
            !parent.contains(&moved_fn),
            "{STRUCTURE_GUARD_OWNER} should not keep moved guard body {moved_fn}"
        );
    }
    assert_contains_all(
        "late API cleanup folder-backed children preserve route/status/budget guard names",
        &child_blob,
        &[GUARD],
    );
}
