use super::ExpectedStatusOutputSlice;

#[path = "runtime_15/m3.rs"]
mod m3;
#[path = "runtime_15/m4.rs"]
mod m4;

pub(super) const RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES:
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
        "Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup",
        &[
            "runtime_15_ui_boundary_runtime_host_literal_cleanup_static_passed_cargo_deferred",
            "tests/ui_boundary/runtime_host.rs",
            "DEAD_CODE_ALLOW_ATTRIBUTE",
            "runtime_ui_host_surface_splits_production_frame_from_test_support",
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
    (
        "Runtime 15 M2 core runtime state module naming hard cutover",
        &[
            "runtime_15_core_runtime_state_module_naming_hard_cutover_static_passed_cargo_deferred",
            "core/runtime/state/core_runtime_state.rs",
            "core/runtime/state/mod.rs",
            "runtime_15_core_runtime_state_module_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS observer callback registry module naming hard cutover",
        &[
            "runtime_15_scene_ecs_observer_callback_registry_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/ecs/observer/callback_registry.rs",
            "scene/ecs/observer/mod.rs",
            "runtime_15_scene_ecs_observer_callback_registry_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS query-state many-item array module naming hard cutover",
        &[
            "runtime_15_scene_ecs_query_state_many_item_array_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/ecs/query/query_state/many_item_array.rs",
            "scene/ecs/query/query_state/mod.rs",
            "runtime_15_scene_ecs_query_state_many_item_array_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS component-storage component results module naming hard cutover",
        &[
            "runtime_15_scene_ecs_component_storage_component_results_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/ecs/storage/component_storage/component_results.rs",
            "scene/ecs/storage/component_storage/mod.rs",
            "runtime_15_scene_ecs_component_storage_component_results_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover",
        &[
            "runtime_15_asset_watcher_shutdown_on_drop_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/watch/shutdown_on_drop.rs",
            "asset/watch/mod.rs",
            "runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 asset change construction module naming hard cutover",
        &[
            "runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/watch/asset_change_construction.rs",
            "asset/watch/mod.rs",
            "runtime_15_asset_change_construction_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 resource streamer construction module naming hard cutover",
        &[
            "runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/resources/resource_streamer/resource_streamer_construction.rs",
            "graphics/scene/resources/resource_streamer/mod.rs",
            "runtime_15_resource_streamer_construction_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 offscreen target construct directory naming hard cutover",
        &[
            "runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result",
            "graphics/backend/render_backend/offscreen_target_construct/construct.rs",
            "graphics/backend/render_backend/mod.rs",
            "runtime_15_offscreen_target_construct_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 asset texture upload readiness container fixtures module naming hard cutover",
        &[
            "runtime_15_asset_texture_upload_readiness_container_fixtures_naming_hard_cutover_static_passed_cargo_deferred",
            "asset/tests/assets/texture_upload_readiness/container_fixtures.rs",
            "asset/tests/assets/texture_upload_readiness.rs",
            "runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS query cached queries module naming hard cutover",
        &[
            "runtime_15_scene_ecs_query_cached_queries_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/tests/ecs_query/cached_queries.rs",
            "scene/tests/ecs_query.rs",
            "runtime_15_scene_ecs_query_cached_queries_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover",
        &[
            "runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred",
            "dynamic_api/session/tests/vampire_runtime_support.rs",
            "dynamic_api/session/tests/mod.rs",
            "runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 camera controller output module naming hard cutover",
        &[
            "runtime_15_camera_controller_output_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/camera_controller/controller_output.rs",
            "core/framework/camera_controller/mod.rs",
            "runtime_15_camera_controller_output_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 scene ECS systems many/single queries module naming hard cutover",
        &[
            "runtime_15_scene_ecs_systems_many_single_queries_naming_hard_cutover_static_passed_cargo_timeout_no_result",
            "scene/tests/ecs_systems/many_single_queries.rs",
            "scene/tests/ecs_systems.rs",
            "runtime_15_scene_ecs_systems_many_single_queries_uses_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 plugin static manifest contract owner naming hard cutover",
        &[
            "runtime_15_plugin_static_manifest_contract_owner_naming_hard_cutover_static_passed_cargo_deferred",
            "plugin_extensions/static_manifest_contracts/feature_bundles/feature_bundle_rows.rs",
            "plugin_extensions/static_manifest_contracts/package_coordinates/package_coordinate_resolution.rs",
            "plugin_extensions/static_manifest_contracts/package_identity/package_id_tokens.rs",
            "plugin_extensions/static_manifest_contracts/package_kind/package_kind_fields.rs",
            "runtime_15_plugin_static_manifest_contract_owners_use_domain_names",
        ],
    ),
    (
        "Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover",
        &[
            "runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred",
            "ui/component/catalog/editor_showcase/descriptor_builders.rs",
            "ui/component/catalog/editor_showcase.rs",
            "runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name",
        ],
    ),
    (
        "Runtime 15 M2 UI table sortingMode server literal allowed-context sync",
        &[
            "runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred",
            "ui/surface/surface/default_interactions/table/columns.rs",
            "non_network_server_naming.py",
            "runtime_non_network_server_naming_is_classified_by_owner",
            "runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context",
        ],
    ),
    (
        "Runtime 15 M2 graphics render-framework receiver naming hard cutover",
        &[
            "runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/runtime/render_framework",
            "framework: &WgpuRenderFramework",
            "runtime_non_network_server_naming_is_classified_by_owner",
            "runtime_15_render_framework_receiver_uses_framework_name",
        ],
    ),
    (
        "Runtime 15 M2 editor workbench authority-label naming hard cutover",
        &[
            "runtime_15_editor_workbench_authority_label_naming_hard_cutover_static_passed_cargo_deferred",
            "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/gameplay_state.rs",
            "Selected Condition_Night   editor authority",
            "non_network_server_naming.py",
            "runtime_15_editor_workbench_authority_label_uses_editor_name",
        ],
    ),
];

pub(super) const RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] =
    m4::EXPECTED_STATUS_OUTPUT_SLICES;

pub(super) const RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 F12 offscreen target texture owner cleanup",
        &[
            "runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result",
            "graphics/backend/render_backend/offscreen_target.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_offscreen_target_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 render backend state owner cleanup",
        &[
            "runtime_15_render_backend_state_owner_cleanup_coremin_check_passed",
            "graphics/backend/render_backend/render_backend.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_render_backend_state_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu texture resource owner cleanup",
        &[
            "runtime_15_gpu_texture_resource_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_texture/gpu_texture_resource.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_gpu_texture_resource_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu material uniform owner cleanup",
        &[
            "runtime_15_gpu_material_uniform_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_gpu_material_uniform_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu mesh order signature cleanup",
        &[
            "runtime_15_gpu_mesh_order_signature_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs",
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
            "runtime_15_gpu_mesh_order_signature_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu model identity cleanup",
        &[
            "runtime_15_gpu_model_identity_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_model/gpu_model_resource.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_gpu_model_identity_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 post-process LUT texture owner cleanup",
        &[
            "runtime_15_post_process_lut_texture_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_post_process_lut_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 output target texture owner cleanup",
        &[
            "runtime_15_output_target_texture_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/output_target_texture/output_target_texture_resource.rs",
            "graphics/scene/resources/prepared/prepared_output_target_texture.rs",
            "runtime_15_output_target_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 material runtime capture seed cleanup",
        &[
            "runtime_15_material_runtime_capture_seed_cleanup_coremin_check_passed",
            "graphics/scene/resources/runtime/material_runtime.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_material_runtime_capture_seed_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 resource streamer diagnostics accessor cleanup",
        &[
            "runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "resource_streamer_ensure_scene_resources.rs",
            "runtime_15_resource_streamer_diagnostics_accessor_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 resource streamer resolve texture id cleanup",
        &[
            "runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs",
            "resolve_texture_reference_with_support",
            "runtime_15_resource_streamer_resolve_texture_id_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 particle GPU readback output accessor cleanup",
        &[
            "runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs",
            "renderer.take_last_particle_gpu_readback_outputs()",
            "runtime_15_particle_gpu_readback_output_accessor_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 advanced plugin output test accessor cleanup",
        &[
            "runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs",
            "has_particle_gpu_readback",
            "runtime_15_advanced_plugin_output_test_accessor_cleanup",
        ],
    ),
];

pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_UI_TESTS_FIRST_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::UI_TESTS_FIRST_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::ASSET_BUDGET_TESTS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_SCENE_SCRIPT_TESTS_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::SCENE_SCRIPT_TESTS_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::STATUS_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_UI_TESTS_SECOND_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::UI_TESTS_SECOND_EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES:
    &[ExpectedStatusOutputSlice] = m3::PRODUCTION_GUARD_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES;
