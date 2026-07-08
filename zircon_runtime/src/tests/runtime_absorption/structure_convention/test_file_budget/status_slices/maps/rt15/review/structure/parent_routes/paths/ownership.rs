use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_paths_are_child_owned() {
    let paths_parent = include_str!("../paths.rs");
    let status_metadata = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/status_metadata.rs",
    );
    let route_inputs = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/route_inputs.rs",
    );
    let child_guard_paths = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/child_guard_paths.rs",
    );

    assert_contains_all(
        "structure-support parent-route paths delegates child owners",
        paths_parent,
        &[
            "#[path = \"paths/child_guard_paths.rs\"]",
            "mod child_guard_paths;",
            "#[path = \"paths/ownership.rs\"]",
            "mod ownership;",
            "#[path = \"paths/route_inputs.rs\"]",
            "mod route_inputs;",
            "#[path = \"paths/status_metadata.rs\"]",
            "mod status_metadata;",
            "pub(super) use child_guard_paths::*;",
            "pub(super) use route_inputs::*;",
            "pub(super) use status_metadata::*;",
        ],
    );

    for parent_owned_literal in [
        "pub(super) const STRUCTURE_SUPPORT_PARENT_ROUTE_CHILD: &str",
        "pub(super) const PARENT_ROUTE_GUARD_BODY_SLICE: &str",
        "pub(super) const PARENT_ROUTE_METADATA_SLICE: &str",
        "pub(super) const STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN: &[&str]",
        "pub(super) const DATE_STRUCTURE_PARENT_ROUTE_CHILDREN: &[&str]",
        "pub(super) const PARENT_ROUTE_GUARD_BODY_ROUTE_PATH: &str",
        "pub(super) const PARENT_ROUTE_METADATA_ROUTE_PATH: &str",
    ] {
        assert!(
            !paths_parent.contains(parent_owned_literal),
            "structure-support parent-route paths should delegate {parent_owned_literal}"
        );
    }

    assert_contains_all(
        "status metadata owns slice/status constants",
        &status_metadata,
        &[
            "STRUCTURE_SUPPORT_PARENT_ROUTE_CHILD",
            "PARENT_ROUTE_GUARD_BODY_SLICE",
            "PARENT_ROUTE_METADATA_SLICE",
            "PARENT_ROUTE_METADATA_FRAMEWORKS_STATUS",
            "pub(in super::super) const PARENT_ROUTE_PATHS_SLICE",
        ],
    );
    assert_contains_all(
        "route inputs own status/date parent route children",
        &route_inputs,
        &[
            "STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN",
            "DATE_STRUCTURE_PARENT_ROUTE_CHILDREN",
            "pub(in super::super) const STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN",
            "runtime07_script_maps.rs",
            "plugin_export_gameplay_maps.rs",
        ],
    );
    assert_contains_all(
        "child guard paths own nested route path lists",
        &child_guard_paths,
        &[
            "PARENT_ROUTE_GUARD_BODY_ROUTE_PATH",
            "PARENT_ROUTE_GUARD_BODY_CHILDREN",
            "PARENT_ROUTE_METADATA_ROUTE_PATH",
            "PARENT_ROUTE_METADATA_CHILDREN",
            "pub(in super::super) const PARENT_ROUTE_GUARD_BODY_ROUTE_PATH",
        ],
    );
}
