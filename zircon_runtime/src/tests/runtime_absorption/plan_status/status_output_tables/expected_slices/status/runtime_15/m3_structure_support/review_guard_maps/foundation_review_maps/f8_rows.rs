pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 F8 API convergence review guard child-owner split" => Some(
            "runtime_15_f8_api_convergence_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 child-owner structure guard folder-backed split" => Some(
            "runtime_15_f8_child_owner_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 child-owner root inventory child split" => Some(
            "runtime_15_f8_child_owner_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 route ownership guard child split" => Some(
            "runtime_15_f8_route_ownership_guard_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 child-owner source status-map reconciliation" => Some(
            "runtime_15_f8_child_owner_source_status_map_reconciliation_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 F8 descriptor review guard child-owner split" => Some(
            "runtime_15_f8_descriptor_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
