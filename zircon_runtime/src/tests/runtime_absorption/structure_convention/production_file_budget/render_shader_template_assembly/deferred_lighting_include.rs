use super::*;

#[test]
fn runtime_15_deferred_lighting_include_source_dispatch_is_owned() {
    let lighting_shader_source = read_runtime_src(
        "graphics/scene/scene_renderer/deferred/lighting_pipeline/shader_source.rs",
    );
    let lighting_tests =
        read_runtime_src("graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs");
    let deferred_lighting_wgsl =
        read_runtime_src("graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl");
    let standard_pbr_include =
        read_runtime_src("graphics/shader/wgsl/zr_shade_deferred_standard_pbr.wgsl");
    let blinn_phong_include =
        read_runtime_src("graphics/shader/wgsl/zr_shade_deferred_blinn_phong.wgsl");
    let unlit_include = read_runtime_src("graphics/shader/wgsl/zr_shade_deferred_unlit.wgsl");
    let dynamic_resolution =
        read_runtime_src("graphics/tests/pipeline_compile/dynamic_resolution.rs");

    assert_contains_all(
        "deferred lighting source assembler owns request-level custom include dispatch",
        &lighting_shader_source,
        &[
            "pub(in crate::graphics::scene::scene_renderer::deferred) struct DeferredLightingShaderSourceRequest",
            "with_shading_model_deferred_include_source",
            "pub(in crate::graphics::scene::scene_renderer::deferred) fn assemble_deferred_lighting_shader_source",
            "UnknownDeferredInclude",
            "shading_model_deferred_include_sources",
            "DEFERRED_STANDARD_PBR_INCLUDE_TOKEN",
            "DEFERRED_BLINN_PHONG_INCLUDE_TOKEN",
            "DEFERRED_UNLIT_INCLUDE_TOKEN",
            "CUSTOM_DISPATCH_MARKER",
            "custom_deferred_dispatch",
            "builtin_deferred_include_token",
            "deferred_shading_function_name",
            "zr_shade_deferred_standard_pbr.wgsl",
            "zr_shade_deferred_blinn_phong.wgsl",
            "zr_shade_deferred_unlit.wgsl",
        ],
    );
    assert_contains_all(
        "deferred lighting tests cover unknown and custom descriptor include source",
        &lighting_tests,
        &[
            "CUSTOM_TOON_DEFERRED_INCLUDE",
            "toon_shading_model_descriptor",
            "deferred_lighting_shader_rejects_unknown_shading_model_deferred_include",
            "deferred_lighting_shader_uses_custom_shading_model_deferred_include_source",
            "with_shading_model_deferred_include_source",
            "UnknownDeferredInclude",
            "zr_shade_deferred_toon.wgsl",
            "shade_deferred_toon(position, coord, albedo, material, normal)",
            "custom deferred lighting shader should validate",
        ],
    );
    assert_contains_all(
        "deferred lighting entry keeps built-in dispatch and custom dispatch marker",
        &deferred_lighting_wgsl,
        &[
            "return apply_deferred_volumetric(",
            "add_deferred_emissive(shade_deferred_unlit(albedo), emissive),",
            "shade_deferred_blinn_phong(position, coord, albedo, material, normal),",
            "zr-deferred-lighting-custom-shading-model-dispatch",
            "shade_deferred_standard_pbr(position, coord, albedo, material, normal),",
        ],
    );
    assert_contains_all(
        "built-in deferred lighting includes preserve standard PBR, BlinnPhong, and Unlit dispatch",
        &standard_pbr_include,
        &[
            "fn shade_deferred_standard_pbr",
            "shade_deferred_lit(position, coord, albedo, material, normal, ZR_SHADING_MODEL_STANDARD_PBR_ID)",
        ],
    );
    assert_contains_all(
        "built-in deferred lighting includes preserve BlinnPhong dispatch",
        &blinn_phong_include,
        &[
            "fn shade_deferred_blinn_phong",
            "shade_deferred_lit(position, coord, albedo, material, normal, ZR_SHADING_MODEL_BLINN_PHONG_ID)",
        ],
    );
    assert_contains_all(
        "built-in deferred lighting includes preserve Unlit dispatch",
        &unlit_include,
        &["fn shade_deferred_unlit", "return albedo;"],
    );
    assert_contains_all(
        "pipeline compile fixture includes deferred lighting built-in source chunks",
        &dynamic_resolution,
        &[
            "zr_shade_deferred_standard_pbr.wgsl",
            "zr_shade_deferred_blinn_phong.wgsl",
            "zr_shade_deferred_unlit.wgsl",
            "deferred_lighting.wgsl",
        ],
    );

    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let template_doc = read_repo("docs/zircon_runtime/graphics/shader/template.md");
    let material_doc = read_repo("docs/zircon_runtime/core/framework/render/material.md");
    let lighting_doc = read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/lighting.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("shader template doc", template_doc.as_str()),
        ("material doc", material_doc.as_str()),
        ("lighting doc", lighting_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Deferred lighting include source dispatch",
                "render_plan08_deferred_lighting_include_source_dispatch_static_passed_cargo_deferred",
                "with_shading_model_deferred_include_source",
                "assemble_deferred_lighting_shader_source",
                "deferred_lighting_shader_uses_custom_shading_model_deferred_include_source",
            ],
        );
    }
}
