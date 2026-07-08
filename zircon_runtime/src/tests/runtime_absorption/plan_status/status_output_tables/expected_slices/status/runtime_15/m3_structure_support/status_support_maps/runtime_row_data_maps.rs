pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 test file budget root-layout child split" => {
            Some("runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status output Runtime 15 row data split" => {
            Some("runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 Runtime 15 row-data guard folder-backed split" => Some(
            "runtime_15_runtime_15_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 15 row-data status-mirror child split" => Some(
            "runtime_15_runtime_15_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 15 row-data row-ownership child split" => Some(
            "runtime_15_runtime_15_row_data_row_ownership_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 Runtime 15 row-data root inventory child split" => Some(
            "runtime_15_runtime_15_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
