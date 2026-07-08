use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_metadata_is_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{PARENT_ROUTE_METADATA_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(PARENT_ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "structure-support parent-route route metadata route owner",
        &parent,
        &[
            "#[path = \"route_meta/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"route_meta/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"route_meta/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"route_meta/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"route_meta/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );

    for moved_anchor in [
        "#[test]",
        "STRUCTURE_SUPPORT_PARENT_ROUTE_CHILDREN",
        "STRUCTURE_SUPPORT_EXPECTED_SLICE_ROWS",
        "STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN",
        PARENT_ROUTE_METADATA_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "parent_routes/route_metadata.rs should delegate moved anchor {moved_anchor}"
        );
    }

    assert_contains_all(
        "structure-support parent-route route metadata children",
        &children,
        &[
            "runtime_15_structure_support_expected_slice_parent_route_metadata_sources_stay_budgeted",
            "runtime_15_structure_support_expected_slice_parent_route_metadata_is_child_owned",
            "runtime_15_structure_support_expected_slice_parent_route_metadata_docs_are_synced",
            "runtime_15_structure_support_expected_slice_parent_route_metadata_status_mirrors_are_synced",
            PARENT_ROUTE_METADATA_GUARD,
        ],
    );
}
