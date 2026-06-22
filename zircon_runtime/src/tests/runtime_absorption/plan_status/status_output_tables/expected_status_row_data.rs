use super::ExpectedStatusOutputSlice;

#[path = "expected_status_row_data/runtime_01_04.rs"]
mod runtime_01_04;
#[path = "expected_status_row_data/runtime_05.rs"]
mod runtime_05;
#[path = "expected_status_row_data/runtime_06_09.rs"]
mod runtime_06_09;
#[path = "expected_status_row_data/runtime_10_13.rs"]
mod runtime_10_13;
#[path = "expected_status_row_data/runtime_14.rs"]
mod runtime_14;

pub(super) const RUNTIME_15_EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 F9 runtime prelude required type coverage",
        [
            "runtime_15_prelude_required_types_coremin_check_passed",
            "asset/prelude.rs",
            "runtime_prelude_exports_asset_scene_ui_and_graphics_contracts",
            "runtime_15_prelude_covers_required_types",
        ],
    ),
    (
        "Runtime 15 runtime UI dead-code support split",
        [
            "runtime_15_runtime_ui_dead_code_support_split_coremin_check_passed",
            "ui/public_runtime_frame.rs",
            "ui/tests/runtime_ui_support",
            "runtime_15_runtime_ui_dead_code_surface_is_test_support",
        ],
    ),
    (
        "Runtime 15 graphics facade visibility note",
        [
            "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift",
            "graphics/mod.rs",
            "Public facade exports",
            "runtime_15_mixed_visibility_has_facade_note",
        ],
    ),
    (
        "Runtime 15 F14 diagnostics normalization",
        [
            "runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed",
            "FrameDiagnosticsStatus",
            "scene.ecs",
            "runtime_15_diagnostics_use_frame_trait_without_world_wrapper",
        ],
    ),
    (
        "Runtime 15 F13 provider registration shared owner",
        [
            "runtime_15_provider_registration_shared_owner_coremin_check_passed",
            "graphics/runtime_provider/registration.rs",
            "RuntimeProviderRegistration<P: ?Sized>",
            "runtime_15_provider_registration_uses_shared_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider update shared stats owner",
        [
            "runtime_15_provider_update_shared_stats_owner_coremin_check_passed",
            "graphics/runtime_provider/update.rs",
            "RuntimeProviderUpdate<S>",
            "runtime_15_provider_update_uses_shared_stats_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider feedback shared payload owner",
        [
            "runtime_15_provider_feedback_shared_payload_owner_coremin_check_passed",
            "graphics/runtime_provider/feedback.rs",
            "RuntimeProviderFeedback<G, V>",
            "runtime_15_provider_feedback_uses_shared_payload_owner",
        ],
    ),
    (
        "Runtime 15 F13 provider prepare input shared frame owner",
        [
            "runtime_15_provider_prepare_input_shared_frame_owner_coremin_check_passed",
            "graphics/runtime_provider/prepare_input.rs",
            "RuntimeProviderPrepareInput<'a, E>",
            "runtime_15_provider_prepare_input_uses_shared_extract_generation_owner",
        ],
    ),
    (
        "Runtime 15 F13 full provider boilerplate audit",
        [
            "runtime_15_provider_boilerplate_full_audit_coremin_check_passed",
            "structure_convention/provider_boilerplate.rs",
            "RuntimeProviderRegistration<P: ?Sized>",
            "runtime_15_no_duplicated_provider_boilerplate",
        ],
    ),
    (
        "Runtime 15 F12 runtime-owned dead-code suppression cleanup",
        [
            "runtime_15_runtime_owned_dead_code_suppression_cleanup_coremin_check_passed",
            "asset/pipeline/worker_pool.rs",
            "core/runtime/state/module_entry.rs",
            "runtime_15_runtime_owned_dead_code_suppression_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 script host value descriptor dead-code cleanup",
        [
            "runtime_15_script_host_value_descriptors_coremin_check_passed",
            "script/vm/host/builtin_host_modules.rs",
            "docs/zircon_runtime/script/vm/host/function_ledger.md",
            "runtime_15_script_host_value_descriptors_do_not_suppress_dead_code",
        ],
    ),
    (
        "Runtime 15 F12 offscreen target texture owner cleanup",
        [
            "runtime_15_offscreen_target_texture_owner_cleanup_static_passed_cargo_timeout_no_result",
            "graphics/backend/render_backend/offscreen_target.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_offscreen_target_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 render backend state owner cleanup",
        [
            "runtime_15_render_backend_state_owner_cleanup_coremin_check_passed",
            "graphics/backend/render_backend/render_backend.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_render_backend_state_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu texture resource owner cleanup",
        [
            "runtime_15_gpu_texture_resource_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_texture/gpu_texture_resource.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_gpu_texture_resource_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu material uniform owner cleanup",
        [
            "runtime_15_gpu_material_uniform_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs",
            "docs/zircon_runtime/graphics/render-product-submit.md",
            "runtime_15_gpu_material_uniform_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu mesh order signature cleanup",
        [
            "runtime_15_gpu_mesh_order_signature_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_mesh/gpu_mesh_resource.rs",
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs",
            "runtime_15_gpu_mesh_order_signature_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 gpu model identity cleanup",
        [
            "runtime_15_gpu_model_identity_cleanup_coremin_check_passed",
            "graphics/scene/resources/gpu_model/gpu_model_resource.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_gpu_model_identity_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 post-process LUT texture owner cleanup",
        [
            "runtime_15_post_process_lut_texture_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/post_process_lut_texture/post_process_lut_texture_resource.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_post_process_lut_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 output target texture owner cleanup",
        [
            "runtime_15_output_target_texture_owner_cleanup_coremin_check_passed",
            "graphics/scene/resources/output_target_texture/output_target_texture_resource.rs",
            "graphics/scene/resources/prepared/prepared_output_target_texture.rs",
            "runtime_15_output_target_texture_owner_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 material runtime capture seed cleanup",
        [
            "runtime_15_material_runtime_capture_seed_cleanup_coremin_check_passed",
            "graphics/scene/resources/runtime/material_runtime.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "runtime_15_material_runtime_capture_seed_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 resource streamer diagnostics accessor cleanup",
        [
            "runtime_15_resource_streamer_diagnostics_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "resource_streamer_ensure_scene_resources.rs",
            "runtime_15_resource_streamer_diagnostics_accessor_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 resource streamer resolve texture id cleanup",
        [
            "runtime_15_resource_streamer_resolve_texture_id_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs",
            "resolve_texture_reference_with_support",
            "runtime_15_resource_streamer_resolve_texture_id_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 particle GPU readback output accessor cleanup",
        [
            "runtime_15_particle_gpu_readback_output_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/take_last_particle_gpu_readback_outputs.rs",
            "renderer.take_last_particle_gpu_readback_outputs()",
            "runtime_15_particle_gpu_readback_output_accessor_cleanup",
        ],
    ),
    (
        "Runtime 15 F12 advanced plugin output test accessor cleanup",
        [
            "runtime_15_advanced_plugin_output_test_accessor_cleanup_static_passed_cargo_lock_blocked",
            "graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/output_access.rs",
            "has_particle_gpu_readback",
            "runtime_15_advanced_plugin_output_test_accessor_cleanup",
        ],
    ),
    (
        "Runtime 15 M3 graphics dead-code guard module split",
        [
            "runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
            "graphics_dead_code/module_layout.rs",
            "graphics_dead_code/renderer_output_accessors.rs",
            "runtime_15_graphics_dead_code_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 provider boilerplate guard module split",
        [
            "runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/provider_boilerplate.rs",
            "runtime_15_provider_boilerplate_guard_is_folder_backed",
            "runtime_15_provider_registration_uses_shared_owner",
        ],
    ),
    (
        "Runtime 15 M3 facade surface guard module split",
        [
            "runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/facade_surface.rs",
            "runtime_15_facade_surface_guard_is_folder_backed",
            "runtime_15_prelude_covers_required_types",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code guard module split",
        [
            "runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/runtime_dead_code.rs",
            "runtime_15_runtime_dead_code_guard_is_folder_backed",
            "runtime_15_runtime_ui_dead_code_surface_is_test_support",
        ],
    ),
    (
        "Runtime 15 M3 diagnostics guard module split",
        [
            "runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/diagnostics_surface.rs",
            "runtime_15_diagnostics_guard_is_folder_backed",
            "runtime_15_diagnostics_use_frame_trait_without_world_wrapper",
        ],
    ),
];

pub(super) const EXPECTED_STATUS_OUTPUT_SLICE_GROUPS: &[&[ExpectedStatusOutputSlice]] = &[
    runtime_01_04::RUNTIME_01_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_01_04::RUNTIME_02_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_01_04::RUNTIME_03_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_01_04::RUNTIME_04_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_CROSS_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_SUPPORT_STRUCTURE_PLAN_STATUS_MODULE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_SUPPORT_STRUCTURE_STATUS_OUTPUT_SPLIT_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_SCENE_CLOSEOUT_DYNAMIC_SCENE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_SCENE_CLOSEOUT_FULL_SCENE_GATE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_SCENE_CLOSEOUT_SOURCE_GUARD_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_CARGO_EARLY_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_CARGO_LATE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_AUDIT_PLAN_COVERAGE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_AUDIT_RUNTIME_02_03_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_05::RUNTIME_05_AUDIT_RUNTIME_07_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_06_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_07_PERFORMANCE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_07_ASSET_RENDER_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_07_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_07_OWNER_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_08_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_09_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_09_LEGACY_RENAME_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_06_09::RUNTIME_09_LAYOUT_PIPELINE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_10_DYNAMIC_API_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_10_SESSION_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_10_UI_CONTRACT_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_11_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_12_BASELINE_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_12_ACTION_MAPPING_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_12_GAMEPAD_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_12_HOST_RECORDING_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_10_13::RUNTIME_13_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_14::RUNTIME_14_AUDIT_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_14::RUNTIME_14_GUARD_EXPECTED_STATUS_OUTPUT_SLICES,
    runtime_14::RUNTIME_14_CARGO_EXPECTED_STATUS_OUTPUT_SLICES,
    RUNTIME_15_EXPECTED_STATUS_OUTPUT_SLICES,
];
