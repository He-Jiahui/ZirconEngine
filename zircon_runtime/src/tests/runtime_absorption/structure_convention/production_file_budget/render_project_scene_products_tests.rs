use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_project_render_scene_products_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/project_render.rs");
    let project_scenes = read_runtime_src("graphics/tests/project_render/project_scenes.rs");
    let render_quality = read_runtime_src("graphics/tests/project_render/render_quality.rs");

    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_submit_doc =
        read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "project render parent keeps shared fixtures and child mounts",
        &parent,
        &[
            "mod project_scenes;",
            "mod render_quality;",
            "fn unique_temp_project_root(",
            "fn project_asset_manager_with_first_wave_plugin_importers(",
            "fn write_valid_wgsl(",
            "fn write_flat_green_wgsl(",
            "fn write_flat_color_wgsl(",
            "fn write_checker_png(",
            "fn write_solid_png(",
            "fn write_triangle_obj(",
            "fn write_quad_obj(",
            "fn write_material(",
            "fn write_scene(",
            "fn build_snapshot(",
            "fn submit_snapshot(",
        ],
    );

    for moved_anchor in [
        "fn directory_project_scene_renders_non_background_frame_with_gizmo_overlay(",
        "fn example_vampire_scene_renders_visible_mesh_pixels(",
        "fn export_example_vampire_scene_png(",
        "fn directory_project_material_shader_drives_pipeline_color_output(",
        "fn wire_only_mode_reduces_filled_surface_pixels(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "project_render.rs should delegate `{moved_anchor}` to project_render/project_scenes.rs"
        );
        assert!(
            project_scenes.contains(moved_anchor),
            "project_render/project_scenes.rs should own `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "existing render-quality child still consumes shared parent fixtures",
        &render_quality,
        &[
            "unique_temp_project_root",
            "write_flat_color_wgsl",
            "write_flat_green_wgsl",
            "build_snapshot",
            "submit_snapshot",
            "average_channel",
        ],
    );

    for (path, source) in [
        ("graphics/tests/project_render.rs", parent.as_str()),
        (
            "graphics/tests/project_render/project_scenes.rs",
            project_scenes.as_str(),
        ),
        (
            "graphics/tests/project_render/render_quality.rs",
            render_quality.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 project render test budget after the scene-products split; got {line_count}"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        (
            "render product submit doc",
            render_product_submit_doc.as_str(),
        ),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project render scene product tests owner split",
                "render_project_scene_products_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/project_render.rs",
                "graphics/tests/project_render/project_scenes.rs",
                "runtime_15_project_render_scene_products_tests_are_child_owner",
            ],
        );
    }
}
