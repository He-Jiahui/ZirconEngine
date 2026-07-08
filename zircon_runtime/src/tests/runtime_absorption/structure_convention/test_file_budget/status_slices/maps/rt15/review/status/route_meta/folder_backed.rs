use super::*;

#[test]
fn runtime_15_status_support_expected_slice_route_metadata_is_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{ROUTE_METADATA_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "status-support route metadata route owner",
        &parent,
        &[
            "#[path = \"route_meta/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"route_meta/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"route_meta/paths.rs\"]",
            "mod paths;",
            "#[path = \"route_meta/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"route_meta/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STRUCTURE_REVIEW_STATUS_SUPPORT_EXPECTED_SLICE_GUARD",
        "STATUS_SUPPORT_STATUS_MAP_PATH",
        GUARD_ROUTE_METADATA_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status/route_metadata.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "status-support route metadata child tests",
        &children,
        &[
            "runtime_15_status_support_expected_slice_guard_route_metadata_is_child_owned",
            "runtime_15_status_support_expected_slice_route_metadata_children_stay_budgeted",
            "runtime_15_status_support_expected_slice_route_metadata_status_mirrors_are_registered",
            ROUTE_METADATA_GUARD,
        ],
    );
}
