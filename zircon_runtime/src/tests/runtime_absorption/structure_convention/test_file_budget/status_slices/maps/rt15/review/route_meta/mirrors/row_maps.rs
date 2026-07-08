use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_route_metadata_status_is_mirrored() {
    let status_rows = read_review_guard_structure_rows();
    let status_map = read_status_review_foundation_sources();
    let date_map = read_date_review_foundation_sources();

    assert_contains_all(
        "review-guard root route metadata row data",
        &status_rows,
        &[
            ROUTE_SLICE,
            ROUTE_STATUS,
            ROOT_ROUTE_METADATA_GUARD_SLICE,
            ROOT_ROUTE_METADATA_GUARD_STATUS,
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_SLICE,
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_STATUS,
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_SLICE,
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_STATUS,
            ROOT_ROUTE_METADATA_ROUTE_PATH,
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_ROUTE_PATH,
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_ROUTE_PATH,
            ROOT_ROUTE_METADATA_CHILDREN[0],
            ROOT_ROUTE_METADATA_CHILDREN[1],
            ROOT_ROUTE_METADATA_CHILDREN[2],
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[0],
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_CHILDREN[1],
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_CHILDREN[0],
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_CHILDREN[1],
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_CHILDREN[2],
            ROUTE_GUARD,
            ROOT_ROUTE_METADATA_GUARD,
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_GUARD,
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status review-guard root route metadata map",
        &status_map,
        &[
            ROUTE_SLICE,
            ROUTE_STATUS,
            ROOT_ROUTE_METADATA_GUARD_SLICE,
            ROOT_ROUTE_METADATA_GUARD_STATUS,
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_SLICE,
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_STATUS,
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_SLICE,
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_STATUS,
        ],
    );
    assert_contains_all(
        "date review-guard root route metadata map",
        &date_map,
        &[
            ROUTE_SLICE,
            ROOT_ROUTE_METADATA_GUARD_SLICE,
            ROOT_ROUTE_METADATA_ROUTE_MOUNTS_SLICE,
            ROOT_ROUTE_METADATA_STATUS_MIRROR_GUARD_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
