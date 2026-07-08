pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == "Runtime 15 F5 typed API residual typed errors" {
        Some("runtime_15_typed_api_residual_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 fixed world mutation typed errors" {
        Some("runtime_15_fixed_world_mutation_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 asset authoring typed errors" {
        Some("runtime_15_asset_authoring_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 navigation asset typed errors" {
        Some("runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 font asset typed errors" {
        Some("runtime_15_font_asset_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 sound asset typed errors" {
        Some("runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F7 artifact cache JSON number typed errors" {
        Some("runtime_15_artifact_cache_json_number_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 zshader v2 user definition migration" {
        Some("runtime_15_zshader_v2_user_definition_migration_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 asset meta typed errors" {
        Some("runtime_15_asset_meta_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 texture loader typed errors" {
        Some("runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F5 mesh loader typed errors" {
        Some("runtime_15_mesh_loader_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F8 texture descriptor typed errors" {
        Some("runtime_15_texture_descriptor_typed_errors_static_passed_cargo_deferred")
    } else if slice == "Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup" {
        Some(
            "runtime_15_runtime_plugin_descriptor_status_mirror_cleanup_static_passed_cargo_deferred",
        )
    } else if slice == "Runtime 15 F13 provider registration shared owner" {
        Some("runtime_15_provider_registration_shared_owner_coremin_check_passed")
    } else if slice == "Runtime 15 F13 provider update shared stats owner" {
        Some("runtime_15_provider_update_shared_stats_owner_coremin_check_passed")
    } else if slice == "Runtime 15 F13 provider feedback shared payload owner" {
        Some("runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed")
    } else if slice == "Runtime 15 F13 provider prepare input shared frame owner" {
        Some("runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed")
    } else if slice == "Runtime 15 F13 full provider boilerplate audit" {
        Some("runtime_15_provider_boilerplate_full_audit_coremin_check_passed")
    } else if slice == "Runtime 15 F12 runtime-owned dead-code suppression cleanup" {
        Some("runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed")
    } else if slice == "Runtime 15 F12 script host value descriptor dead-code cleanup" {
        Some("runtime_15_script_host_value_descriptors_coremin_check_passed")
    } else if slice == "Runtime 15 F12 script reflection macro fixture dead-code cleanup" {
        Some(
            "runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred",
        )
    } else {
        None
    }
}
