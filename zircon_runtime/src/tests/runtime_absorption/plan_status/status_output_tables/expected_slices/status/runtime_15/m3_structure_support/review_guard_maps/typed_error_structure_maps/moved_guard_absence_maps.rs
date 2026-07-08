pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error structure moved-guard absence child-owner split" => Some(
            "runtime_15_typed_error_structure_moved_guard_absence_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split" => Some(
            "runtime_15_typed_error_moved_guard_absence_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error moved-guard absence parent-backflow child split" => Some(
            "runtime_15_typed_error_moved_guard_absence_parent_backflow_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error moved-guard absence root inventory child split" => Some(
            "runtime_15_typed_error_moved_guard_absence_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error moved-guard absence child-owner route split" => Some(
            "runtime_15_typed_error_moved_guard_absence_child_owner_route_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
