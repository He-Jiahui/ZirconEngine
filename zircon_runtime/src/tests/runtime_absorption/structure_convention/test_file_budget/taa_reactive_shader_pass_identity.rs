use super::*;

const STATUS: &str = "render_plan08_taa_reactive_shader_pass_identity_static_passed_cargo_deferred";

#[test]
fn runtime_15_taa_reactive_shader_pass_identity_is_wired() {
    let variant_key = read_runtime_src("core/framework/render/shader/variant_key.rs");
    let pass_specialization = read_runtime_src("graphics/shader/template/pass_specialization.rs");
    let registry = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
    );
    let taa_cache = read_runtime_src(
        "graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs",
    );
    let dynamic_api = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let prewarm_manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let prewarm_tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "framework shader pass key exposes a dedicated TAA reactive mask pass",
        &variant_key,
        &[
            "TaaReactiveMask",
            "Self::TaaReactiveMask => 5",
            "Self::TaaReactiveMask => \"taa_reactive_mask\"",
            "render_shader_pass_type_names_taa_reactive_mask_separately_from_forward",
        ],
    );
    assert_contains_all(
        "material template specialization can assemble the TAA reactive mask template",
        &pass_specialization,
        &[
            "TAA_REACTIVE_MASK_TEMPLATE_TOKEN",
            "zr_template_taa_reactive_mask.wgsl",
            "ShaderPassType::TaaReactiveMask => ShaderPassTemplate",
            "requires_shading_include: false",
        ],
    );
    assert_contains_all(
        "mesh pipeline variant registry maps TAA reactive kinds to the dedicated pass",
        &registry,
        &[
            "MeshPassPipelineKind::TaaReactiveMask | MeshPassPipelineKind::TaaReactiveMaterialMask",
            "ShaderPassType::TaaReactiveMask",
            "mesh_pipeline_variant_registry_maps_taa_reactive_to_taa_reactive_pass_type",
        ],
    );
    assert_contains_all(
        "TAA reactive shader module key records the dedicated pass identity",
        &taa_cache,
        &[
            "ShaderPassType::TaaReactiveMask",
            "|pass=taa_reactive_mask|",
            "taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash",
        ],
    );
    assert_contains_all(
        "built-in shader prewarm enumerates the TAA reactive pass",
        &dynamic_api,
        &[
            "BUILTIN_STANDARD_MATERIAL_PREWARM_PASSES: [ShaderPassType; 6]",
            "ShaderPassType::TaaReactiveMask",
            "zr_template_taa_reactive_mask.wgsl",
        ],
    );
    assert_contains_all(
        "asset-root shader prewarm enumerates the TAA reactive pass for full material sources",
        &prewarm_manifest,
        &[
            "ASSET_SCAN_FULL_MATERIAL_PASSES: [ShaderPassType; 6]",
            "ShaderPassType::TaaReactiveMask",
        ],
    );
    assert_contains_all(
        "manifest tests track the expanded standard material pass set",
        &prewarm_tests,
        &[
            "BUILTIN_MATERIAL_PASS_TYPES: [ShaderPassType; 6]",
            "ShaderPassType::TaaReactiveMask",
            "assert_eq!(manifest.variants.len(), 24)",
        ],
    );

    for (path, source) in [
        (
            "zircon_runtime/src/core/framework/render/shader/variant_key.rs",
            variant_key.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/shader/template/pass_specialization.rs",
            pass_specialization.as_str(),
        ),
        (
            "zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs",
            registry.as_str(),
        ),
        (
            "zircon_runtime/src/dynamic_api/shader_prewarm.rs",
            dynamic_api.as_str(),
        ),
        (
            "zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs",
            prewarm_manifest.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/taa_reactive_shader_pass_identity.rs",
            include_str!("taa_reactive_shader_pass_identity.rs"),
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
                "TAA reactive shader pass identity",
                STATUS,
                "runtime_15_taa_reactive_shader_pass_identity_is_wired",
            ],
        );
    }
}
