use super::*;

#[test]
fn runtime_15_structure_route_status_date_maps_row_data_is_synced() {
    let status_rows = read_structure_support_expected_slice_rows();
    assert_contains_all(
        "structure-route map row data",
        &status_rows,
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/structure_support_rows.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/guard_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/structure_support_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/guard_rows.rs",
            GUARD_ROUTE_PATH,
            ROWS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-route map guard row data",
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
