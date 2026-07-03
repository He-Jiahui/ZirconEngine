use super::*;

#[test]
fn runtime_15_late_api_cleanup_structure_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let child_blob = folder_backed_child_source_blob();

    assert_contains_all(
        "late API cleanup structure guard parent delegates to folder-backed children",
        &parent,
        &[
            "#[path = \"late_api_cleanup_child_owners/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"late_api_cleanup_child_owners/route_ownership.rs\"]",
            "mod route_ownership;",
            "#[path = \"late_api_cleanup_child_owners/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"late_api_cleanup_child_owners/budgets.rs\"]",
            "mod budgets;",
            STRUCTURE_GUARD_OWNER,
            FOLDER_BACKED_SLICE,
            FOLDER_BACKED_STATUS,
        ],
    );
    for (_, child_path, child_guard) in FOLDER_BACKED_CHILDREN {
        assert!(
            parent.contains(child_path),
            "{STRUCTURE_GUARD_OWNER} should inventory child path {child_path}"
        );
        assert!(
            child_blob.contains(child_guard),
            "{child_path} should own guard {child_guard}"
        );
    }
    for moved_fn in [
        format!("fn {GUARD}"),
        format!("fn {FOLDER_BACKED_STATUS_GUARD}"),
        format!("fn {BUDGET_GUARD}"),
    ] {
        assert!(
            !parent.contains(&moved_fn),
            "{STRUCTURE_GUARD_OWNER} should not keep moved guard body {moved_fn}"
        );
    }
    assert_contains_all(
        "late API cleanup folder-backed children preserve route/status/budget guard names",
        &child_blob,
        &[
            GUARD,
            FOLDER_BACKED_GUARD,
            FOLDER_BACKED_STATUS_GUARD,
            BUDGET_GUARD,
        ],
    );
}
