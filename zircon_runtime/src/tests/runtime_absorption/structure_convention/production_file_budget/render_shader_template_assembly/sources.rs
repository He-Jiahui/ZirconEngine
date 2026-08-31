use super::*;

pub(super) struct RenderShaderTemplateAssemblySources {
    pub(super) shader_mod: String,
    pub(super) template_mod: String,
    pub(super) assemble: String,
    pub(super) module_registry: String,
    pub(super) module_registry_tests: String,
    pub(super) material_surface: String,
    pub(super) pass_specialization: String,
    pub(super) taa_reactive_template: String,
    pub(super) validation: String,
    pub(super) tests: String,
    pub(super) template_surface_module_tests: String,
    pub(super) template_standard_material_surface_tests: String,
    pub(super) variant_cache_prewarm: String,
    pub(super) variant_cache_prewarm_worker: String,
    pub(super) variant_cache_prewarm_tests: String,
    pub(super) variant_cache_prewarm_combined_tests: String,
    pub(super) pipeline_key: String,
    pub(super) mesh_cache_mod: String,
    pub(super) mesh_cache_state: String,
    pub(super) mesh_cache_ensure: String,
    pub(super) mesh_cache_ensure_tests: String,
    pub(super) mesh_cache_velocity: String,
    pub(super) mesh_cache_taa: String,
    pub(super) mesh_cache_shadow: String,
    pub(super) mesh_cache_source: String,
    pub(super) mesh_cache_source_tests: String,
    pub(super) mesh_pipeline_mod: String,
    pub(super) mesh_pipeline_test_support: String,
    pub(super) mesh_pipeline_velocity: String,
    pub(super) mesh_pipeline_taa: String,
    pub(super) mesh_pipeline_shadow: String,
    pub(super) shadow_processor: String,
    pub(super) non_material_rebuild: String,
    pub(super) shadow_renderer: String,
    pub(super) shadow_mod: String,
    pub(super) graph_gpu_context: String,
    pub(super) graph_gpu_mesh_recording: String,
    pub(super) graph_gpu_reports: String,
    pub(super) graph_stage_execution: String,
}

pub(super) fn read_render_shader_template_assembly_sources() -> RenderShaderTemplateAssemblySources
{
    RenderShaderTemplateAssemblySources {
        shader_mod: read_runtime_src("graphics/shader/mod.rs"),
        template_mod: read_runtime_src("graphics/shader/template/mod.rs"),
        assemble: read_runtime_src("graphics/shader/template/assemble.rs"),
        module_registry: read_runtime_src("graphics/shader/template/module_registry.rs"),
        module_registry_tests: read_runtime_src(
            "graphics/shader/template/module_registry/tests.rs",
        ),
        material_surface: read_runtime_src("graphics/shader/template/material_surface.rs"),
        pass_specialization: read_runtime_src("graphics/shader/template/pass_specialization.rs"),
        taa_reactive_template: read_runtime_src("graphics/shader/template/taa_reactive_mask.rs"),
        validation: read_runtime_src("graphics/shader/template/validation.rs"),
        tests: read_runtime_src("graphics/shader/template/tests.rs"),
        template_surface_module_tests: read_runtime_src(
            "graphics/shader/template/tests/surface_modules.rs",
        ),
        template_standard_material_surface_tests: read_runtime_src(
            "graphics/shader/template/tests/standard_material_surface_template.rs",
        ),
        variant_cache_prewarm: read_runtime_src("graphics/shader/variant_cache/prewarm.rs"),
        variant_cache_prewarm_worker: read_runtime_src(
            "graphics/shader/variant_cache/prewarm/worker.rs",
        ),
        variant_cache_prewarm_tests: read_runtime_src(
            "graphics/shader/variant_cache/prewarm/tests.rs",
        ),
        variant_cache_prewarm_combined_tests: read_runtime_src(
            "graphics/shader/variant_cache/prewarm/tests/combined_validation_tests.rs",
        ),
        pipeline_key: read_runtime_src("graphics/scene/resources/pipeline/pipeline_key.rs"),
        mesh_cache_mod: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs",
        ),
        mesh_cache_state: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
        ),
        mesh_cache_ensure: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs",
        ),
        mesh_cache_ensure_tests: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs",
        ),
        mesh_cache_velocity: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs",
        ),
        mesh_cache_taa: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs",
        ),
        mesh_cache_shadow: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs",
        ),
        mesh_cache_source: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
        ),
        mesh_cache_source_tests: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs",
        ),
        mesh_pipeline_mod: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/mod.rs",
        ),
        mesh_pipeline_test_support: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs",
        ),
        mesh_pipeline_velocity: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs",
        ),
        mesh_pipeline_taa: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs",
        ),
        mesh_pipeline_shadow: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs",
        ),
        shadow_processor: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/mesh_pass/processors/shadow.rs",
        ),
        non_material_rebuild: read_runtime_src(
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs",
        ),
        shadow_renderer: read_runtime_src(
            "graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs",
        ),
        shadow_mod: read_runtime_src("graphics/scene/scene_renderer/shadow/mod.rs"),
        graph_gpu_context: read_runtime_src(
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
        ),
        graph_gpu_mesh_recording: read_runtime_src(
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/mesh_recording.rs",
        ),
        graph_gpu_reports: read_runtime_src(
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/reports.rs",
        ),
        graph_stage_execution: read_runtime_src(
            "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs",
        ),
    }
}
