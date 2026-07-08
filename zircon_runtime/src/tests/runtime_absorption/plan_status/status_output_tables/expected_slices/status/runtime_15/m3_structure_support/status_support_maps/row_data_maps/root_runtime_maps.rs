pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support expected-slice row-data owner folder-backed split" => Some(
            "runtime_15_status_support_expected_slice_row_data_owner_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support row-data expected-slice maps folder-backed split" => Some(
            "runtime_15_status_support_row_data_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support row-data route expected-slice guard folder-backed split" => Some(
            "runtime_15_status_support_row_data_route_expected_slice_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status output M3 row data child-owner split" => {
            Some("runtime_15_status_output_m3_row_data_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status output row-data guard child-owner split" => {
            Some("runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 status output Runtime 15 row data split" => Some(
            "runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred",
        ),
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
        "Runtime 15 M3 Runtime 15 row-data source/status-map sync" => Some(
            "runtime_15_runtime_15_row_data_source_status_map_sync_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support anchor mirror child-owner split" => Some(
            "runtime_15_status_support_anchor_mirror_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
