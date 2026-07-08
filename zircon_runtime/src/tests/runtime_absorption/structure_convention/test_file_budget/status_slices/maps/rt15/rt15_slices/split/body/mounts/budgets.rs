use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_route_mounts_children_stay_budgeted(
) {
    for (path, source, limit) in [
        (
            "split/body/route_mounts.rs",
            read_runtime_15_map("split/body/route_mounts.rs"),
            25usize,
        ),
        (
            "mounts/budgets.rs",
            read_runtime_15_map("split/body/mounts/budgets.rs"),
            60,
        ),
        (
            "mounts/child_owner_routes.rs",
            read_runtime_15_map("split/body/mounts/child_owner_routes.rs"),
            45,
        ),
        (
            "mounts/folder_backed.rs",
            read_runtime_15_map("split/body/mounts/folder_backed.rs"),
            70,
        ),
        (
            "mounts/naming_boundary_routes.rs",
            read_runtime_15_map("split/body/mounts/naming_boundary_routes.rs"),
            75,
        ),
        (
            "mounts/parent_routes.rs",
            read_runtime_15_map("split/body/mounts/parent_routes.rs"),
            45,
        ),
        (
            "mounts/split_layout_routes.rs",
            read_runtime_15_map("split/body/mounts/split_layout_routes.rs"),
            50,
        ),
        (
            "mounts/status_mirrors.rs",
            read_runtime_15_map("split/body/mounts/status_mirrors.rs"),
            95,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= limit,
            "{path} should stay below the route-mounts budget {limit}; got {line_count}"
        );
    }
}
