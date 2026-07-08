use super::*;

#[test]
fn runtime_15_status_support_expected_slice_route_metadata_row_maps_are_registered() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_support_m3_m4_status_maps();
    let date_map = read_status_support_m3_m4_date_maps();

    assert_contains_all(
        "status-support route metadata row data",
        &status_rows,
        &[
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
            ROUTE_METADATA_STATUS_MIRRORS_SLICE,
            ROUTE_METADATA_STATUS_MIRRORS_STATUS,
            ROUTE_METADATA_ROUTE_PATH,
            ROUTE_METADATA_STATUS_MIRRORS_ROUTE_PATH,
            ROUTE_METADATA_CHILDREN[0],
            ROUTE_METADATA_CHILDREN[1],
            ROUTE_METADATA_CHILDREN[2],
            ROUTE_METADATA_CHILDREN[3],
            ROUTE_METADATA_CHILDREN[4],
            ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[0],
            ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[1],
            ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[2],
            ROUTE_METADATA_GUARD,
            ROUTE_METADATA_STATUS_MIRRORS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status-support route metadata status map",
        &status_map,
        &[
            GUARD_ROUTE_METADATA_SLICE,
            GUARD_ROUTE_METADATA_STATUS,
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
            ROUTE_METADATA_STATUS_MIRRORS_SLICE,
            ROUTE_METADATA_STATUS_MIRRORS_STATUS,
        ],
    );
    assert_contains_all(
        "status-support route metadata date map",
        &date_map,
        &[
            GUARD_ROUTE_METADATA_SLICE,
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS_MIRRORS_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
