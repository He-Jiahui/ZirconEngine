use super::super::sources::RenderShaderTemplateAssemblySources;

pub(super) fn assert_render_shader_template_assembly_production_owners_stay_below_budget(
    sources: &RenderShaderTemplateAssemblySources,
) {
    let RenderShaderTemplateAssemblySources {
        shader_mod,
        template_mod,
        assemble,
        module_registry,
        material_surface,
        pass_specialization,
        taa_reactive_template,
        validation,
        tests,
        template_surface_module_tests,
        variant_cache_prewarm,
        pipeline_key,
        mesh_cache_state,
        mesh_cache_ensure,
        mesh_cache_ensure_tests,
        mesh_cache_velocity,
        mesh_cache_taa,
        mesh_cache_shadow,
        mesh_cache_source,
        mesh_cache_source_tests,
        mesh_pipeline_mod,
        mesh_pipeline_test_support,
        mesh_pipeline_velocity,
        mesh_pipeline_taa,
        mesh_pipeline_shadow,
        shadow_processor,
        non_material_rebuild,
        shadow_renderer,
        graph_gpu_context,
        graph_stage_execution,
        ..
    } = sources;

    for (path, source) in [
        ("graphics/shader/mod.rs", shader_mod.as_str()),
        ("graphics/shader/template/mod.rs", template_mod.as_str()),
        ("graphics/shader/template/assemble.rs", assemble.as_str()),
        (
            "graphics/shader/template/module_registry.rs",
            module_registry.as_str(),
        ),
        (
            "graphics/shader/template/material_surface.rs",
            material_surface.as_str(),
        ),
        (
            "graphics/shader/template/pass_specialization.rs",
            pass_specialization.as_str(),
        ),
        (
            "graphics/shader/template/taa_reactive_mask.rs",
            taa_reactive_template.as_str(),
        ),
        (
            "graphics/shader/template/validation.rs",
            validation.as_str(),
        ),
        ("graphics/shader/template/tests.rs", tests.as_str()),
        (
            "graphics/shader/template/tests/surface_modules.rs",
            template_surface_module_tests.as_str(),
        ),
        (
            "graphics/shader/variant_cache/prewarm.rs",
            variant_cache_prewarm.as_str(),
        ),
        (
            "graphics/scene/resources/pipeline/pipeline_key.rs",
            pipeline_key.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs",
            mesh_cache_ensure.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs",
            mesh_cache_ensure_tests.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs",
            mesh_cache_velocity.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs",
            mesh_cache_taa.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs",
            mesh_cache_shadow.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
            mesh_cache_state.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            mesh_cache_source.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs",
            mesh_cache_source_tests.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/mod.rs",
            mesh_pipeline_mod.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs",
            mesh_pipeline_test_support.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs",
            mesh_pipeline_velocity.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs",
            mesh_pipeline_taa.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs",
            mesh_pipeline_shadow.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pass/processors/shadow.rs",
            shadow_processor.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_command_cache_extract/non_material_rebuild.rs",
            non_material_rebuild.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs",
            shadow_renderer.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
            graph_gpu_context.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs",
            graph_stage_execution.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below R4.3 production/test owner budget; got {line_count}"
        );
    }
}
