pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split" => {
            Some("runtime_15_typed_error_native_plugin_loader_structure_guard_child_owner_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error native plugin loader structure guard folder-backed split" => {
            Some("runtime_15_typed_error_native_plugin_loader_structure_guard_folder_backed_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error native plugin loader routes child split" => {
            Some("runtime_15_typed_error_native_plugin_loader_routes_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error native plugin loader routes source helper child split" => {
            Some("runtime_15_typed_error_native_plugin_loader_routes_source_helper_child_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 typed-error native plugin loader source helper child split" => {
            Some("runtime_15_typed_error_native_plugin_loader_source_helper_child_split_static_passed_cargo_deferred")
        }
        _ => None,
    }
}
