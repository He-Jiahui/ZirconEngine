pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F5 native plugin distribution compatibility typed errors" {
        Some(
            "runtime_15_native_plugin_distribution_compat_typed_errors_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F5 native plugin registration manifest typed errors" {
        Some(
            "runtime_15_native_plugin_registration_manifest_typed_errors_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F5 native plugin behavior ABI typed errors" {
        Some("runtime_15_native_plugin_behavior_abi_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native bridge method ABI typed errors" {
        Some("runtime_15_native_bridge_method_abi_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native plugin manifest collection typed errors" {
        Some(
            "runtime_15_native_plugin_manifest_collection_typed_errors_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F5 native plugin manifest candidate typed errors" {
        Some(
            "runtime_15_native_plugin_manifest_candidate_typed_errors_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F5 native plugin string helper typed errors" {
        Some("runtime_15_native_plugin_string_helper_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native plugin descriptor ABI typed errors" {
        Some("runtime_15_native_plugin_descriptor_abi_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native plugin entry ABI typed errors" {
        Some("runtime_15_native_plugin_entry_abi_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native host API adapter typed errors" {
        Some("runtime_15_native_host_api_adapter_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native live-host loading typed errors" {
        Some("runtime_15_native_live_host_loading_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native live-host behavior diagnostics typed errors" {
        Some(
            "runtime_15_native_live_host_behavior_diagnostics_typed_errors_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F5 native live-host lifecycle typed errors" {
        Some("runtime_15_native_live_host_lifecycle_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native live-host hot reload typed errors" {
        Some("runtime_15_native_live_host_hot_reload_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native live-host registration replay typed errors" {
        Some(
            "runtime_15_native_live_host_registration_replay_typed_errors_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F5 native live-host bridge methods typed errors" {
        Some("runtime_15_native_live_host_bridge_methods_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native live-host runtime behavior typed errors" {
        Some("runtime_15_native_live_host_runtime_behavior_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 native live-host bridge lifecycle typed errors" {
        Some(
            "runtime_15_native_live_host_bridge_lifecycle_typed_errors_static_passed_cargo_deferred",
        )
    } else {
        None
    }
}
