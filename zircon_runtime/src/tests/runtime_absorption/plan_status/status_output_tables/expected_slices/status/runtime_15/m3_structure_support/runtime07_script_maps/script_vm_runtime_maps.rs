pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 script VM test folder split" => {
            Some("runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result")
        }
        "Runtime 15 M3 script VM primary guard child-owner split" => Some(
            "runtime_15_script_vm_primary_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 script VM hot-reload coordinator test folder split" => Some(
            "runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 script VM hot-reload guard child-owner split" => Some(
            "runtime_15_script_vm_hot_reload_guard_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 native live-host tests folder split" => {
            Some("runtime_15_native_live_host_tests_folder_split_static_passed_cargo_deferred")
        }
        "Runtime 15 M3 native plugin loader real fixture test folder split" => Some(
            "runtime_15_native_plugin_loader_real_fixture_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 extension registry bridge test folder split" => Some(
            "runtime_15_extension_registry_bridge_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 manifest contributions test folder split" => Some(
            "runtime_15_manifest_contributions_tests_folder_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 manifest contributions runtime-family test child-owner split" => Some(
            "runtime_15_manifest_contributions_runtime_family_tests_child_owner_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
