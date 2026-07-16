use super::*;

#[test]
fn runtime_15_builtin_fallback_prewarm_uses_template_source() {
    let dynamic_api = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let dynamic_api_tests = read_runtime_src("dynamic_api/shader_prewarm/tests.rs");
    let scene_mod = read_runtime_src("graphics/scene/mod.rs");
    let scene_renderer_mod = read_runtime_src("graphics/scene/scene_renderer/mod.rs");
    let mesh_mod = read_runtime_src("graphics/scene/scene_renderer/mesh/mod.rs");
    let mesh_cache_mod =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs");
    let mesh_cache_ensure = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs",
    );
    let mesh_cache_ensure_tests = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline/tests.rs",
    );
    let mesh_cache_source =
        read_runtime_src("graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let mesh_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "dynamic API builtin fallback prewarm uses mesh template source",
        &(dynamic_api.clone() + &dynamic_api_tests),
        &[
            "mesh_pipeline_standard_material_template_source",
            "BUILTIN_STANDARD_MATERIAL_PREWARM_PASSES",
            "MeshPipelineShaderSource",
            "cache_content_hashes",
            "template_revision",
            "ShaderVariantPrewarmManifest::new(Vec::new())",
            "builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source",
            "zr-material-template-v1",
        ],
    );
    assert!(
        !dynamic_api.contains("FALLBACK_MESH_SHADER"),
        "dynamic_api/shader_prewarm.rs should not prewarm the legacy monolithic fallback shader source"
    );
    for (label, source) in [
        ("graphics scene facade", scene_mod.as_str()),
        ("scene renderer facade", scene_renderer_mod.as_str()),
        ("mesh facade", mesh_mod.as_str()),
        ("mesh pipeline cache facade", mesh_cache_mod.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "mesh_pipeline_standard_material_template_source",
                "mesh_pipeline_standard_material_template_source_for_shader_pass",
                "MeshPipelineShaderSource",
            ],
        );
    }
    assert_contains_all(
        "mesh cache source owner DTO is shared with prewarm",
        &mesh_cache_source,
        &[
            "pub(crate) struct MeshPipelineShaderSource",
            "pub(crate) fn mesh_pipeline_standard_material_template_source",
            "pub(crate) fn mesh_pipeline_standard_material_template_source_for_shader_pass",
            "pub(crate) cache_content_hashes",
            "pub(crate) template_revision",
        ],
    );
    assert_contains_all(
        "mesh cache ensure delegates source assembly and proves runtime staged hits",
        &(mesh_cache_ensure.clone() + &mesh_cache_ensure_tests),
        &[
            "mesh_pipeline_shader_source",
            "MeshPipelineShaderSource",
            "runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss",
            "prewarm_shader_variants_to_disk",
            "ShaderVariantCacheDisk::with_fallback_roots",
            "ensure_pipeline_for_variant",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
            "disk_hit_count",
            "compile_miss_count",
        ],
    );

    for (path, source) in [
        ("dynamic_api/shader_prewarm.rs", dynamic_api.as_str()),
        ("graphics/scene/mod.rs", scene_mod.as_str()),
        (
            "graphics/scene/scene_renderer/mod.rs",
            scene_renderer_mod.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mod.rs",
            mesh_mod.as_str(),
        ),
        (
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs",
            mesh_cache_mod.as_str(),
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
            "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs",
            mesh_cache_source.as_str(),
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
        ("mesh pipeline cache doc", mesh_cache_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Builtin fallback prewarm template source alignment",
                "render_plan08_builtin_fallback_prewarm_template_source_static_passed_cargo_deferred_implementation_cadence",
                "render_plan08_runtime_base_mesh_staged_prewarm_cache_hit_wgpu_pipeline_passed_renderdoc_deferred",
                "Runtime Base mesh staged prewarm cache hit",
                "dynamic_api/shader_prewarm.rs",
                "mesh_pipeline_standard_material_template_source",
                "builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source",
                "runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss",
                "runtime_15_builtin_fallback_prewarm_uses_template_source",
            ],
        );
    }
}
