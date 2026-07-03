use super::super::super::super::super::*;

const TYPED_ERROR_PARENT_PATHS: &[(&str, &str)] = &[
    (
        "typed-error convergence parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
    ),
    (
        "asset loader typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
    ),
    (
        "asset records typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
    ),
    (
        "native typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
    ),
    (
        "native ABI typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
    ),
    (
        "native plugin descriptor typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs",
    ),
    (
        "native live-host typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs",
    ),
    (
        "native live-host lifecycle-paths typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs",
    ),
    (
        "native live-host replay-runtime typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs",
    ),
    (
        "native manifest typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs",
    ),
    (
        "scene world typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
    ),
    (
        "script host typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
    ),
    (
        "shader prewarm CLI typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli.rs",
    ),
    (
        "UI input typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
    ),
];

const PARENT_BACKFLOW_GUARDS: &[&str] = &[
    "fn review_f5_world_spawn_bundle_surface_uses_scene_error",
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
    "fn review_f5_native_plugin_behavior_abi_uses_typed_error",
    "fn review_f5_native_bridge_method_abi_uses_typed_error",
    "fn review_f5_native_plugin_string_helpers_use_typed_error",
    "fn review_f5_native_plugin_descriptor_abi_uses_typed_error",
    "fn review_f5_native_plugin_entry_abi_uses_typed_error",
    "fn review_f5_native_host_api_adapter_uses_typed_error",
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
    "fn review_f5_fixed_world_mutation_uses_scene_error_variants",
    "fn review_f5_dynamic_component_errors_preserve_scene_error_sources",
    "fn review_f5_scene_property_access_uses_scene_error",
    "fn review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary",
    "fn review_f5_script_scene_hook_uses_typed_errors_before_core_boundary",
    "fn review_f5_vm_plugin_management_policy_uses_typed_validation_errors",
    "fn review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary",
    "fn review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary",
    "fn review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary",
];

pub(super) fn assert_typed_error_parent_backflow_guards_are_absent() {
    let parent_sources = TYPED_ERROR_PARENT_PATHS
        .iter()
        .map(|(label, path)| (*label, *path, read_runtime_src(path)))
        .collect::<Vec<_>>();
    for child_owned_test in PARENT_BACKFLOW_GUARDS {
        for (label, path, source) in &parent_sources {
            assert!(
                !source.contains(child_owned_test),
                "child-owned review guard `{child_owned_test}` should not return to {label} at {path}"
            );
        }
    }
}

#[test]
fn runtime_15_typed_error_moved_guard_absence_parent_backflow_guards_are_child_owned() {
    assert_typed_error_parent_backflow_guards_are_absent();
}
