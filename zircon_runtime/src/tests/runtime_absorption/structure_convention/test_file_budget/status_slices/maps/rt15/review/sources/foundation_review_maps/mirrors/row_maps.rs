use super::*;

#[test]
fn runtime_15_review_guard_foundation_status_date_maps_status_is_mirrored() {
    let status_rows = read_structure_support_expected_slice_rows();
    let status_map = read_status_review_foundation_sources();
    let date_map = read_date_review_foundation_sources();

    assert_contains_all(
        "review foundation status-date map row data",
        &status_rows,
        &[
            REVIEW_FOUNDATION_MAPS_SLICE,
            REVIEW_FOUNDATION_MAPS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/foundation_review_maps/expected_slice_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/foundation_review_maps/expected_slice_rows.rs",
            REVIEW_FOUNDATION_MAPS_GUARD,
            REVIEW_FOUNDATION_MAP_GUARD_SLICE,
            REVIEW_FOUNDATION_MAP_GUARD_STATUS,
            REVIEW_FOUNDATION_MAP_GUARD_ROUTE_PATH,
            REVIEW_FOUNDATION_MAP_GUARD_CHILDREN[0],
            REVIEW_FOUNDATION_MAP_GUARD_CHILDREN[1],
            REVIEW_FOUNDATION_MAP_GUARD_GUARD,
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_SLICE,
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_STATUS,
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_ROUTE_PATH,
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_CHILDREN[0],
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_CHILDREN[1],
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_GUARD,
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_SLICE,
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_STATUS,
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_ROUTE_PATH,
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_CHILDREN[0],
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_CHILDREN[1],
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_CHILDREN[2],
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "review foundation status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            REVIEW_FOUNDATION_MAPS_SLICE,
            REVIEW_FOUNDATION_MAPS_STATUS,
            REVIEW_FOUNDATION_MAP_GUARD_SLICE,
            REVIEW_FOUNDATION_MAP_GUARD_STATUS,
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_SLICE,
            REVIEW_FOUNDATION_ROUTE_MOUNT_GUARD_STATUS,
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_SLICE,
            REVIEW_FOUNDATION_STATUS_MIRROR_GUARD_STATUS,
            "Some(\"2026-07-06\")",
        ],
    );
}
