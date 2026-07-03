use super::super::super::super::super::*;
use super::*;

const PRESERVED_TYPED_ERROR_REVIEW_GUARDS: &[&str] = &[
    "fn review_f5_animation_manager_uses_animation_error",
    "fn review_f6_core_resource_registry_rename_uses_core_error",
    "fn review_f5_texture_loader_uses_typed_error",
    "fn review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
    "fn review_f5_animation_asset_binary_uses_typed_errors",
    "fn review_f7_asset_artifact_errors_use_asset_import_error_sources",
    "fn review_f5_asset_authoring_uses_typed_error",
    "fn review_f5_navigation_asset_uses_typed_error",
    "fn review_f5_font_asset_uses_typed_error_source",
    "fn review_f5_sound_asset_uses_typed_error",
    "fn review_f5_zshader_v2_replaces_user_shader_definitions",
    "fn review_f5_asset_meta_uses_typed_error",
    "fn review_f5_profile_export_uses_typed_error",
    "fn review_f5_dynamic_api_session_uses_typed_errors_before_abi_status_boundary",
    "fn review_f5_export_cli_uses_typed_errors_before_cli_boundary",
    "fn review_f5_native_plugin_behavior_abi_uses_typed_error",
    "fn review_f5_native_bridge_method_abi_uses_typed_error",
    "fn review_f5_native_plugin_string_helpers_use_typed_error",
    "fn review_f5_native_plugin_descriptor_abi_uses_typed_error",
    "fn review_f5_native_plugin_entry_abi_uses_typed_error",
    "fn review_f5_native_host_api_adapter_uses_typed_error",
    "fn review_f5_native_live_host_bridge_lifecycle_uses_typed_error",
    "fn review_f5_native_live_host_behavior_diagnostics_use_typed_error",
    "fn review_f5_native_live_host_loading_uses_typed_error",
    "fn review_f5_native_live_host_lifecycle_uses_typed_error",
    "fn review_f5_native_live_host_hot_reload_uses_typed_error",
    "fn review_f5_native_live_host_registration_replay_uses_typed_error",
    "fn review_f5_native_live_host_bridge_methods_use_typed_error",
    "fn review_f5_native_live_host_runtime_behavior_uses_typed_error",
    "fn review_f5_native_plugin_distribution_compat_uses_typed_error",
    "fn review_f5_native_plugin_registration_manifest_uses_typed_error",
    "fn review_f5_native_plugin_manifest_collection_uses_typed_error",
    "fn review_f5_native_plugin_manifest_candidate_uses_typed_error",
    "fn review_f5_world_spawn_bundle_surface_uses_scene_error",
    "fn review_f5_fixed_world_mutation_uses_scene_error_variants",
    "fn review_f5_dynamic_component_errors_preserve_scene_error_sources",
    "fn review_f5_scene_property_access_uses_scene_error",
    "fn review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary",
    "fn review_f5_script_scene_hook_uses_typed_errors_before_core_boundary",
    "fn review_f5_vm_plugin_management_policy_uses_typed_validation_errors",
    "fn review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary",
    "fn review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary",
    "fn review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary",
    "fn review_f5_ui_asset_documents_use_typed_errors_before_import_boundary",
    "fn review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
    "fn review_f5_ui_input_surrounding_text_error_implements_std_error",
    "fn review_f5_ui_template_resource_resolver_uses_typed_lookup_errors_before_diagnostics_boundary",
];

pub(super) fn assert_typed_error_preserved_review_guards_are_current() {
    let typed_error_children = typed_error_children_source();
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_CHILD);

    assert!(
        !parent.contains("fn review_f5_texture_loader_uses_typed_error"),
        "preserved typed-error review guard list should stay in {TYPED_ERROR_STRUCTURE_MOVED_GUARD_ABSENCE_PRESERVED_GUARDS_CHILD}"
    );
    assert_contains_all(
        "typed-error child owners preserve F5/F6/F7 review guards",
        &typed_error_children,
        PRESERVED_TYPED_ERROR_REVIEW_GUARDS,
    );
}

#[test]
fn runtime_15_typed_error_moved_guard_absence_preserved_guards_are_child_owned() {
    assert_typed_error_preserved_review_guards_are_current();
}
