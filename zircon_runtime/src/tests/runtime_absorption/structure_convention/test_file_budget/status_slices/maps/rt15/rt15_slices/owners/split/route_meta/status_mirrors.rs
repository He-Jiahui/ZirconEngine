use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_guard_route_metadata_status_mirrors_are_synced(
) {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_support_status_map_sources();
    let date_map = read_status_support_date_map_sources();

    assert_contains_all(
        "status child-owner split-layout route metadata row data",
        &status_rows,
        &[
            ROUTE_SLICE,
            ROUTE_STATUS,
            SPLIT_LAYOUT_PATH,
            SPLIT_LAYOUT_SOURCES_PATH,
            SPLIT_LAYOUT_GUARD_BODY_PATH,
            SPLIT_LAYOUT_ROUTE_METADATA_PATH,
            SPLIT_LAYOUT_STATUS_MIRRORS_PATH,
            ROUTE_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status child-owner split-layout route metadata split row data",
        &status_rows,
        &[
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
            SPLIT_LAYOUT_ROUTE_METADATA_PATH,
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
        "status child-owner split-layout route metadata map",
        &status_map,
        &[
            ROUTE_SLICE,
            ROUTE_STATUS,
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
        ],
    );
    assert_contains_all(
        "date child-owner split-layout route metadata map",
        &date_map,
        &[
            ROUTE_SLICE,
            "Some(\"2026-07-06\")",
            ROUTE_METADATA_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
