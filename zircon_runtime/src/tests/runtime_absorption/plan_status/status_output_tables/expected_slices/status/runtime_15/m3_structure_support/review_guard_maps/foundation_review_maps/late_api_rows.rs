pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 late API cleanup review guard child-owner split" => Some(
            "runtime_15_late_api_cleanup_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 late API cleanup structure guard folder-backed split" => Some(
            "runtime_15_late_api_cleanup_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 late API cleanup root inventory child split" => Some(
            "runtime_15_late_api_cleanup_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 late API cleanup source status-map reconciliation" => Some(
            "runtime_15_late_api_cleanup_source_status_map_reconciliation_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
