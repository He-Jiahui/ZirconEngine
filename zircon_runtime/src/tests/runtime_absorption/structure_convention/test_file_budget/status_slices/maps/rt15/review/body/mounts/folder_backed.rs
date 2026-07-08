use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_guard_body_route_mounts_is_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{ROUTE_MOUNTS_ROUTE_PATH}"
    ));
    let children = ROUTE_MOUNTS_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "review guard root guard-body route-mount route owner",
        &parent,
        &[
            "#[path = \"mounts/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"mounts/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"mounts/moved_children.rs\"]",
            "mod moved_children;",
            "#[path = \"mounts/parent_routes.rs\"]",
            "mod parent_routes;",
            "#[path = \"mounts/paths.rs\"]",
            "mod paths;",
            "#[path = \"mounts/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "let structure_parent =",
        "runtime_15_structure_support_expected_slice_maps_are_child_owners",
        "Runtime 15 M3 status-support expected-slice map child split",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "body/route_mounts.rs should delegate route-mount body {moved_anchor}"
        );
    }
    assert_contains_all(
        "review guard root guard-body route-mount children",
        &children,
        &[
            "runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned",
            "runtime_15_review_guard_expected_slice_structure_guard_moved_tests_stay_child_owned",
            "runtime_15_review_guard_expected_slice_root_guard_body_route_mounts_sources_stay_budgeted",
            "runtime_15_review_guard_expected_slice_root_guard_body_route_mounts_status_is_mirrored",
            ROUTE_MOUNTS_SLICE,
            ROUTE_MOUNTS_STATUS,
            ROUTE_MOUNTS_GUARD,
        ],
    );
}
