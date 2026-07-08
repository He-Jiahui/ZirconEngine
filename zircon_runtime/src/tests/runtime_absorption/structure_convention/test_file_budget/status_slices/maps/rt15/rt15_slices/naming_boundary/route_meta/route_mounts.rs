use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_route_metadata_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_NAMING_BOUNDARY_GUARD);
    let children = read_runtime_sources(STRUCTURE_NAMING_BOUNDARY_GUARD_CHILDREN);
    let guard_body_route =
        read_runtime_src(&format!("tests/runtime_absorption/{GUARD_BODY_ROUTE_PATH}"));
    let guard_body_children = read_runtime_absorption_sources(GUARD_BODY_ROUTE_CHILDREN);
    let source_children = read_runtime_absorption_sources(SOURCES_CHILDREN);
    let route_metadata_children = read_runtime_absorption_sources(ROUTE_METADATA_CHILDREN);

    assert_contains_all(
        "naming-boundary expected-slice parent mounts child-owned route metadata",
        &parent,
        &[
            "#[path = \"naming_boundary/sources.rs\"]",
            "mod sources;",
            "#[path = \"naming_boundary/guard_body.rs\"]",
            "mod guard_body;",
            "#[path = \"naming_boundary/route_metadata.rs\"]",
            "mod route_metadata;",
            "use sources::*;",
        ],
    );

    for moved_anchor in [
        "const SLICE",
        "fn read_status_support_expected_slice_rows",
        "#[test]",
        "runtime_15_status_output_naming_boundary_expected_slice_maps_are_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "naming_boundary.rs should delegate moved route metadata {moved_anchor}"
        );
    }

    assert_contains_all(
        "naming-boundary expected-slice children own moved declarations",
        &format!(
            "{children}\n{source_children}\n{guard_body_route}\n{guard_body_children}\n{route_metadata_children}"
        ),
        &[
            "const SLICE",
            "pub(in super::super) fn read_status_support_expected_slice_rows",
            "runtime_15_status_output_naming_boundary_expected_slice_maps_are_folder_backed",
            ROUTE_GUARD,
            GUARD_BODY_ROUTE_GUARD,
        ],
    );
}
