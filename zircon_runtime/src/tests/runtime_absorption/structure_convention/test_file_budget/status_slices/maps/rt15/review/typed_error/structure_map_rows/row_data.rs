use super::*;

#[test]
fn runtime_15_review_guard_typed_error_structure_maps_row_data_is_synced() {
    let status_rows = read_typed_error_structure_rows();
    assert_contains_all(
        "typed-error structure expected-slice row data",
        &status_rows,
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps/top_level_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps/expected_slice_map_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps/moved_guard_absence_maps.rs",
            GUARD_ROUTE_PATH,
            ROWS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "typed-error structure guard split row data",
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

fn read_typed_error_structure_rows() -> String {
    [
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows/map_rows.rs",
    ]
    .into_iter()
    .map(read_runtime_src)
    .collect::<Vec<_>>()
    .join("\n")
}
