use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_typed_error_route_metadata_status_mirrors_are_synced() {
    let status_rows = read_status_support_expected_slice_rows();
    let status_map = read_status_review_typed_error_sources();
    let date_map = read_date_review_typed_error_sources();

    assert_contains_all(
        "status typed-error expected-slice route metadata row data",
        &status_rows,
        &[
            TYPED_ERROR_ROUTE_SLICE,
            TYPED_ERROR_ROUTE_STATUS,
            "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error_expected_slice.rs",
            "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/sources.rs",
            "structure_convention/test_file_budget/status_slices/maps/rt15/review/typed_error/guard_body.rs",
            ROUTE_METADATA_ROUTE_PATH,
            TYPED_ERROR_ROUTE_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status typed-error expected-slice route metadata split row data",
        &status_rows,
        &[
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
            ROUTE_METADATA_ROUTE_PATH,
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
        "status typed-error expected-slice route metadata map",
        &status_map,
        &[
            TYPED_ERROR_ROUTE_SLICE,
            TYPED_ERROR_ROUTE_STATUS,
            ROUTE_METADATA_SLICE,
            ROUTE_METADATA_STATUS,
        ],
    );
    assert_contains_all(
        "date typed-error expected-slice route metadata map",
        &date_map,
        &[
            TYPED_ERROR_ROUTE_SLICE,
            ROUTE_METADATA_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
