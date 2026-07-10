use super::*;

#[test]
fn runtime_15_naming_boundary_expected_slice_guard_body_route_metadata_status_mirrors_are_synced() {
    let status_rows = read_route_metadata_row_sources();
    let status_map = read_status_structure_route_map_sources();
    let date_map = read_date_structure_route_map_sources();

    assert_contains_all(
        "status naming-boundary guard-body route metadata row data",
        &status_rows,
        &[
            GUARD_BODY_ROUTE_SLICE,
            GUARD_BODY_ROUTE_STATUS,
            GUARD_BODY_ROUTE_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status naming-boundary guard-body route metadata split row data",
        &status_rows,
        &[
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
            ROUTE_METADATA_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status naming-boundary guard-body route metadata map",
        &status_map,
        &[
            GUARD_BODY_ROUTE_SLICE,
            GUARD_BODY_ROUTE_STATUS,
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
        ],
    );
    assert_contains_all(
        "date naming-boundary guard-body route metadata map",
        &date_map,
        &[
            GUARD_BODY_ROUTE_SLICE,
            "Some(\"2026-07-06\")",
            ROUTE_METADATA_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
