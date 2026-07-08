use super::*;

#[test]
fn runtime_15_naming_boundary_expected_slice_route_metadata_status_mirrors_are_synced() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_structure_route_map_sources();
    let date_map = read_date_structure_route_map_sources();

    assert_contains_all(
        "status naming-boundary route metadata row data",
        &status_rows,
        &[
            ROUTE_SLICE,
            ROUTE_STATUS,
            "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary.rs",
            "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/sources.rs",
            "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/naming_boundary/guard_body.rs",
            STRUCTURE_NAMING_BOUNDARY_ROUTE_PATH,
            ROUTE_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status naming-boundary route metadata split row data",
        &status_rows,
        &[
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
            STRUCTURE_NAMING_BOUNDARY_ROUTE_PATH,
            ROUTE_METADATA_CHILDREN[0],
            ROUTE_METADATA_CHILDREN[1],
            ROUTE_METADATA_CHILDREN[2],
            ROUTE_METADATA_CHILDREN[3],
            ROUTE_METADATA_CHILDREN[4],
            ROUTE_METADATA_CHILDREN[5],
            ROUTE_METADATA_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status naming-boundary route metadata map",
        &status_map,
        &[
            ROUTE_SLICE,
            ROUTE_STATUS,
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
        ],
    );
    assert_contains_all(
        "date naming-boundary route metadata map",
        &date_map,
        &[
            ROUTE_SLICE,
            "Some(\"2026-07-06\")",
            ROUTE_METADATA_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
