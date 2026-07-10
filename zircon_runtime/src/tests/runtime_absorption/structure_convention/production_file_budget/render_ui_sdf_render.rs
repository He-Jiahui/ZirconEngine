use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_sdf_render_tests_are_child_owner_split() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render.rs");
    let vertices_owner =
        read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render/vertices.rs");
    let tests_mod = read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs");
    let draw_plan =
        read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render/tests/draw_plan.rs");
    let shader_contract =
        read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render/tests/shader_contract.rs");
    let layout_placement =
        read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render/tests/layout_placement.rs");
    let prepare_report =
        read_runtime_src("graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs");
    let tests = [
        tests_mod.as_str(),
        draw_plan.as_str(),
        shader_contract.as_str(),
        layout_placement.as_str(),
        prepare_report.as_str(),
    ]
    .join("\n");

    let plan_14 = read_repo("docs/plans/zircon_runtime/render/14/2026-07-09-2d-stack-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_submit = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let ui_text = read_repo("docs/zircon_runtime/ui/text.md");

    assert_contains_all(
        "screen-space UI SDF render parent keeps production owner and test mount",
        &parent,
        &[
            "pub(super) struct ScreenSpaceUiSdfRenderer",
            "pub(super) fn prepare(",
            "pub(super) fn render<'pass>(",
            "mod vertices;",
            "use self::vertices::{build_sdf_vertices, ScreenSpaceUiSdfVertex};",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    assert_contains_all(
        "screen-space UI SDF render vertices child owns draw-plan geometry helpers",
        &vertices_owner,
        &[
            "pub(super) struct ScreenSpaceUiSdfVertex",
            "pub(super) fn build_sdf_vertices(",
            "fn push_horizontal_sdf_text_vertices(",
            "fn push_vertical_sdf_text_vertices(",
            "fn push_clipped_glyph_quad(",
            "pub(super) fn resolve_sdf_glyph_advances(",
        ],
    );

    for moved_test in [
        "fn sdf_draw_plan_creates_one_textured_quad_per_glyph(",
        "fn sdf_draw_plan_skips_whitespace_quads_but_preserves_advance(",
        "fn sdf_draw_plan_clips_to_text_frame_without_explicit_clip(",
        "fn sdf_draw_plan_clips_glyph_vertices_and_uvs(",
        "fn sdf_draw_plan_applies_text_alignment_inside_frame(",
        "fn sdf_prepare_report_summarizes_atlas_bake_and_vertices(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "screen-space UI SDF render parent should not own moved test `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "screen-space UI SDF render test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "screen-space UI SDF render test owner keeps private helper coverage",
        &tests,
        &[
            "use super::*;",
            "SdfAtlasUploadMode::FullTexture",
            "fn bake_atlas(",
            "fn text_advance(",
            "fn text_batch(",
        ],
    );

    for (path, source) in [
        ("scene_renderer/ui/sdf_render.rs", parent.as_str()),
        (
            "scene_renderer/ui/sdf_render/vertices.rs",
            vertices_owner.as_str(),
        ),
        (
            "scene_renderer/ui/sdf_render/tests/mod.rs",
            tests_mod.as_str(),
        ),
        (
            "scene_renderer/ui/sdf_render/tests/draw_plan.rs",
            draw_plan.as_str(),
        ),
        (
            "scene_renderer/ui/sdf_render/tests/shader_contract.rs",
            shader_contract.as_str(),
        ),
        (
            "scene_renderer/ui/sdf_render/tests/layout_placement.rs",
            layout_placement.as_str(),
        ),
        (
            "scene_renderer/ui/sdf_render/tests/prepare_report.rs",
            prepare_report.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the SDF test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 14", &plan_14),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("render product submit docs", &render_product_submit),
        ("UI text docs", &ui_text),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "Screen-space UI SDF render test owner split",
                "render_plan14_sdf_render_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "Runtime 15 M4 SDF atlas/render tests folder-backed guard sync",
                "runtime_15_sdf_atlas_render_tests_folder_backed_guard_sync_static_passed_cargo_deferred",
                "graphics/scene/scene_renderer/ui/sdf_render.rs",
                "graphics/scene/scene_renderer/ui/sdf_render/vertices.rs",
                "graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs",
                "graphics/scene/scene_renderer/ui/sdf_render/tests/draw_plan.rs",
                "graphics/scene/scene_renderer/ui/sdf_render/tests/layout_placement.rs",
                "graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs",
                "runtime_15_screen_space_ui_sdf_render_tests_are_child_owner_split",
            ],
        );
    }
}
