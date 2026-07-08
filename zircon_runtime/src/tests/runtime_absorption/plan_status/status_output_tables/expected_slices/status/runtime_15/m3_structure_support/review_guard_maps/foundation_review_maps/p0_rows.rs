pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 P0 robustness review guard child-owner split" => Some(
            "runtime_15_p0_robustness_review_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 robustness structure guard folder-backed split" => Some(
            "runtime_15_p0_robustness_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 robustness root inventory child split" => Some(
            "runtime_15_p0_robustness_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 route ownership guard child split" => Some(
            "runtime_15_p0_route_ownership_guard_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 source status-map reconciliation" => Some(
            "runtime_15_p0_source_status_map_reconciliation_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 native fixture review guard leaf-owner split" => Some(
            "runtime_15_p0_native_fixture_review_guard_leaf_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 native fixture leaf-owner guard folder-backed split" => Some(
            "runtime_15_p0_native_fixture_leaf_owner_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 native fixture leaf-owner root inventory child split" => Some(
            "runtime_15_p0_native_fixture_leaf_owner_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 P0 native fixture source status-map reconciliation" => Some(
            "runtime_15_p0_native_fixture_source_status_map_reconciliation_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
