use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_guard_body_route_metadata_is_folder_backed(
) {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "structure-support parent-route guard body route metadata route owner",
        &parent,
        &[
            "#[path = \"route_meta/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"route_meta/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
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
        "STRUCTURE_SUPPORT_EXPECTED_SLICE_ROWS",
        "STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN",
        PARENT_ROUTE_GUARD_BODY_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "body/route_metadata.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "structure-support parent-route guard body route metadata children",
        &children,
        &[
            PARENT_ROUTE_GUARD_BODY_GUARD,
            "runtime_15_structure_support_expected_slice_parent_route_guard_body_route_metadata_children_stay_budgeted",
            "runtime_15_structure_support_expected_slice_parent_route_guard_body_route_metadata_docs_are_synced",
            "runtime_15_structure_support_expected_slice_parent_route_guard_body_route_metadata_status_mirrors_are_synced",
            PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_GUARD,
        ],
    );
}
