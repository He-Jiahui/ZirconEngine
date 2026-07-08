use super::*;

#[test]
fn runtime_15_status_support_m3_m4_expected_slice_maps_row_data_is_synced() {
    let status_rows = read_status_support_expected_slice_rows();
    assert_contains_all(
        "status-support M3/M4 expected-slice row data",
        &status_rows,
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/m4_row_data_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/status_support_guard_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/m3_row_data_maps.rs",
            GUARD_ROUTE_PATH,
            ROWS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "M3/M4 expected-slice map guard row data",
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
