use super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_review_guards_are_preserved(
    sources: &TypedErrorChildOwnershipSources,
) {
    assert_contains_all(
        "typed-error native plugin loader child owns native structure checks",
        &sources.native_plugin_loader_child,
        &[
            "fn runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs",
            "runtime_15_typed_error_native_plugin_loader_structure_is_child_owner",
        ],
    );
    assert_contains_all(
        "typed-error moved-guard child owns review guard preservation checks",
        &sources.moved_guard_absence_child_tree,
        &[
            "fn runtime_15_typed_error_structure_moved_guard_absence_is_child_owner",
            "super::super::source_inventory::typed_error_children_source",
            "runtime_15_typed_error_moved_guard_absence_preserved_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned",
            "runtime_15_typed_error_moved_guard_absence_path_anchors_are_child_owned",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/string_helpers.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/descriptor_abi.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/entry_abi.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surface_effects.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input/surrounding_text.rs",
            "review_f5_texture_loader_uses_typed_error",
            "review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
            "review_f5_asset_authoring_uses_typed_error",
            "review_f5_navigation_asset_uses_typed_error",
            "review_f5_font_asset_uses_typed_error_source",
            "review_f5_sound_asset_uses_typed_error",
            "review_f5_zshader_v2_replaces_user_shader_definitions",
            "review_f5_asset_meta_uses_typed_error",
            "review_f5_native_plugin_descriptor_abi_uses_typed_error",
            "review_f5_native_live_host_loading_uses_typed_error",
            "review_f5_native_live_host_hot_reload_uses_typed_error",
            "review_f5_native_live_host_registration_replay_uses_typed_error",
            "review_f5_native_live_host_bridge_methods_use_typed_error",
            "review_f5_native_live_host_runtime_behavior_uses_typed_error",
            "review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary",
            "review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary",
            "review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
            "review_f5_ui_input_surrounding_text_error_implements_std_error",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    );
    assert_contains_all(
        "typed-error source helper preserves review guard anchors",
        &sources.typed_error_sources,
        &[
            "review_f5_texture_loader_uses_typed_error",
            "review_f5_native_plugin_descriptor_abi_uses_typed_error",
            "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        ],
    );
    assert_eq!(
        typed_error_review_guard_count(),
        47,
        "typed-error child owners should preserve all current F5/F6/F7 review guards"
    );
}
