use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_render_tests_are_child_owner_split() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/render.rs");
    let geometry = read_runtime_src("graphics/scene/scene_renderer/ui/render/geometry.rs");
    let record = read_runtime_src("graphics/scene/scene_renderer/ui/render/record.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/ui/render/tests.rs");
    let clipping = read_runtime_src("graphics/scene/scene_renderer/ui/render/tests/clipping.rs");
    let glyph_artifacts =
        read_runtime_src("graphics/scene/scene_renderer/ui/render/tests/glyph_artifacts.rs");

    let plan_14 =
        read_repo("docs/plans/zircon_runtime/render/14/2026-07-09-2d-stack-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_submit = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "screen-space UI render parent keeps planning owner and child mounts",
        &parent,
        &[
            "mod record;",
            "fn prepare_screen_space_ui(",
            "fn plan_screen_space_ui_batches(",
            "#[cfg(all(test, feature = \"ui\"))]\nmod tests;",
        ],
    );
    assert!(
        !parent.contains("pub(crate) fn record("),
        "screen-space UI render parent should not retain the moved GPU submission owner"
    );
    assert_contains_all(
        "screen-space UI geometry owns shared clipping conversion",
        &geometry,
        &[
            "pub(in crate::graphics::scene::scene_renderer::ui) fn clipped_scissor(",
            "let visible_frame = viewport.intersection(frame)?;",
            "Some(clip) => visible_frame.intersection(clip).and_then(frame_to_scissor),",
        ],
    );
    assert_contains_all(
        "screen-space UI record child owns GPU submission",
        &record,
        &[
            "impl ScreenSpaceUiRenderer",
            "pub(crate) fn record(",
            "self.image_system.clear_frame_state();",
            "fn record_empty_screen_space_ui_pass(",
            "color_attachment_operations(attachment_ops, clear_color)",
        ],
    );

    for moved_test in [
        "fn screen_space_ui_plan_keeps_text_batches_for_quad_commands(",
        "fn screen_space_ui_plan_routes_sdf_text_to_a_separate_batch(",
        "fn screen_space_ui_plan_keeps_auto_text_in_a_separate_batch(",
        "fn screen_space_ui_plan_uses_resolved_text_layout_lines_as_batches(",
        "fn screen_space_ui_plan_splits_rich_text_runs_from_shared_paint(",
        "fn screen_space_ui_plan_uses_shared_text_decorations_as_pre_and_post_text_draws(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "screen-space UI render parent should not own moved test `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "screen-space UI render test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "screen-space UI render test owner keeps private helper coverage",
        &tests,
        &[
            "use super::*;",
            "UiResolvedTextLayout",
            "UiEditableTextState",
            "screen_space_ui_plan_splits_rich_text_runs_from_shared_paint",
            "screen_space_ui_plan_uses_shared_text_decorations_as_pre_and_post_text_draws",
        ],
    );
    assert_contains_all(
        "screen-space UI test owner mounts glyph artifact routing tests",
        &tests,
        &["mod clipping;", "mod glyph_artifacts;"],
    );
    assert_contains_all(
        "screen-space UI clip test owner rejects invisible commands",
        &clipping,
        &[
            "fn screen_space_ui_plan_skips_a_command_with_a_clip_outside_the_viewport()",
            "fn screen_space_ui_plan_skips_a_command_when_its_clip_misses_the_command_frame()",
            "fn screen_space_ui_plan_ignores_a_fully_clipped_quad_for_later_text_backgrounds()",
            "assert!(plan.vertices.is_empty());",
            "assert!(plan.native_texts.is_empty());",
        ],
    );
    assert_contains_all(
        "screen-space UI glyph artifact test owner keeps its direct dependencies",
        &glyph_artifacts,
        &[
            "use std::sync::Arc;",
            "register_resolved_text_glyph_artifact",
            "screen_space_ui_plan_renders_source_isomorphic_plain_layout_without_glyph_artifact",
        ],
    );
    assert!(
        !glyph_artifacts.contains(concat!("refresh_screen_space_", "text_batch_glyphs")),
        "glyph artifact tests must not restore renderer-owned artifact reshaping"
    );
    assert!(
        !tests.contains("use std::sync::Arc;"),
        "screen-space UI parent test owner must not retain glyph-artifact-only imports"
    );
    for moved_test in [
        "fn screen_space_ui_plan_renders_source_isomorphic_plain_layout_without_glyph_artifact(",
        "fn screen_space_ui_plan_does_not_shape_visual_bidi_runs_without_an_artifact(",
        "fn screen_space_ui_plan_preserves_plain_glyph_artifact_through_native_routing(",
    ] {
        assert!(
            !tests.contains(moved_test),
            "screen-space UI parent test owner should not retain glyph artifact test `{moved_test}`"
        );
        assert!(
            glyph_artifacts.contains(moved_test),
            "glyph artifact test owner should contain moved test `{moved_test}`"
        );
    }

    for (path, source) in [
        ("scene_renderer/ui/render.rs", parent.as_str()),
        ("scene_renderer/ui/render/geometry.rs", geometry.as_str()),
        ("scene_renderer/ui/render/record.rs", record.as_str()),
        ("scene_renderer/ui/render/tests.rs", tests.as_str()),
        (
            "scene_renderer/ui/render/tests/clipping.rs",
            clipping.as_str(),
        ),
        (
            "scene_renderer/ui/render/tests/glyph_artifacts.rs",
            glyph_artifacts.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 14", &plan_14),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("render product submit docs", &render_product_submit),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "Screen-space UI render test owner split",
                "render_plan14_screen_space_ui_render_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/ui/render.rs",
                "graphics/scene/scene_renderer/ui/render/tests.rs",
                "runtime_15_screen_space_ui_render_tests_are_child_owner_split",
            ],
        );
    }
}
