pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 child-group moved-row guard child-owner split" => Some(
            "runtime_15_m3_child_group_moved_row_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row guard folder-backed split" => Some(
            "runtime_15_m3_child_group_moved_row_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row status-mirror child split" => Some(
            "runtime_15_m3_child_group_moved_row_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 child-group moved-row root inventory child split" => Some(
            "runtime_15_m3_child_group_moved_row_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
