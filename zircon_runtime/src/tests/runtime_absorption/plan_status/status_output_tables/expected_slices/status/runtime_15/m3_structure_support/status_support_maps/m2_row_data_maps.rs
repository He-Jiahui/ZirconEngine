pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status output Runtime 15 M2 row data split" => Some(
            "runtime_15_status_output_runtime_15_m2_row_data_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 M2 row-data guard child-owner split" => {
            Some("runtime_15_m2_row_data_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 M2 row-data guard folder-backed split" => {
            Some("runtime_15_m2_row_data_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 M2 row-data status-mirror child split" => {
            Some("runtime_15_m2_row_data_status_mirror_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 M2 row-data root inventory child split" => {
            Some("runtime_15_m2_row_data_root_inventory_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 M2 row-data source/status-map sync" => {
            Some("runtime_15_m2_row_data_source_status_map_sync_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
