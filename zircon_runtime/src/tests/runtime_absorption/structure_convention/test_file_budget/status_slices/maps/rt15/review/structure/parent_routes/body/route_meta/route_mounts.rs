use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_guard_body_is_child_owned() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{PARENT_ROUTE_GUARD_BODY_ROUTE_PATH}"
    ));
    let children = read_runtime_absorption_sources(PARENT_ROUTE_GUARD_BODY_CHILDREN);
    let route_metadata_children =
        read_runtime_absorption_sources(PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "structure-support parent-route guard body route",
        &parent,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/literal_ownership.rs\"]",
            "mod literal_ownership;",
            "#[path = \"body/route_metadata.rs\"]",
            "mod route_metadata;",
            "#[path = \"body/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STRUCTURE_SUPPORT_EXPECTED_SLICE_ROWS",
        "STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN",
        "runtime_15_structure_support_expected_slice_parent_maps_are_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "parent_routes/guard_body.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "structure-support parent-route guard body children",
        &format!("{children}\n{route_metadata_children}"),
        &[
            "runtime_15_structure_support_expected_slice_parent_maps_are_folder_backed",
            "runtime_15_structure_support_expected_slice_parent_route_literals_are_child_owned",
            "runtime_15_structure_support_expected_slice_parent_route_sources_stay_budgeted",
            "runtime_15_structure_support_expected_slice_parent_route_status_mirrors_are_synced",
            PARENT_ROUTE_GUARD_BODY_GUARD,
            PARENT_ROUTE_GUARD_BODY_ROUTE_METADATA_GUARD,
        ],
    );
}
