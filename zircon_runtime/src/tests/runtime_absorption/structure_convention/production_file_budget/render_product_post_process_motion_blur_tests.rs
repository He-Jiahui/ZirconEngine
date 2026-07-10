use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_product_post_process_motion_blur_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/render_product_post_process.rs");
    let motion_blur = read_runtime_src("graphics/tests/render_product_post_process/motion_blur.rs");

    let plan_07 = read_repo("docs/plans/zircon_runtime/render/07/2026-07-09-postprocess-color-pipeline-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let post_process_doc = read_repo("docs/zircon_runtime/core/framework/render/post_process.md");
    let post_process_scene_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/post_process/index.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "post-process product parent keeps compact product tests, shared fixtures, and child mount",
        &parent,
        &[
            "mod motion_blur;",
            "fn render_product_post_uber_light_effects_change_final_frame(",
            "fn render_product_post_non_neutral_tonemap_grading_changes_final_frame(",
            "fn render_product_post_user_lut_texture_changes_final_frame_and_matches_readback_reference(",
            "fn submit_and_capture_post_process_product(",
            "fn assert_graph_executor_executed(",
            "fn frame_rgb_abs_delta(",
        ],
    );

    for moved_anchor in [
        "fn render_product_post_motion_blur_split_uses_velocity_and_changes_final_frame(",
        "fn motion_blur_product_framework(",
        "fn particle_color_product_framework(",
        "fn particle_transparent_billboard_executor(",
        "fn assert_scene_velocity_readback_nonzero(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "render_product_post_process.rs should delegate `{moved_anchor}` to motion_blur.rs"
        );
        assert!(
            motion_blur.contains(moved_anchor),
            "motion blur child owner should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "post-process motion-blur child keeps the velocity product scene and local executor fixtures",
        &motion_blur,
        &[
            "use super::{",
            "POST_MOTION_BLUR_EXECUTOR_ID",
            "TEMPORAL_VELOCITY_OBJECT_EXECUTOR_ID",
            "particle_render_feature_descriptor_with_velocity",
            "RenderMotionBlurSettings",
            "RenderParticlePreviousSpriteSnapshot",
            "assert_graph_executor_order",
            "assert_scene_velocity_readback_nonzero",
        ],
    );

    for (path, source) in [
        (
            "graphics/tests/render_product_post_process.rs",
            parent.as_str(),
        ),
        (
            "graphics/tests/render_product_post_process/motion_blur.rs",
            motion_blur.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render product test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 07", plan_07.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("post-process docs", post_process_doc.as_str()),
        (
            "scene renderer post-process docs",
            post_process_scene_doc.as_str(),
        ),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render product post-process motion-blur test owner split",
                "render_plan07_product_post_process_motion_blur_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/tests/render_product_post_process.rs",
                "graphics/tests/render_product_post_process/motion_blur.rs",
                "runtime_15_render_product_post_process_motion_blur_tests_are_child_owner",
            ],
        );
    }
}
