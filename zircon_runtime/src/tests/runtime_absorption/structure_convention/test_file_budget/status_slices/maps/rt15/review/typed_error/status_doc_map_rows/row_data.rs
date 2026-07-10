use super::*;

#[test]
fn runtime_15_typed_error_status_doc_expected_slice_rows_row_data_is_synced() {
    let status_rows = read_review_guard_structure_rows();
    assert_contains_all(
        "review-guard structure typed-error row data",
        &status_rows,
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps/status_doc_rows.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps/status_doc_rows/base_status_doc_rows.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps/status_doc_rows/status_mirrors_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps/status_doc_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps/typed_error_maps/status_doc_rows/base_status_doc_rows.rs",
            GUARD_ROUTE_PATH,
            ROWS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "typed-error status-doc guard split row data",
        &status_rows,
        &[
            GUARD_SLICE,
            GUARD_STATUS,
            GUARD_ROUTE_PATH,
            GUARD_CHILDREN[0],
            GUARD_CHILDREN[1],
            GUARD_CHILDREN[2],
            GUARD_CHILDREN[3],
            GUARD_CHILDREN[4],
            GUARD_CHILDREN[5],
            GUARD_GUARD,
            "Cargo gate deferred",
        ],
    );
}
