use super::*;

#[test]
fn runtime_15_runtime_07_script_expected_slice_maps_row_data_is_synced() {
    let status_rows = read_structure_support_expected_slice_rows();
    assert_contains_all(
        "structure-support Runtime 07/script expected-slice row data",
        &status_rows,
        &[
            ROWS_SLICE,
            ROWS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps/runtime07_guard_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps/runtime07_owner_budget_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps/script_vm_runtime_maps.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/runtime07_script_maps/expected_slice_map_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/runtime07_script_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/runtime07_script_maps/script_vm_runtime_maps.rs",
            GUARD_ROUTE_PATH,
            ROWS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Runtime 07/script map guard row data",
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
