type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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
        "Runtime 15 M2 render framework trait/construction owner naming hard cutover",
        &[
            "runtime_15_render_framework_trait_construction_owner_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/runtime/render_framework/render_framework_trait_binding/wgpu_framework.rs",
            "graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs",
            "runtime_15_no_banned_name_modules",
        ],
    ),
    (
        "Runtime 15 M2 graphics construction new owner naming hard cutover",
        &[
            "runtime_15_graphics_construction_new_owner_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/feature/render_feature_descriptor/construct.rs",
            "graphics/scene/scene_renderer/post_process/resources/construct/construct/construct.rs",
            "runtime_15_graphics_construction_new_owners_use_construct_names",
            "runtime_15_no_banned_name_modules",
        ],
    ),
    (
        "Runtime 15 M2 scene dynamic document v1 owner naming hard cutover",
        &[
            "runtime_15_scene_dynamic_document_v1_owner_naming_hard_cutover_static_passed_cargo_deferred",
            "scene/dynamic_scene/document/v1_project_document.rs",
            "V1ProjectDocument",
            "runtime_15_scene_dynamic_document_v1_owner_uses_versioned_name",
        ],
    ),
    (
        "Runtime 15 M2 scene render layer schema-v1 mask naming hard cutover",
        &[
            "runtime_15_scene_render_layer_schema_v1_mask_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/camera.rs",
            "scene/world/render.rs",
            "scene/world/render_particles.rs",
            "from_scene_schema_v1_mask",
            "runtime_15_scene_render_layer_schema_v1_masks_use_versioned_names",
        ],
    ),
    (
        "Runtime 15 M2 render layer schema-v1 mask API naming hard cutover",
        &[
            "runtime_15_render_layer_schema_v1_mask_api_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/camera.rs",
            "graphics/scene/scene_renderer/lighting/light_buffer.rs",
            "graphics/runtime/render_framework/viewport_record/camera_history_key.rs",
            "from_scene_schema_v1_mask",
            "to_scene_schema_v1_mask_lossy",
            "intersects_scene_schema_v1_mask",
            "runtime_15_render_layer_schema_v1_mask_api_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 render shader definition bare-flag naming hard cutover",
        &[
            "runtime_15_render_shader_definition_bare_flag_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/shader/definition_value.rs",
            "BareFlag",
            "runtime_15_render_shader_definition_uses_bare_flag_names",
        ],
    ),
    (
        "Runtime 15 M2 GPU model embedded primitive naming hard cutover",
        &[
            "runtime_15_gpu_model_embedded_primitive_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs",
            "embedded primitive",
            "model_render_primitives_keep_embedded_payload_when_mesh_reference_unresolved",
            "runtime_15_gpu_model_embedded_primitive_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 frame extract snapshot adapter naming hard cutover",
        &[
            "runtime_15_frame_extract_snapshot_adapter_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/frame_extract.rs",
            "scene viewport snapshot packet",
            "RenderFrameExtract::from_snapshot",
            "runtime_15_frame_extract_snapshot_adapter_uses_current_names",
        ],
    ),
    (
        "Runtime 15 M2 core framework render fixture naming hard cutover",
        &[
            "runtime_15_core_framework_render_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "core/framework/render/core_pipeline/render_queue.rs",
            "scene_schema_v1_mask",
            "extended_effect_stack_settings_enable_product_node_without_retired_fields",
            "runtime_15_core_framework_render_fixtures_use_current_names",
        ],
    ),
    (
        "Runtime 15 M2 render feature fallback capability naming hard cutover",
        &[
            "runtime_15_render_feature_fallback_capability_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs",
            "graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs",
            "fallback-virtual-geometry-without-capability",
            "runtime_15_render_feature_fallback_capability_fixtures_use_current_names",
        ],
    ),
    (
        "Runtime 15 M2 render material stale texture fixture naming hard cutover",
        &[
            "runtime_15_render_material_stale_texture_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/render_product_streamer_tests/material_runtime.rs",
            "unresolved_stale_texture",
            "res://textures/missing-stale-base.png",
            "runtime_15_render_material_stale_texture_fixtures_use_current_names",
        ],
    ),
    (
        "Runtime 15 M2 render graph fallback fixture naming hard cutover",
        &[
            "runtime_15_render_graph_fallback_fixture_naming_hard_cutover_static_passed_cargo_deferred",
            "graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs",
            "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs",
            "unexpected-compute",
            "runtime_15_render_graph_fallback_fixtures_use_current_names",
        ],
    ),
    (
        "Runtime 15 M2 Hybrid GI extract scene-source naming hard cutover",
        &[
            "runtime_15_hybrid_gi_extract_scene_source_naming_hard_cutover_static_passed_cargo_deferred",
            "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi",
            "extract_trace_region_ids",
            "extract-backed",
            "extract-sourced RenderHybridGiProbe",
            "runtime_15_hybrid_gi_extract_scene_source_uses_current_names",
        ],
    ),
];
