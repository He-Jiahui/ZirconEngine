use super::*;

const STRUCTURE_SUPPORT_PARENT_ROUTE_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/status_metadata.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/route_inputs.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/paths/child_guard_paths.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/sources.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/guard_body.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/structure/parent_routes/route_metadata.rs",
];

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_metadata_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_SUPPORT_PARENT_ROUTE_CHILD);
    let children = read_sources(STRUCTURE_SUPPORT_PARENT_ROUTE_CHILDREN);
    let guard_body_children = read_runtime_absorption_sources(PARENT_ROUTE_GUARD_BODY_CHILDREN);
    let route_metadata_children = read_runtime_absorption_sources(PARENT_ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "structure-support parent-route mounts child-owned route metadata",
        &parent,
        &[
            "#[path = \"parent_routes/paths.rs\"]",
            "mod paths;",
            "#[path = \"parent_routes/sources.rs\"]",
            "mod sources;",
            "#[path = \"parent_routes/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"parent_routes/route_metadata.rs\"]",
            "mod route_metadata;",
            "use paths::*;",
            "use sources::*;",
        ],
    );

    for moved_anchor in [
        "const STRUCTURE_SUPPORT_PARENT_ROUTE_CHILD",
        "const STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN",
        "fn read_sources",
        "#[test]",
        "runtime_15_structure_support_expected_slice_parent_maps_are_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "parent_route_children.rs should delegate moved route metadata {moved_anchor}"
        );
    }

    assert_contains_all(
        "structure-support parent-route metadata children own moved declarations",
        &format!("{children}\n{guard_body_children}\n{route_metadata_children}"),
        &[
            "pub(in super::super) const STRUCTURE_SUPPORT_PARENT_ROUTE_CHILD",
            "pub(in super::super) const STATUS_STRUCTURE_PARENT_ROUTE_CHILDREN",
            "pub(super) fn read_sources",
            "runtime_15_structure_support_expected_slice_parent_maps_are_folder_backed",
            "runtime_15_structure_support_expected_slice_parent_route_metadata_is_child_owned",
            PARENT_ROUTE_GUARD_BODY_GUARD,
            PARENT_ROUTE_METADATA_GUARD,
        ],
    );
}
