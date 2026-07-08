use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_guard_body_is_child_owned() {
    let parent = read_runtime_src(&format!("tests/runtime_absorption/{GUARD_BODY_ROUTE_PATH}"));
    let children = read_runtime_absorption_sources(GUARD_BODY_ROUTE_CHILDREN);
    let route_metadata_children = read_runtime_absorption_sources(ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "naming-boundary guard body route parent",
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
        "STATUS_CHILD_PATHS",
        "TOP_LEVEL_SUPPORT_ROWS_PATH",
        "runtime_15_status_output_naming_boundary_expected_slice_maps_are_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "naming_boundary/guard_body.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "naming-boundary guard body child-owned tests",
        &format!("{children}\n{route_metadata_children}"),
        &[
            "runtime_15_status_output_naming_boundary_expected_slice_maps_are_folder_backed",
            "runtime_15_status_output_naming_boundary_expected_slice_literals_are_child_owned",
            "runtime_15_status_output_naming_boundary_expected_slice_sources_stay_budgeted",
            "runtime_15_status_output_naming_boundary_expected_slice_status_mirrors_are_registered",
            GUARD_BODY_ROUTE_GUARD,
            ROUTE_METADATA_GUARD,
        ],
    );
}
