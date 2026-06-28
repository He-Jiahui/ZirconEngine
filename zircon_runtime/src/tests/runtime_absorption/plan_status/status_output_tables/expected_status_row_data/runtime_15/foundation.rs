use super::ExpectedStatusOutputSlice;

pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 F9 runtime prelude required type coverage",
        &[
            "runtime_15_prelude_required_types_coremin_check_passed",
            "asset/prelude.rs",
            "runtime_prelude_exports_asset_scene_ui_and_graphics_contracts",
            "runtime_15_prelude_covers_required_types",
        ],
    ),
    (
        "Runtime 15 runtime UI dead-code support split",
        &[
            "runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed",
            "ui/public_runtime_frame.rs",
            "ui/tests/runtime_ui_support",
            "runtime_15_runtime_ui_dead_code_surface_is_test_support",
        ],
    ),
    (
        "Runtime 15 M5 production dead-code suppression global gate",
        &[
            "runtime_15_production_dead_code_suppression_global_gate_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code.rs",
            "DEAD_CODE_ALLOW_ATTRIBUTE",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
        ],
    ),
    (
        "Runtime 15 F12 dead-code review status sync",
        &[
            "runtime_15_f12_dead_code_review_status_sync_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
        ],
    ),
    (
        "Runtime 15 F12 dead-code runtime/editor boundary status guard",
        &[
            "runtime_15_f12_dead_code_runtime_editor_boundary_status_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "Runtime 15 + Editor UI 10 + convention",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
        ],
    ),
    (
        "Runtime 15 F12 UI text edit-state dead-code suppression cleanup",
        &[
            "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred",
            "ui/text/mod.rs",
            "ui/text/edit_state.rs",
            "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup",
        ],
    ),
    (
        "Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup",
        &[
            "runtime_15_ui_boundary_runtime_host_literal_cleanup_static_passed_cargo_deferred",
            "tests/ui_boundary/runtime_host.rs",
            "DEAD_CODE_ALLOW_ATTRIBUTE",
            "runtime_ui_host_surface_splits_production_frame_from_test_support",
        ],
    ),
    (
        "Runtime 15 F1 native host callback panic guard",
        &[
            "runtime_15_native_host_callback_panic_guard_static_passed_cargo_deferred",
            "plugin/native_plugin_loader/ffi_panic_guard.rs",
            "catch_native_host_api_panic",
            "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
        ],
    ),
    (
        "Runtime 15 graphics facade visibility note",
        &[
            "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift",
            "graphics/mod.rs",
            "Public facade exports",
            "runtime_15_mixed_visibility_has_facade_note",
        ],
    ),
    (
        "Runtime 15 F14 diagnostics normalization",
        &[
            "runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed",
            "FrameDiagnosticsStatus",
            "scene.ecs",
            "runtime_15_diagnostics_use_frame_trait_without_world_wrapper",
        ],
    ),
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
    (
        "Runtime 15 F5 fixed world mutation typed errors",
        &[
            "runtime_15_fixed_world_mutation_typed_errors_static_passed_cargo_deferred",
            "scene/world/component_access.rs",
            "scene/world/hierarchy.rs",
            "review_f5_fixed_world_mutation_uses_scene_error_variants",
        ],
    ),
    (
        "Runtime 15 F5 asset authoring typed errors",
        &[
            "runtime_15_asset_authoring_typed_errors_static_passed_cargo_deferred",
            "asset/assets/authoring.rs",
            "AssetAuthoringError",
            "review_f5_asset_authoring_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 navigation asset typed errors",
        &[
            "runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred",
            "asset/assets/navigation.rs",
            "NavigationAssetError",
            "review_f5_navigation_asset_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 font asset typed errors",
        &[
            "runtime_15_font_asset_typed_errors_static_passed_cargo_deferred",
            "asset/assets/font.rs",
            "FontAssetError::Parse",
            "review_f5_font_asset_uses_typed_error_source",
        ],
    ),
    (
        "Runtime 15 F5 sound asset typed errors",
        &[
            "runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred",
            "asset/assets/sound.rs",
            "SoundAssetError::UnsupportedSpeakerMaskBits",
            "review_f5_sound_asset_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 zshader definition typed errors",
        &[
            "runtime_15_zshader_definition_typed_errors_static_passed_cargo_deferred",
            "asset/assets/shader/zshader.rs",
            "ZShaderDefinitionError::UnsupportedKind",
            "review_f5_zshader_definition_values_use_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 asset meta typed errors",
        &[
            "runtime_15_asset_meta_typed_errors_static_passed_cargo_deferred",
            "asset/project/meta.rs",
            "AssetMetaError::UnsupportedFormatVersion",
            "review_f5_asset_meta_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 texture loader typed errors",
        &[
            "runtime_15_texture_loader_typed_errors_static_passed_cargo_deferred",
            "asset/load/texture.rs",
            "TextureLoadError::OpenImage",
            "review_f5_texture_loader_uses_typed_error",
        ],
    ),
    (
        "Runtime 15 F5 mesh loader typed errors",
        &[
            "runtime_15_mesh_loader_typed_errors_static_passed_cargo_deferred",
            "asset/load/mesh.rs",
            "MeshLoadError::UnsupportedFormat",
            "review_f5_mesh_loader_and_obj_decoder_use_typed_errors",
        ],
    ),
    (
        "Runtime 15 F13 provider registration shared owner",
        &[
            "runtime_15_provider_registration_shared_owner_coremin_check_passed",
            "graphics/runtime_provider/registration.rs",
            "RuntimeProviderRegistration<P: ?Sized>",
            "runtime_15_provider_registration_uses_shared_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider update shared stats owner",
        &[
            "runtime_15_provider_update_shared_stats_owner_coremin_check_passed",
            "graphics/runtime_provider/update.rs",
            "RuntimeProviderUpdate<S>",
            "runtime_15_provider_update_uses_shared_stats_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider feedback shared payload owner",
        &[
            "runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed",
            "graphics/runtime_provider/feedback.rs",
            "RuntimeProviderFeedback<G, V>",
            "runtime_15_provider_feedback_uses_shared_payload_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider prepare input shared frame owner",
        &[
            "runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed",
            "graphics/runtime_provider/prepare_input.rs",
            "RuntimeProviderPrepareInput<'a, E>",
            "runtime_15_provider_prepare_input_uses_shared_extract_generation_owner",
        ],
    ),
    (
        "Runtime 15 F13 full provider boilerplate audit",
        &[
            "runtime_15_provider_boilerplate_full_audit_coremin_check_passed",
            "structure_convention/provider_boilerplate.rs",
            "RuntimeProviderRegistration<P: ?Sized>",
            "runtime_15_no_duplicated_provider_boilerplate",
        ],
    ),
    (
        "Runtime 15 F12 runtime-owned dead-code suppression cleanup",
        &[
            "runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed",
            "asset/pipeline/worker_pool.rs",
            "core/runtime/state/module_entry.rs",
            "runtime_15_runtime_owned_dead_code_suppression_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 script host value descriptor dead-code cleanup",
        &[
            "runtime_15_script_host_value_descriptors_coremin_check_passed",
            "script/vm/host/builtin_host_modules.rs",
            "docs/zircon_runtime/script/vm/host/function_ledger.md",
            "runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
        ],
    ),
    (
        "Runtime 15 F12 script reflection macro fixture dead-code cleanup",
        &[
            "runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred",
            "script/vm/tests/reflection_docs.rs",
            "docs/zircon_runtime/script/vm/zr_vm_host_reflection.md",
            "runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code",
        ],
    ),
    (
        "Runtime 15 M1 animation manager folder-backed cutover",
        &[
            "runtime_15_animation_manager_folder_backed_cutover_static_passed_cargo_deferred",
            "animation/manager/mod.rs",
            "animation/manager/graph.rs",
            "docs/zircon_runtime/animation/runtime.md",
            "runtime_15_animation_manager_is_folder_backed",
        ],
    ),
];
