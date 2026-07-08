pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 plugin-importer status-output guard folder-backed split" => Some(
            "runtime_15_plugin_importer_status_output_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-mirror child split" => Some(
            "runtime_15_review_guard_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data root inventory child split" => Some(
            "runtime_15_review_guard_row_data_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data aggregation guard child split" => Some(
            "runtime_15_review_guard_row_data_aggregation_guard_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
