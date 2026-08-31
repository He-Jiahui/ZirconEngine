use super::*;

const STATUS: &str = "render_plan08_non_base_mesh_variant_cache_owner_static_passed_cargo_deferred";

#[test]
fn runtime_15_non_base_mesh_variant_cache_owner_is_wired() {
    let cache = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
    );
    let registry = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
    );
    let pipeline_key = read_runtime_src("graphics/scene/resources/pipeline/pipeline_key.rs");
    let prewarm_contract = read_runtime_src("core/framework/render/shader/variant_prewarm.rs");
    let depth = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs",
    );
    let gbuffer = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs",
    );
    let shadow = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs",
    );
    let taa = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs",
    );
    let velocity = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs",
    );
    let gbuffer_record = read_runtime_src(
        "graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs",
    );
    let graph_gpu = read_runtime_src(
        "graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs",
    );
    let shadow_renderer =
        read_runtime_src("graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs");
    let velocity_execute = read_runtime_src(
        "graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs",
    );
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "mesh pipeline cache stores all pass pipelines by resolved variant id",
        &cache,
        &[
            "gbuffer_mesh_pipelines:",
            "depth_prepass_mesh_pipelines:",
            "velocity_mesh_pipelines:",
            "shadow_mesh_pipelines:",
            "taa_reactive_mask_mesh_pipelines:",
            "taa_reactive_material_mask_mesh_pipelines:",
            "HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>",
            "pipeline_and_shader_key_for_variant",
            "Option<(MeshPassPipelineKind, PipelineKey, ShaderVariantKey)>",
        ],
    );
    assert_contains_all(
        "mesh pipeline variant registry maps every material pass to ShaderVariantKey",
        &registry,
        &[
            "shader_pass_type_for_mesh_pipeline_kind",
            "MeshPassPipelineKind::GBuffer => ShaderPassType::GBuffer",
            "MeshPassPipelineKind::DepthPrepass => ShaderPassType::DepthPrepass",
            "MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask",
            "ShaderPassType::Shadow",
            "MeshPassPipelineKind::Velocity => ShaderPassType::Velocity",
            "MeshPassPipelineKind::TaaReactiveMask | MeshPassPipelineKind::TaaReactiveMaterialMask",
            "ShaderPassType::TaaReactiveMask",
        ],
    );
    for (label, source) in [
        ("PipelineKey", pipeline_key.as_str()),
        ("runtime prewarm pipeline state", prewarm_contract.as_str()),
    ] {
        for binding_presence in [
            "has_base_color_texture",
            "has_metallic_roughness_texture",
            "has_occlusion_texture",
            "has_emissive_texture",
        ] {
            assert!(
                !source.contains(binding_presence),
                "{label} must not treat fallback-backed `{binding_presence}` as pipeline identity",
            );
        }
    }
    assert!(
        !registry.contains("has_texture_presence_equivalent_variant"),
        "mesh pipeline identity must normalize binding presence before lookup instead of scanning 16 equivalent keys",
    );

    assert_non_base_pass_cache_owner(
        "GBuffer pass cache owner",
        &gbuffer,
        "fn ensure_gbuffer_pipeline(",
        "pub(crate) fn ensure_gbuffer_pipeline_admission_for_variant(",
        "MeshPassPipelineKind::GBuffer",
        "gbuffer_mesh_shader_key(shader_variant_key, &shader_source.source_hash)",
        "self.gbuffer_mesh_pipelines.insert(variant_id, pipeline)",
    );
    assert_non_base_pass_cache_owner(
        "Depth prepass cache owner",
        &depth,
        "fn ensure_depth_prepass_pipeline<'a>(",
        "pub(crate) fn ensure_depth_prepass_pipeline_for_variant<'a>(",
        "MeshPassPipelineKind::DepthPrepass",
        "depth_prepass_mesh_shader_key(shader_variant_key, &shader_source.source_hash)",
        "self.depth_prepass_mesh_pipelines",
    );
    assert_non_base_pass_cache_owner(
        "Velocity pass cache owner",
        &velocity,
        "fn ensure_velocity_pipeline<'a>(",
        "pub(crate) fn ensure_velocity_pipeline_for_variant<'a>(",
        "MeshPassPipelineKind::Velocity",
        "velocity_mesh_shader_key(shader_variant_key, &shader_source.source_hash)",
        "self.velocity_mesh_pipelines.insert(variant_id, pipeline)",
    );
    assert_non_base_pass_cache_owner(
        "Shadow pass cache owner",
        &shadow,
        "fn ensure_shadow_pipeline<'a>(",
        "pub(crate) fn ensure_shadow_pipeline_for_variant<'a>(",
        "MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask",
        "shadow_mesh_shader_key(shader_variant_key, &shader_source.source_hash)",
        "self.shadow_mesh_pipelines.insert(variant_id, pipeline)",
    );
    assert_non_base_pass_cache_owner(
        "TAA reactive pass cache owner",
        &taa,
        "fn ensure_taa_reactive_mask_pipeline<'a>(",
        "pub(crate) fn ensure_taa_reactive_mask_pipeline_for_variant<'a>(",
        "MeshPassPipelineKind::TaaReactiveMask",
        "taa_reactive_mask_mesh_shader_key(shader_variant_key, &shader_source.source_hash)",
        "self.taa_reactive_mask_mesh_pipelines",
    );
    assert_contains_all(
        "TAA reactive material mask shares the variant-aware owner",
        &taa,
        &[
            "MeshPassPipelineKind::TaaReactiveMaterialMask",
            "fn ensure_taa_reactive_material_mask_pipeline<'a>(",
            "self.taa_reactive_material_mask_mesh_pipelines",
        ],
    );

    let non_base_render_paths = [
        gbuffer_record.as_str(),
        graph_gpu.as_str(),
        shadow_renderer.as_str(),
        velocity_execute.as_str(),
    ]
    .concat();
    assert_contains_all(
        "render pass execution calls only variant-aware non-Base pass entries",
        &non_base_render_paths,
        &[
            "ensure_gbuffer_pipeline_admission_for_variant",
            "ensure_depth_prepass_pipeline_for_variant",
            "ensure_taa_reactive_mask_pipeline_for_variant",
            ".ensure_shadow_pipeline_for_variant(",
            ".ensure_velocity_pipeline_for_variant(",
        ],
    );
    assert_no_direct_pipeline_call(
        "GBuffer render path",
        &gbuffer_record,
        ".ensure_gbuffer_pipeline(",
    );
    assert_no_direct_pipeline_call(
        "Depth render path",
        &graph_gpu,
        ".ensure_depth_prepass_pipeline(",
    );
    assert_no_direct_pipeline_call(
        "TAA reactive render path",
        &graph_gpu,
        ".ensure_taa_reactive_mask_pipeline(",
    );
    assert_no_direct_pipeline_call(
        "Shadow render path",
        &shadow_renderer,
        ".ensure_shadow_pipeline(",
    );
    assert_no_direct_pipeline_call(
        "Velocity render path",
        &velocity_execute,
        ".ensure_velocity_pipeline(",
    );

    for (path, source) in [
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs",
            depth.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs",
            gbuffer.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs",
            shadow.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs",
            taa.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs",
            velocity.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs",
            cache.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
            registry.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mesh_pipeline_variant_cache_owner.rs",
            include_str!("mesh_pipeline_variant_cache_owner.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Non-Base mesh variant-aware cache owner",
                STATUS,
                "runtime_15_non_base_mesh_variant_cache_owner_is_wired",
            ],
        );
    }
}

fn assert_non_base_pass_cache_owner(
    label: &str,
    source: &str,
    private_entry: &str,
    public_entry: &str,
    pass_kind: &str,
    shader_key_call: &str,
    pipeline_map: &str,
) {
    assert_contains_all(
        label,
        source,
        &[
            "ShaderVariantKey",
            "PipelineKey",
            "MeshPipelineVariantId",
            private_entry,
            public_entry,
            "self.pipeline_and_shader_key_for_variant(variant_id)?",
            pass_kind,
            shader_key_call,
            "variant_key.canonical_string()",
            "source_hash",
            pipeline_map,
        ],
    );
}

fn assert_no_direct_pipeline_call(label: &str, source: &str, direct_call: &str) {
    assert!(
        !source.contains(direct_call),
        "{label} must call the variant-aware pass entry instead of `{direct_call}`"
    );
}
