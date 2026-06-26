use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_material_runtime_pbr_projection_tests_are_child_owner() {
    let parent =
        read_runtime_src("graphics/scene/render_product_streamer_tests/material_runtime.rs");
    let pbr_projection = read_runtime_src(
        "graphics/scene/render_product_streamer_tests/material_runtime/pbr_projection.rs",
    );

    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_framework_architecture =
        read_repo("docs/assets-and-rendering/render-framework-architecture.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "material runtime parent keeps remaining streamer coverage and child mount",
        &parent,
        &[
            "mod pbr_projection;",
            "fn render_product_streamer_projects_blinn_phong_shading_model_into_pipeline_key(",
            "fn render_product_pbr_streamer_projects_material_sort_offsets_without_pipeline_variant(",
            "fn render_product_pbr_streamer_projects_receive_shadows_override(",
            "fn render_product_pbr_streamer_keeps_authored_texture_key_bits_when_upload_falls_back(",
            "fn render_product_streamer_prepares_shader_texture_slot_runtime_mapping(",
            "fn render_product_streamer_prepares_shader_property_runtime_values(",
        ],
    );

    for moved_anchor in [
        "fn render_product_pbr_streamer_projects_standard_material_into_runtime_key(",
        "fn transform(scale: [f32; 2], offset: [f32; 2]) -> RenderMaterialTextureTransform",
        "RenderMaterialTextureTransform",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "material_runtime.rs should delegate `{moved_anchor}` to material_runtime/pbr_projection.rs"
        );
        assert!(
            pbr_projection.contains(moved_anchor),
            "material_runtime/pbr_projection.rs should own `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "pbr projection child keeps all standard material texture-slot transform assertions",
        &pbr_projection,
        &[
            "pbr_material_with_all_texture_slots()",
            "capture.base_color_texture_transform",
            "material.alpha_cutoff",
            "material.pipeline_key.receive_shadows",
            "capture.normal_texture_transform",
            "capture.metallic_roughness_texture_transform",
            "capture.occlusion_texture_transform",
            "capture.emissive_texture_transform",
            "material.pipeline_key.has_base_color_texture",
            "material_standard_texture_slot_summary",
        ],
    );

    for (path, source) in [
        (
            "graphics/scene/render_product_streamer_tests/material_runtime.rs",
            parent.as_str(),
        ),
        (
            "graphics/scene/render_product_streamer_tests/material_runtime/pbr_projection.rs",
            pbr_projection.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render product streamer test budget after the PBR projection split; got {line_count}"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        (
            "render framework architecture",
            render_framework_architecture.as_str(),
        ),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Material runtime PBR projection tests owner split",
                "render_plan08_material_runtime_pbr_projection_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "PipelineKey receive-shadows shader feature foundation",
                "render_plan08_pipeline_key_receive_shadows_shader_feature_static_passed_cargo_deferred_implementation_cadence",
                "material.pipeline_key.receive_shadows",
                "ShaderFeatureBits::RECEIVE_SHADOWS",
                "graphics/scene/render_product_streamer_tests/material_runtime.rs",
                "graphics/scene/render_product_streamer_tests/material_runtime/pbr_projection.rs",
                "runtime_15_material_runtime_pbr_projection_tests_are_child_owner",
            ],
        );
    }
}
