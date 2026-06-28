use super::*;

#[test]
fn runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired() {
    let dynamic_api = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let dynamic_mod = read_runtime_src("dynamic_api/mod.rs");
    let scene_mod = read_runtime_src("graphics/scene/mod.rs");
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let template_doc = read_repo("docs/zircon_runtime/graphics/shader/template.md");
    let mesh_cache_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "dynamic API exposes neutral builtin standard material template prewarm builder",
        &dynamic_api,
        &[
            "pub fn builtin_standard_material_shader_prewarm_manifest",
            "pub fn builtin_standard_material_shader_prewarm_manifest_for_geometry",
            "ShaderFeatureBits",
            "ShaderQualityTier",
            "ShadingModelId",
            "GeometrySourceId",
            "alpha_cutoff",
            "BUILTIN_STANDARD_MATERIAL_PREWARM_PASSES",
            "mesh_pipeline_standard_material_template_source_for_shader_pass",
            "ShaderPassType::DepthPrepass",
            "builtin_standard_material_shader_prewarm_manifest_projects_material_features",
            "builtin_standard_material_shader_prewarm_manifest_projects_geometry_source",
            "builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules",
            "prewarm_shader_variants",
            "ShaderVariantCacheDiskKey::from_variant_key",
            "ShaderVariantCacheDiskLookup::Hit",
            "create_shader_module",
            "push_error_scope(wgpu::ErrorFilter::Validation)",
        ],
    );
    assert_contains_all(
        "dynamic API module re-exports builtin standard material template prewarm builder",
        &dynamic_mod,
        &[
            "builtin_standard_material_shader_prewarm_manifest",
            "builtin_standard_material_shader_prewarm_manifest_for_geometry",
        ],
    );
    assert_contains_all(
        "scene facade exposes PipelineKey only crate-wide for dynamic API source building",
        &scene_mod,
        &["pub(crate) use resources::{default_pipeline_key, PipelineKey};"],
    );
    assert_contains_all(
        "asset-root manifest uses dynamic template source only for builtin standard materials",
        &manifest,
        &[
            "BUILTIN_STANDARD_MATERIAL_SHADER_URI",
            "builtin_standard_material_shader_prewarm_manifest_for_geometry",
            "uses_builtin_standard_shader",
            "material_alpha_cutoff",
            "material.receive_shadows()",
            ".flat_map(|geometry_source|",
            "geometry_source",
        ],
    );
    assert_contains_all(
        "asset-root manifest tests cover builtin standard material template source",
        &tests,
        &[
            "shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source",
            "builtin://shader/pbr.wgsl",
            "fn zr_material_surface(",
            "zr_geometry_skinned.wgsl",
            "ZR_GEOMETRY_SOURCE_SKINNED_MESH",
            "ZR_STANDARD_MATERIAL_ALPHA_CUTOFF",
            "zr_template_depth_alpha.wgsl",
            "ShaderPassType::DepthPrepass",
            "ShaderFeatureBits::RECEIVE_SHADOWS",
        ],
    );

    for (path, source) in [
        ("dynamic_api/shader_prewarm.rs", dynamic_api.as_str()),
        ("dynamic_api/mod.rs", dynamic_mod.as_str()),
        ("graphics/scene/mod.rs", scene_mod.as_str()),
        ("bin/zircon_shader_prewarm/manifest.rs", manifest.as_str()),
        (
            "bin/zircon_shader_prewarm/manifest/tests.rs",
            tests.as_str(),
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
        ("template doc", template_doc.as_str()),
        ("mesh pipeline cache doc", mesh_cache_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Asset-root builtin standard material template prewarm",
                "render_plan08_asset_root_builtin_standard_material_template_prewarm_static_passed_cargo_deferred_implementation_cadence",
                "render_plan08_asset_root_builtin_standard_material_multi_geometry_prewarm_static_passed_cargo_deferred_implementation_cadence",
                "render_plan08_builtin_material_multi_pass_depth_only_prewarm_tests_passed_renderdoc_deferred",
                "render_plan08_builtin_material_staged_prewarm_cache_hit_wgpu_module_passed_renderdoc_deferred",
                "dynamic_api::builtin_standard_material_shader_prewarm_manifest",
                "dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry",
                "builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules",
                "shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source",
                "runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired",
            ],
        );
    }
}
