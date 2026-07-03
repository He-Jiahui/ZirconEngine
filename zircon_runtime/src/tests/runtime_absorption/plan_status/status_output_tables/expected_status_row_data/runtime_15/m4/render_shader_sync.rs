type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M4 shader prewarm owner guard sync",
        &[
            "runtime_15_shader_prewarm_owner_guard_sync_static_passed_cargo_deferred",
            "tools/zircon_build_plugin_assets.py",
            "tools/zircon_build_plugin_shader_descriptors.py",
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs",
            "runtime_15_shader_prewarm_project_plugin_registry_runtime_staged_cache_hit_is_wired",
            "runtime_15_shader_prewarm_project_plugin_registry_product_staged_cache_is_wired",
        ],
    ),
    (
        "Runtime 15 M4 deferred GBuffer template output guard sync",
        &[
            "runtime_15_deferred_gbuffer_template_output_guard_sync_static_passed_cargo_deferred",
            "graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl",
            "graphics/shader/wgsl/zr_surface_types.wgsl",
            "runtime_15_deferred_gbuffer_pipeline_template_cache_is_mesh_cache_owned",
        ],
    ),
];
