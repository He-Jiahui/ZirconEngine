use super::*;

#[test]
fn runtime_15_depth_prepass_pipeline_template_cache_is_mesh_cache_owned() {
    let depth_processor = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pass/processors/depth_prepass.rs",
    );
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
    let shader_source_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs",
    );
    let ensure_depth = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs",
    );
    let depth_pipeline = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs",
    );
    let graph_gpu_context = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
    );
    let core_state = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core/scene_renderer_core.rs",
    );
    let core_construct = read_runtime_src(
        "graphics/scene/scene_renderer/core/scene_renderer_core_construct/construct/construct.rs",
    );
    let prepass_mod = read_runtime_src("graphics/scene/scene_renderer/prepass/mod.rs");

    assert_contains_all(
        "depth prepass processor resolves a real cache-backed variant id",
        &depth_processor,
        &[
            "let pipeline_kind = MeshPassPipelineKind::DepthPrepass;",
            "context.pipeline_variant_id(pipeline_kind, batch)",
        ],
    );
    assert!(
        !depth_processor.contains("MeshPipelineVariantId::new(0)"),
        "depth prepass processor should not keep the old fixed variant id"
    );
    assert_contains_all(
        "mesh pipeline cache mounts depth prepass source/cache owners",
        &mesh_cache_mod,
        &[
            "mod ensure_depth_prepass_pipeline;",
            "mesh_pipeline_depth_prepass_template_source_for_geometry",
        ],
    );
    assert_contains_all(
        "depth prepass mesh cache state is keyed by variant id",
        &mesh_cache_state,
        &[
            "depth_prepass_mesh_pipelines:\n        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>",
            "pipeline_and_shader_key_for_variant",
        ],
    );
    assert_contains_all(
        "depth prepass runtime variant maps to the DepthPrepass shader pass",
        &variant_registry,
        &[
            "MeshPassPipelineKind::DepthPrepass => ShaderPassType::DepthPrepass",
            "mesh_pipeline_variant_registry_maps_depth_prepass_to_depth_prepass_pass_type",
        ],
    );
    assert_contains_all(
        "depth prepass shader source uses template source hash contract",
        &shader_source,
        &[
            "mesh_pipeline_depth_prepass_template_source_for_geometry",
            "ShaderPassType::DepthPrepass",
        ],
    );
    assert_contains_all(
        "depth prepass shader source tests keep depth-only template coverage",
        &shader_source_tests,
        &[
            "mesh_pipeline_depth_prepass_template_source_uses_depth_only_template",
            "mesh_pipeline_standard_material_shader_pass_source_keeps_depth_only_contract",
        ],
    );
    assert_contains_all(
        "depth prepass ensure path reuses shared disk/source cache helper",
        &ensure_depth,
        &[
            "DEPTH_PREPASS_MESH_SHADER_KEY_PREFIX",
            "mesh_pipeline_depth_prepass_template_source_for_geometry",
            "mesh_pipeline_shader_source_with_cache",
            "depth_prepass_mesh_shader_key",
            "variant_key.canonical_string()",
            "ensure_depth_prepass_pipeline_for_variant",
        ],
    );
    assert_contains_all(
        "depth prepass WGPU pipeline declares depth-only template entries and static layout",
        &depth_pipeline,
        &[
            "entry_point: Some(\"vs_main\")",
            "key.is_alpha_mask()",
            "targets: &[]",
            "depth_write_enabled: Some(true)",
            "GpuMeshVertex::layout()",
            "depth_prepass_mesh_pipeline_declares_depth_only_template_entries_and_static_layout",
            "depth_prepass_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "GpuScene::new",
        ],
    );
    assert_contains_all(
        "depth prepass graph execution replays through mesh pipeline cache",
        &graph_gpu_context,
        &[
            "record_depth_prepass_to_resources",
            "depth prepass graph executor for pass `{pass_name}` requires mesh pipeline context",
            "ensure_depth_prepass_pipeline_for_variant",
            "bind_standard_material_if_needed(pass, command)",
            "bind_geometry_if_needed(pass, command)",
        ],
    );
    for source in [&core_state, &core_construct, &prepass_mod] {
        assert!(
            !source.contains("NormalPrepassPipeline")
                && !source.contains("normal_prepass_pipeline")
                && !source.contains("normal_prepass_shader_source"),
            "renderer-local normal prepass pipeline owner should stay deleted after depth prepass cutover"
        );
    }

    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_template_doc = read_repo("docs/zircon_runtime/graphics/shader/template.md");
    let mesh_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader template doc", shader_template_doc.as_str()),
        ("mesh cache doc", mesh_cache_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "DepthPrepass normal-target template source cache cutover",
                "render_plan08_depth_prepass_template_source_cache_static_passed_cargo_check_wgpu_deferred",
                "render_plan08_depth_prepass_wgpu_device_pipeline_validation_passed_renderdoc_deferred",
                "depth_prepass_mesh_pipeline_creates_on_wgpu_device_with_template_shader",
                "ensure_depth_prepass_pipeline_for_variant",
                "create_depth_prepass_mesh_pipeline.rs",
            ],
        );
    }
}
