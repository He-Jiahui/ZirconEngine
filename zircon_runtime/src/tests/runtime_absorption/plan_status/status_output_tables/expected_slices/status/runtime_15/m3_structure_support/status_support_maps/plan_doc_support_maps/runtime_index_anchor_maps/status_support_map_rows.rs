pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 status-support runtime-index anchor row-data child split" => Some(
            "runtime_15_status_support_runtime_index_anchor_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 status-support runtime-index anchor expected-slice maps folder-backed split" => Some(
            "runtime_15_status_support_runtime_index_anchor_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}

// Guard: runtime_15_status_support_runtime_index_anchor_rows_are_child_owned.
// Guard: runtime_15_status_support_runtime_index_anchor_expected_slice_maps_are_folder_backed.
