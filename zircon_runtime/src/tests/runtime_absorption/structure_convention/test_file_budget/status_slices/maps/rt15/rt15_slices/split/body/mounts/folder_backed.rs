use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_is_folder_backed(
) {
    let parent = read_runtime_15_map("split/body/route_mounts.rs");
    let children = GUARD_BODY_ROUTE_MOUNTS_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body route-mount parent",
        &parent,
        &[
            "#[path = \"mounts/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"mounts/child_owner_routes.rs\"]",
            "mod child_owner_routes;",
            "#[path = \"mounts/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"mounts/naming_boundary_routes.rs\"]",
            "mod naming_boundary_routes;",
            "#[path = \"mounts/parent_routes.rs\"]",
            "mod parent_routes;",
            "#[path = \"mounts/split_layout_routes.rs\"]",
            "mod split_layout_routes;",
            "#[path = \"mounts/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "read_runtime_15_map_parent",
        GUARD,
        ROUTE_MOUNTS_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "rt15_slices/split/body/route_mounts.rs should delegate `{moved_anchor}`"
        );
    }
    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body route-mount children",
        &children,
        &[
            "fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_is_folder_backed",
            "fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_children_stay_budgeted",
            "fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_is_folder_backed",
            "fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_status_is_synced",
        ],
    );
}
