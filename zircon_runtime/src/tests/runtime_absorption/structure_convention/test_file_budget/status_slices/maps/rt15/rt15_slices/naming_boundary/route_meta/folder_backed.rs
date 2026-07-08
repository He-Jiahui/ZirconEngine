use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_route_metadata_is_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{STRUCTURE_NAMING_BOUNDARY_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "naming-boundary route metadata route",
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
        "STRUCTURE_NAMING_BOUNDARY_GUARD",
        "STRUCTURE_ROUTE_STATUS_MAP_PATH",
        ROUTE_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "naming_boundary/route_metadata.rs should delegate moved route metadata anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "naming-boundary route metadata children",
        &children,
        &[
            ROUTE_GUARD,
            ROUTE_METADATA_GUARD,
            "runtime_15_status_output_naming_boundary_expected_slice_route_metadata_is_child_owned",
            "runtime_15_naming_boundary_expected_slice_route_metadata_children_stay_budgeted",
            "runtime_15_naming_boundary_expected_slice_route_metadata_docs_are_synced",
            "runtime_15_naming_boundary_expected_slice_route_metadata_status_mirrors_are_synced",
        ],
    );
}
