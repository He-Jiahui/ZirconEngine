use super::*;

#[test]
fn runtime_15_deferred_gbuffer_pipeline_template_cache_is_mesh_cache_owned() {
    let template_mod = read_runtime_src("graphics/shader/template/mod.rs");
    let deferred_gbuffer_template =
        read_runtime_src("graphics/shader/template/deferred_gbuffer.rs");
    let deferred_gbuffer_wgsl =
        read_runtime_src("graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl");
    let mesh_cache_mod =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs");
    let mesh_cache_state = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
    );
    let variant_registry = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
    );
    let shader_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs");
    let ensure_gbuffer = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs",
    );
    let gbuffer_pipeline = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs",
    );
    let deferred_record = read_runtime_src(
        "graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs",
    );
    let deferred_resources = read_runtime_src(
        "graphics/scene/scene_renderer/deferred/deferred_scene_resources/deferred_scene_resources.rs",
    );
    let deferred_mod = read_runtime_src("graphics/scene/scene_renderer/deferred/mod.rs");
    let scene_passes = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs",
    );

    assert_contains_all(
        "deferred gbuffer template is a first-class shader template assembler",
        &template_mod,
        &[
            "mod deferred_gbuffer;",
            "assemble_deferred_gbuffer_shader_template",
            "DeferredGBufferShaderTemplateRequest",
        ],
    );
    assert_contains_all(
        "deferred gbuffer assembler reuses scene/gpu/geometry/material chunks",
        &deferred_gbuffer_template,
        &[
            "DEFERRED_GBUFFER_TEMPLATE_TOKEN",
            "zr_template_deferred_gbuffer.wgsl",
            "scene_runtime_include()",
            "gpu_scene_include()",
            "surface_types_include()",
            "geometry_source_include_for",
            "rename_material_surface_entry",
        ],
    );
    assert_contains_all(
        "deferred gbuffer WGSL writes albedo and material targets",
        &deferred_gbuffer_wgsl,
        &[
            "struct ZrDeferredGBufferOutput",
            "@location(0) albedo",
            "@location(1) material",
            "zr_deferred_encode_material_flags(surface.shading_model_id",
            "zr_surface_fails_alpha_clip(surface)",
        ],
    );
    assert_contains_all(
        "mesh pipeline cache mounts deferred gbuffer source/cache owners",
        &mesh_cache_mod,
        &[
            "mod ensure_gbuffer_pipeline;",
            "mesh_pipeline_deferred_gbuffer_template_source_for_geometry",
        ],
    );
    assert_contains_all(
        "deferred gbuffer mesh cache state is keyed by variant id",
        &mesh_cache_state,
        &[
            "gbuffer_mesh_pipelines:\n        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>",
            "pipeline_and_shader_key_for_variant",
        ],
    );
    assert_contains_all(
        "deferred gbuffer runtime variant maps to the GBuffer shader pass",
        &variant_registry,
        &[
            "MeshPassPipelineKind::GBuffer => ShaderPassType::GBuffer",
            "mesh_pipeline_variant_registry_maps_deferred_gbuffer_to_gbuffer_pass_type",
        ],
    );
    assert_contains_all(
        "deferred gbuffer shader source uses template source hash contract",
        &shader_source,
        &[
            "mesh_pipeline_deferred_gbuffer_template_source_for_geometry",
            "DeferredGBufferShaderTemplateRequest",
            "mesh_pipeline_deferred_gbuffer_template_source_writes_albedo_and_material_targets",
        ],
    );
    assert_contains_all(
        "deferred gbuffer ensure path reuses shared disk/source cache helper",
        &ensure_gbuffer,
        &[
            "GBUFFER_MESH_SHADER_KEY_PREFIX",
            "gbuffer_variant_id_for_command_variant",
            "mesh_pipeline_deferred_gbuffer_template_source_for_geometry",
            "mesh_pipeline_shader_source_with_cache",
            "gbuffer_mesh_shader_key",
            "variant_key.canonical_string()",
            "ensure_gbuffer_pipeline_for_variant",
        ],
    );
    assert_contains_all(
        "deferred gbuffer WGPU pipeline writes albedo/material and reads depth",
        &gbuffer_pipeline,
        &[
            "GBUFFER_ALBEDO_FORMAT",
            "GBUFFER_MATERIAL_FORMAT",
            "depth_write_enabled: Some(false)",
            "entry_point: Some(\"vs_main\")",
            "entry_point: Some(\"fs_main\")",
            "GpuMeshVertex::layout()",
            "gbuffer_mesh_pipeline_declares_albedo_material_targets_and_static_layout",
            "gbuffer_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "GpuScene::new",
        ],
    );
    assert_contains_all(
        "deferred gbuffer graph execution replays through mesh pipeline cache",
        &deferred_record,
        &[
            "record_gbuffer_geometry",
            "gbuffer_variant_id_for_command_variant(command.pipeline_variant_id)",
            "ensure_gbuffer_pipeline_for_variant(device, gbuffer_variant_id)",
            "MeshPassPipelineKind::GBuffer",
            "bind_standard_material_if_needed(pass, command)",
            "bind_geometry_if_needed(pass, command)",
        ],
    );
    assert_contains_all(
        "deferred graph stage receives mesh pipeline context",
        &scene_passes,
        &[
            "Some(&mut self.mesh_pipelines)",
            "RenderPassStage::Deferred",
            "mesh_pipelines: Option<&mut MeshPipelineCache>",
            "streamer: Option<&ResourceStreamer>",
        ],
    );
    for source in [&deferred_mod, &deferred_resources, &scene_passes] {
        assert!(
            !source.contains("geometry_pipeline")
                && !source.contains("DEFERRED_GEOMETRY_SHADER")
                && !source.contains("deferred_geometry.wgsl")
                && !source.contains("create_geometry_pipeline"),
            "renderer-local deferred geometry pipeline owner should stay deleted after GBuffer cutover"
        );
    }

    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_template_doc = read_repo("docs/zircon_runtime/graphics/shader/template.md");
    let mesh_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");
    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader template doc", shader_template_doc.as_str()),
        ("mesh cache doc", mesh_cache_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Deferred GBuffer template source cache cutover",
                "render_plan08_deferred_gbuffer_template_source_cache_static_passed_cargo_check_wgpu_deferred",
                "render_plan08_deferred_gbuffer_wgpu_device_pipeline_validation_passed_renderdoc_deferred",
                "ensure_gbuffer_pipeline_for_variant",
                "create_gbuffer_mesh_pipeline.rs",
                "gbuffer_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            ],
        );
    }
}
