use super::*;

#[test]
fn runtime_15_status_support_runtime_index_anchor_expected_slice_maps_row_data_is_synced() {
    let status_rows = read_status_support_expected_slice_rows();
    assert_contains_all(
        "runtime-index anchor expected-slice row data",
        &status_rows,
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/index_baseline_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/status_support_map_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/plan_status_guard_maps.rs",
            GUARD_ROUTE_PATH,
            ROWS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "runtime-index anchor expected-slice guard row data",
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
