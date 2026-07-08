use super::*;

#[test]
fn runtime_15_review_guard_code_review_expected_slice_map_rows_row_data_is_synced() {
    let status_rows = read_structure_support_expected_slice_rows();
    assert_contains_all(
        "code-review map row data",
        &status_rows,
        &[
            MAP_ROWS_SLICE,
            MAP_ROWS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review_guard_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review/direct_assertion_rows.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review/status_doc_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review_guard_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review/direct_assertion_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review/status_doc_rows.rs",
            MAP_ROWS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "code-review map rows guard row data",
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

    let status_route_map = read_status_structure_route_map_sources();
    let date_route_map = read_date_structure_route_map_sources();
    assert_contains_all(
        "code-review map structure status/date maps",
        &format!("{status_route_map}\n{date_route_map}"),
        &[
            MAP_ROWS_SLICE,
            MAP_ROWS_STATUS,
            "2026-07-07",
            MAP_ROWS_GUARD,
        ],
    );
}
