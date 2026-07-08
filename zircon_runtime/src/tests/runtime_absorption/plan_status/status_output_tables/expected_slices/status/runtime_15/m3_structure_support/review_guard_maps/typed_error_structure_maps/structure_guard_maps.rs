pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error structure guard folder-backed split" => Some(
            "runtime_15_typed_error_structure_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure guard root inventory child split" => Some(
            "runtime_15_typed_error_structure_guard_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error child-ownership guard folder-backed split" => Some(
            "runtime_15_typed_error_child_ownership_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error child-ownership root inventory child split" => Some(
            "runtime_15_typed_error_child_ownership_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
