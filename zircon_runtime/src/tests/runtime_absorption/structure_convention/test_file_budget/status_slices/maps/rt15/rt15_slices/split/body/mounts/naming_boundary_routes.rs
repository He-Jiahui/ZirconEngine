use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_naming_boundary_routes_are_child_owned(
) {
    let naming_boundary = read_runtime_15_map("naming_boundary.rs");
    let naming_boundary_children = [
        read_runtime_15_map("naming_boundary/sources.rs"),
        read_runtime_15_map("naming_boundary/guard_body.rs"),
        read_runtime_15_map("naming_boundary/body/route_mounts.rs"),
        read_runtime_15_map("naming_boundary/body/route_metadata.rs"),
        read_runtime_15_map("naming_boundary/body/route_meta/route_mounts.rs"),
        read_runtime_15_map("naming_boundary/route_metadata.rs"),
        read_runtime_15_map("naming_boundary/route_meta/route_mounts.rs"),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 naming-boundary expected-slice route child",
        &naming_boundary,
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
    assert_contains_all(
        "Runtime 15 naming-boundary expected-slice guard children",
        &naming_boundary_children,
        &[
            "fn runtime_15_status_output_naming_boundary_expected_slice_maps_are_folder_backed",
            "fn runtime_15_status_output_naming_boundary_expected_slice_route_metadata_is_child_owned",
            "fn runtime_15_status_output_naming_boundary_expected_slice_guard_body_is_child_owned",
        ],
    );
}
