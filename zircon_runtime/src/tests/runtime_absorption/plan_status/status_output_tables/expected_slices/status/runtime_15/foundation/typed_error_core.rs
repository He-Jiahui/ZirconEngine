pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F5 scene property access typed errors" {
        Some("runtime_15_scene_property_access_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 animation manager typed errors" {
        Some("runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 animation asset binary typed errors" {
        Some("runtime_15_animation_asset_binary_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 profile export typed errors" {
        Some("runtime_15_profile_export_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 gameplay host typed errors" {
        Some("runtime_15_gameplay_host_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 script scene hook typed errors" {
        Some("runtime_15_script_scene_hook_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 VM plugin management policy typed errors" {
        Some("runtime_15_vm_plugin_management_policy_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 UI surface input effect typed errors" {
        Some("runtime_15_ui_surface_input_effect_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 UI input surrounding-text error source" {
        Some("runtime_15_ui_input_surrounding_text_error_source_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 UI template resource resolver typed errors" {
        Some("runtime_15_ui_template_resource_resolver_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 UI asset document typed errors" {
        Some("runtime_15_ui_asset_document_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 export CLI typed errors" {
        Some("runtime_15_export_cli_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 host reflection docs CLI typed errors" {
        Some("runtime_15_host_reflection_docs_cli_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 shader prewarm args typed errors" {
        Some("runtime_15_shader_prewarm_args_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 shader prewarm manifest merge typed errors" {
        Some("runtime_15_shader_prewarm_manifest_merge_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 shader prewarm manifest read typed errors" {
        Some("runtime_15_shader_prewarm_manifest_read_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 shader prewarm report output typed errors" {
        Some("runtime_15_shader_prewarm_report_output_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 shader prewarm permutation registry typed errors" {
        Some("runtime_15_shader_prewarm_permutation_registry_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 shader prewarm resource registry typed errors" {
        Some(
            "runtime_15_shader_prewarm_resource_registry_typed_errors_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F5 shader prewarm asset-root scan typed errors" {
        Some("runtime_15_shader_prewarm_asset_root_scan_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 shader prewarm CLI typed-error sweep" {
        Some("runtime_15_shader_prewarm_cli_typed_error_sweep_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 dynamic API session typed errors" {
        Some("runtime_15_dynamic_api_session_typed_errors_static_passed_cargo_deferred")
    } else {
        None
    }
}
