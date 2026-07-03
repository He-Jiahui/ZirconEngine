use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 F5 scene property access typed errors",
        &[
            "runtime_15_scene_property_access_typed_errors_static_passed_cargo_deferred",
            "scene/world/property_access/read.rs",
            "scene/world/property_access/write.rs",
            "review_f5_scene_property_access_uses_scene_error",
        ],
    ),
    (
        "Runtime 15 F5 animation manager typed errors",
        &[
            "runtime_15_animation_manager_typed_errors_static_passed_cargo_deferred",
            "core/framework/animation/error.rs",
            "animation/manager/sampling.rs",
            "review_f5_animation_manager_uses_animation_error",
        ],
    ),
    (
        "Runtime 15 F5 animation asset binary typed errors",
        &[
            "runtime_15_animation_asset_binary_typed_errors_static_passed_cargo_deferred",
            "asset/assets/animation/error.rs",
            "AnimationAssetError::KindMismatch",
            "review_f5_animation_asset_binary_uses_typed_errors",
        ],
    ),
    (
        "Runtime 15 F5 profile export typed errors",
        &[
            "runtime_15_profile_export_typed_errors_static_passed_cargo_deferred",
            "core/runtime/diagnostics/profiling/export.rs",
            "ProfileExportError::CreateExportDirectory",
            "review_f5_profile_export_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 gameplay host typed errors",
        &[
            "runtime_15_gameplay_host_typed_errors_static_passed_cargo_deferred",
            "script/vm/gameplay_host/error.rs",
            "GameplayHostError",
            "review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary",
        ],
    ),
    (
        "Runtime 15 F5 script scene hook typed errors",
        &[
            "runtime_15_script_scene_hook_typed_errors_static_passed_cargo_deferred",
            "script/vm/scene_hook/error.rs",
            "ScriptSceneHookError",
            "review_f5_script_scene_hook_uses_typed_errors_before_core_boundary",
        ],
    ),
    (
        "Runtime 15 F5 VM plugin management policy typed errors",
        &[
            "runtime_15_vm_plugin_management_policy_typed_errors_static_passed_cargo_deferred",
            "script/vm/plugin/management_policy/error.rs",
            "VmPluginManagementPolicyError",
            "review_f5_vm_plugin_management_policy_uses_typed_validation_errors",
        ],
    ),
    (
        "Runtime 15 F5 UI surface input effect typed errors",
        &[
            "runtime_15_ui_surface_input_effect_typed_errors_static_passed_cargo_deferred",
            "ui/surface/input/error.rs",
            "UiSurfaceInputEffectError",
            "review_f5_ui_surface_input_effects_use_typed_errors_before_rejected_reason_boundary",
        ],
    ),
    (
        "Runtime 15 F5 UI input surrounding-text error source",
        &[
            "runtime_15_ui_input_surrounding_text_error_source_static_passed_cargo_deferred",
            "zircon_runtime_interface/src/ui/dispatch/input/effect.rs",
            "UiInputMethodSurroundingTextError",
            "review_f5_ui_input_surrounding_text_error_implements_std_error",
        ],
    ),
    (
        "Runtime 15 F5 UI template resource resolver typed errors",
        &[
            "runtime_15_ui_template_resource_resolver_typed_errors_static_passed_cargo_deferred",
            "ui/template/asset/resource_ref/resolver.rs",
            "UiResourceLookupError",
            "review_f5_ui_template_resource_resolver_uses_typed_lookup_errors_before_diagnostics_boundary",
        ],
    ),
    (
        "Runtime 15 F5 UI asset document typed errors",
        &[
            "runtime_15_ui_asset_document_typed_errors_static_passed_cargo_deferred",
            "asset/assets/ui.rs",
            "UiIconAssetDocumentError::InvalidSourceUri",
            "review_f5_ui_asset_documents_use_typed_errors_before_import_boundary",
        ],
    ),
    (
        "Runtime 15 F5 export CLI typed errors",
        &[
            "runtime_15_export_cli_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_export_pack/error.rs",
            "bin/zircon_export_validate/error.rs",
            "review_f5_export_cli_uses_typed_errors_before_cli_boundary",
        ],
    ),
    (
        "Runtime 15 F5 host reflection docs CLI typed errors",
        &[
            "runtime_15_host_reflection_docs_cli_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_host_reflection_docs/error.rs",
            "HostReflectionDocsError::CollectBuiltInHostModules",
            "review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary",
        ],
    ),
    (
        "Runtime 15 F5 shader prewarm args typed errors",
        &[
            "runtime_15_shader_prewarm_args_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/error.rs",
            "ShaderPrewarmArgsError::Usage",
            "review_f5_shader_prewarm_args_use_typed_usage_errors_before_cli_boundary",
        ],
    ),
    (
        "Runtime 15 F5 shader prewarm manifest merge typed errors",
        &[
            "runtime_15_shader_prewarm_manifest_merge_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/error.rs",
            "ShaderPrewarmManifestError::UnsupportedSchema",
            "shader_prewarm_merge_manifest_reports_typed_schema_error",
        ],
    ),
    (
        "Runtime 15 F5 shader prewarm manifest read typed errors",
        &[
            "runtime_15_shader_prewarm_manifest_read_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/error.rs",
            "ShaderPrewarmManifestError::Read",
            "shader_prewarm_read_manifest_reports_typed_read_error",
            "shader_prewarm_read_manifest_reports_typed_parse_error",
        ],
    ),
    (
        "Runtime 15 F5 shader prewarm report output typed errors",
        &[
            "runtime_15_shader_prewarm_report_output_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/error.rs",
            "ShaderPrewarmReportError::CreateReportDirectory",
            "shader_prewarm_report_write_reports_typed_directory_error",
        ],
    ),
    (
        "Runtime 15 F5 shader prewarm permutation registry typed errors",
        &[
            "runtime_15_shader_prewarm_permutation_registry_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/manifest/permutation_registry.rs",
            "ShaderPrewarmPermutationRegistryError::GeometrySourceIdBelowPluginRange",
            "shader_prewarm_permutation_registry_reports_typed_geometry_id_range_error",
        ],
    ),
    (
        "Runtime 15 F5 shader prewarm resource registry typed errors",
        &[
            "runtime_15_shader_prewarm_resource_registry_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/manifest/resource_registry.rs",
            "ShaderPrewarmResourceRegistryError::ReadRoot",
            "shader_prewarm_resource_registry_read_reports_typed_decode_error",
            "shader_prewarm_resource_registry_export_reports_typed_directory_error",
        ],
    ),
    (
        "Runtime 15 F5 shader prewarm asset-root scan typed errors",
        &[
            "runtime_15_shader_prewarm_asset_root_scan_typed_errors_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/manifest.rs",
            "ShaderPrewarmAssetScanError::ParseZShader",
            "shader_prewarm_asset_root_scan_reports_typed_empty_wgsl_error",
            "shader_prewarm_asset_root_scan_reports_typed_zmaterial_parse_error",
        ],
    ),
    (
        "Runtime 15 F5 shader prewarm CLI typed-error sweep",
        &[
            "runtime_15_shader_prewarm_cli_typed_error_sweep_static_passed_cargo_deferred",
            "bin/zircon_shader_prewarm/run.rs",
            "Result<ExitCode, String>",
            "review_f5_shader_prewarm_cli_typed_error_sweep_is_closed_at_run_boundary",
        ],
    ),
    (
        "Runtime 15 F5 dynamic API session typed errors",
        &[
            "runtime_15_dynamic_api_session_typed_errors_static_passed_cargo_deferred",
            "dynamic_api/session/error.rs",
            "RuntimeDynamicSessionError::RenderBridgeStep",
            "review_f5_dynamic_api_session_uses_typed_errors_before_abi_status_boundary",
        ],
    ),
    (
        "Runtime 15 F5 typed API residual typed errors",
        &[
            "runtime_15_typed_api_residual_typed_errors_static_passed_cargo_deferred",
            "scene/world/typed_api.rs",
            "scene/world/identity.rs",
            "review_f5_world_spawn_bundle_surface_uses_scene_error",
        ],
    ),
];
