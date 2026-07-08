pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error source inventory guard child-owner split" => {
            Some("runtime_15_typed_error_source_inventory_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory guard folder-backed split" => {
            Some("runtime_15_typed_error_source_inventory_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory source helper child split" => {
            Some("runtime_15_typed_error_source_inventory_source_helper_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory child sources folder-backed split" => {
            Some("runtime_15_typed_error_source_inventory_child_sources_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory child sources structure guard child split" => {
            Some("runtime_15_typed_error_source_inventory_child_sources_structure_guard_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory child inventory folder-backed split" => {
            Some("runtime_15_typed_error_source_inventory_child_inventory_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory child inventory status-current child split" => {
            Some("runtime_15_typed_error_source_inventory_child_inventory_status_current_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory metadata child split" => {
            Some("runtime_15_typed_error_source_inventory_metadata_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory metadata status-current child split" => {
            Some("runtime_15_typed_error_source_inventory_metadata_status_current_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory delegation child split" => {
            Some("runtime_15_typed_error_source_inventory_delegation_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory delegation folder-backed child split" => {
            Some("runtime_15_typed_error_source_inventory_delegation_folder_backed_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error source inventory delegation folder-backed ownership child split" => {
            Some("runtime_15_typed_error_source_inventory_delegation_folder_backed_ownership_child_split_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
