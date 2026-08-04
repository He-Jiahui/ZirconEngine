use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_project_render_quality_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/project_render.rs");
    let project_scenes = read_runtime_src("graphics/tests/project_render/project_scenes.rs");
    let quality = read_runtime_src("graphics/tests/project_render/render_quality.rs");

    let plan_04 = read_repo(
        "docs/plans/zircon_runtime/render/04/2026-07-09-visibility-culling-output-records.md",
    );
    let plan_05 = read_repo(
        "docs/plans/zircon_runtime/render/05/2026-07-09-lighting-shadows-output-records.md",
    );
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "project-render parent keeps shared fixtures and child mounts",
        &parent,
        &[
            "mod project_scenes;",
            "mod render_quality;",
            "fn unique_temp_project_root(",
            "fn project_asset_manager_with_first_wave_plugin_importers(",
            "fn submit_snapshot(",
            "fn average_channel(",
        ],
    );

    for scene_products_anchor in [
        "fn directory_project_scene_renders_non_background_frame_with_gizmo_overlay(",
        "fn example_vampire_scene_renders_visible_mesh_pixels(",
        "fn directory_project_material_shader_drives_pipeline_color_output(",
        "fn wire_only_mode_reduces_filled_surface_pixels(",
    ] {
        assert!(
            !parent.contains(scene_products_anchor),
            "project_render.rs should delegate `{scene_products_anchor}` to project_scenes.rs"
        );
        assert!(
            project_scenes.contains(scene_products_anchor),
            "project_scenes.rs should own `{scene_products_anchor}`"
        );
    }

    for moved_anchor in [
        "fn temporal_history_rotates_history_when_scene_material_changes(",
        "fn ssao_quality_profile_darkens_scene_when_enabled(",
        "fn clustered_lighting_quality_profile_schedules_cluster_pass_without_tile_tint(",
        "fn deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path(",
        "fn assert_ssao_shared_hzb_product_path(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "project_render.rs should delegate `{moved_anchor}` to render_quality.rs"
        );
        assert!(
            quality.contains(moved_anchor),
            "project_render render-quality child should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "project-render render-quality child keeps WGPU quality/deferred product coverage",
        &quality,
        &[
            "use super::{",
            "RenderQualityProfile",
            "RenderPipelineHandle",
            "PostProcessGraphResourceNames::HZB_FURTHEST",
            "default_rendering_feature_descriptors",
            "lighting.light-grid",
            "deferred.depth-prepass",
        ],
    );

    for (path, source) in [
        ("graphics/tests/project_render.rs", parent.as_str()),
        (
            "graphics/tests/project_render/render_quality.rs",
            quality.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 project render test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 04", plan_04.as_str()),
        ("Plan 05", plan_05.as_str()),
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project render quality/deferred tests owner split",
                "render_project_render_quality_deferred_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/project_render.rs",
                "graphics/tests/project_render/render_quality.rs",
                "runtime_15_project_render_quality_tests_are_child_owner",
            ],
        );
    }
}
