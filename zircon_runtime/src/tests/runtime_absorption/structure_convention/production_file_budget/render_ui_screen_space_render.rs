use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_screen_space_ui_render_tests_are_child_owner_split() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/ui/render.rs");
    let tests = read_runtime_src("graphics/scene/scene_renderer/ui/render/tests.rs");

    let plan_14 = read_repo("docs/plans/zircon_runtime/render/14/2026-07-09-2d-stack-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_submit = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "screen-space UI render parent keeps production owner and test mount",
        &parent,
        &[
            "pub(crate) fn record(",
            "fn prepare_screen_space_ui(",
            "fn plan_screen_space_ui_batches(",
            "#[cfg(test)]\nmod tests;",
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

    for (path, source) in [
        ("scene_renderer/ui/render.rs", parent.as_str()),
        ("scene_renderer/ui/render/tests.rs", tests.as_str()),
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
