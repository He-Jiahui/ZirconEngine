type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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
        "Runtime 15 F12 ResourceStreamer material diagnostics child owner split",
        &[
            "runtime_15_resource_streamer_material_diagnostics_child_owner_split_static_passed_cargo_deferred",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
            "graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_diagnostics.rs",
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
