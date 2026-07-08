use super::*;

#[test]
fn runtime_15_structure_route_status_date_maps_are_folder_backed() {
    let status_parent = read_runtime_src(STATUS_STRUCTURE_ROUTE_MAP);
    let date_parent = read_runtime_src(DATE_STRUCTURE_ROUTE_MAP);
    let status_maps = read_status_structure_route_map_sources();
    let date_maps = read_date_structure_route_map_sources();

    for (label, parent) in [
        ("status structure-route parent", status_parent.as_str()),
        ("date structure-route parent", date_parent.as_str()),
    ] {
        assert_contains_all(
            label,
            parent,
            &[
                "#[path = \"structure_route_maps/structure_support_rows.rs\"]",
                "mod structure_support_rows;",
                "#[path = \"structure_route_maps/review_guard_rows.rs\"]",
                "mod review_guard_rows;",
                "#[path = \"structure_route_maps/naming_boundary_rows.rs\"]",
                "mod naming_boundary_rows;",
                "#[path = \"structure_route_maps/core_route_rows.rs\"]",
                "mod core_route_rows;",
                "#[path = \"structure_route_maps/guard_rows.rs\"]",
                "mod guard_rows;",
            ],
        );
        for moved in [
            "Runtime 15 M3 structure-support expected-slice guard folder-backed split",
            "Runtime 15 M3 naming-boundary expected-slice parent maps folder-backed split",
            "Runtime 15 M3 root-surface route-owner split",
            "Runtime 15 M3 generated-code guard folder-backed split",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate structure-route row {moved}"
            );
        }
    }

    assert_contains_all(
        "status structure-route map children",
        &status_maps,
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "runtime_15_structure_support_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_naming_boundary_expected_slice_parent_maps_folder_backed_static_passed_cargo_deferred",
            "runtime_15_root_surface_route_owner_split_static_passed_cargo_deferred",
            "runtime_15_generated_code_guard_folder_backed_static_passed_cargo_deferred",
            ROWS_GUARD,
        ],
    );
    assert_contains_all(
        "date structure-route map children",
        &date_maps,
        &[
            ROWS_SLICE,
            "Some(\"2026-07-07\")",
            "Some(\"2026-07-05\")",
            "Some(\"2026-07-06\")",
        ],
    );
}
