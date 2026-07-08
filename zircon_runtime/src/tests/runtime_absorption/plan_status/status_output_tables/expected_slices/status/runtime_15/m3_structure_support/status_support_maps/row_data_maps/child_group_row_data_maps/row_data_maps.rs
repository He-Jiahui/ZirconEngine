pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 child-groups row-data guard folder-backed split" => Some(
            "runtime_15_m3_child_groups_row_data_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups row-data status-mirror child split" => Some(
            "runtime_15_m3_child_groups_row_data_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups root inventory child split" => Some(
            "runtime_15_m3_child_groups_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups exports child split" => Some(
            "runtime_15_m3_child_groups_exports_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups inventory row-data child split" => Some(
            "runtime_15_m3_child_groups_inventory_row_data_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups owner-path budget groups folder-backed split" => Some(
            "runtime_15_m3_child_groups_owner_path_budget_groups_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-groups folder-backed source reconciliation" => Some(
            "runtime_15_m3_child_groups_folder_backed_source_reconciliation_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
