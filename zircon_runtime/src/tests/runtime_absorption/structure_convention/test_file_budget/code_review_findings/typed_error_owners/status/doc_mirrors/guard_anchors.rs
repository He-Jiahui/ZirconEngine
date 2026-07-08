use super::super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_status_doc_guard_anchors_are_synced(
    sources: &TypedErrorStatusDocSources,
) {
    for (label, source) in typed_error_status_doc_mirror_sources(sources) {
        assert_contains_all(
            label,
            source,
            &[
                "runtime_15_code_review_findings_typed_error_structure_guard_is_child_owner",
                "runtime_15_typed_error_structure_assertions_are_child_owner",
                "runtime_15_typed_error_structure_assertions_children_are_child_owned",
                "runtime_15_typed_error_structure_assertions_guard_folder_backed_status_is_current",
                "runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
                "runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
                "runtime_15_typed_error_source_inventory_is_child_owner",
                "runtime_15_code_review_findings_tests_are_folder_backed",
                "review_f5_texture_loader_uses_typed_error",
                "review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
                "review_f5_animation_asset_binary_uses_typed_errors",
                "review_f5_asset_authoring_uses_typed_error",
                "review_f5_navigation_asset_uses_typed_error",
                "review_f5_font_asset_uses_typed_error_source",
                "review_f5_sound_asset_uses_typed_error",
                "review_f5_zshader_v2_replaces_user_shader_definitions",
                "review_f5_asset_meta_uses_typed_error",
                "review_f5_world_spawn_bundle_surface_uses_scene_error",
                "review_f5_scene_property_access_uses_scene_error",
                "review_f7_asset_artifact_errors_use_asset_import_error_sources",
                "review_f5_native_plugin_behavior_abi_uses_typed_error",
                "review_f5_native_plugin_string_helpers_use_typed_error",
                "review_f5_native_plugin_descriptor_abi_uses_typed_error",
                "review_f5_native_plugin_entry_abi_uses_typed_error",
                "review_f5_native_host_api_adapter_uses_typed_error",
                "review_f5_native_live_host_loading_uses_typed_error",
                "review_f5_native_live_host_lifecycle_uses_typed_error",
                "review_f5_native_live_host_hot_reload_uses_typed_error",
                "review_f5_native_live_host_registration_replay_uses_typed_error",
                "review_f5_native_live_host_bridge_methods_use_typed_error",
                "review_f5_native_live_host_runtime_behavior_uses_typed_error",
                "review_f5_native_plugin_distribution_compat_uses_typed_error",
                "review_f5_native_plugin_manifest_candidate_uses_typed_error",
                "review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary",
                "review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary",
                "review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary",
                "review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary",
                "review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
                "review_f5_ui_input_surrounding_text_error_implements_std_error",
                "Cargo gate deferred",
            ],
        );
    }
}
