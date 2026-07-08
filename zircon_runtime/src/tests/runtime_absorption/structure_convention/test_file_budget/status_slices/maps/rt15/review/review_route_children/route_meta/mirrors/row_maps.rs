use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_route_metadata_row_maps_are_mirrored() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_review_foundation_sources();
    let date_map = read_date_review_foundation_sources();

    assert_contains_all(
        "review-route metadata status rows",
        &status_rows,
        &[
            REVIEW_ROUTE_METADATA_SLICE,
            REVIEW_ROUTE_METADATA_STATUS,
            REVIEW_ROUTE_METADATA_GUARD_SLICE,
            REVIEW_ROUTE_METADATA_GUARD_STATUS,
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_SLICE,
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_STATUS,
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_SLICE,
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_STATUS,
            REVIEW_ROUTE_METADATA_ROUTE_PATH,
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_ROUTE_PATH,
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_ROUTE_PATH,
            REVIEW_ROUTE_METADATA_CHILDREN[0],
            REVIEW_ROUTE_METADATA_CHILDREN[1],
            REVIEW_ROUTE_METADATA_CHILDREN[2],
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[0],
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[1],
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[0],
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[1],
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_CHILDREN[2],
            REVIEW_ROUTE_METADATA_GUARD,
            REVIEW_ROUTE_METADATA_GUARD_GUARD,
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_GUARD,
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status expected-slice review maps",
        &status_map,
        &[
            REVIEW_ROUTE_METADATA_SLICE,
            REVIEW_ROUTE_METADATA_STATUS,
            REVIEW_ROUTE_METADATA_GUARD_SLICE,
            REVIEW_ROUTE_METADATA_GUARD_STATUS,
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_SLICE,
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_STATUS,
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_SLICE,
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_STATUS,
        ],
    );
    assert_contains_all(
        "date expected-slice review maps",
        &date_map,
        &[
            REVIEW_ROUTE_METADATA_SLICE,
            REVIEW_ROUTE_METADATA_GUARD_SLICE,
            REVIEW_ROUTE_METADATA_ROUTE_MOUNTS_SLICE,
            REVIEW_ROUTE_METADATA_STATUS_MIRRORS_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
