use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_guard_body_route_metadata_is_folder_backed(
) {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{}",
        GUARD_BODY_ROUTE_CHILDREN[2]
    ));
    let children = read_runtime_absorption_sources(ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "naming-boundary guard-body route metadata route",
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
        "ROUTE_METADATA_ROWS_PATH",
        "STRUCTURE_ROUTE_STATUS_MAP_PATH",
        GUARD_BODY_ROUTE_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "naming_boundary/body/route_metadata.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "naming-boundary guard-body route metadata children",
        &children,
        &[
            GUARD_BODY_ROUTE_GUARD,
            ROUTE_METADATA_GUARD,
            "runtime_15_naming_boundary_expected_slice_guard_body_route_metadata_children_stay_budgeted",
            "runtime_15_naming_boundary_expected_slice_guard_body_route_metadata_docs_are_synced",
            "runtime_15_naming_boundary_expected_slice_guard_body_route_metadata_status_mirrors_are_synced",
        ],
    );
}
