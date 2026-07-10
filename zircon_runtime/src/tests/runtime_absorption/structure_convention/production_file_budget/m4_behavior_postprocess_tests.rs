use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_m4_behavior_postprocess_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/m4_behavior_layers.rs");
    let postprocess = read_runtime_src("graphics/tests/m4_behavior_layers/postprocess.rs");
    let particles = read_runtime_src("graphics/tests/m4_behavior_layers/particles.rs");
    let queue_override = read_runtime_src("graphics/tests/m4_behavior_layers/queue_override.rs");
    let transparent3d = read_runtime_src("graphics/tests/m4_behavior_layers/transparent3d.rs");

    let plan_09 = read_repo("docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "m4 behavior parent keeps fixture/offline-bake coverage and child mounts",
        &parent,
        &[
            "mod particles;",
            "mod postprocess;",
            "mod queue_override;",
            "mod transparent3d;",
            "fn offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering(",
            "struct RenderFixture",
            "fn frame_extract<",
            "fn render_extract(",
            "fn write_flat_color_wgsl(",
        ],
    );

    for moved_anchor in [
        "fn bloom_quality_profile_spreads_bright_pixels_when_enabled(",
        "fn color_grading_extract_tints_scene_after_post_process(",
        "RenderBloomSettings",
        "RenderColorGradingSettings",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "m4_behavior_layers.rs should delegate `{moved_anchor}` to postprocess.rs"
        );
        assert!(
            postprocess.contains(moved_anchor),
            "m4_behavior_layers/postprocess.rs should own `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "m4 postprocess child keeps product-frame comparison helpers and quality toggles",
        &postprocess,
        &[
            "use super::{",
            "ring_luma",
            "average_channel",
            "with_bloom(false)",
            "with_color_grading(false)",
            "RenderQualityProfile",
        ],
    );

    for (path, source) in [
        ("graphics/tests/m4_behavior_layers.rs", parent.as_str()),
        (
            "graphics/tests/m4_behavior_layers/postprocess.rs",
            postprocess.as_str(),
        ),
        (
            "graphics/tests/m4_behavior_layers/particles.rs",
            particles.as_str(),
        ),
        (
            "graphics/tests/m4_behavior_layers/queue_override.rs",
            queue_override.as_str(),
        ),
        (
            "graphics/tests/m4_behavior_layers/transparent3d.rs",
            transparent3d.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render behavior test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "M4 behavior postprocess tests owner split",
                "render_plan09_m4_behavior_postprocess_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/m4_behavior_layers.rs",
                "graphics/tests/m4_behavior_layers/postprocess.rs",
                "runtime_15_m4_behavior_postprocess_tests_are_child_owner",
            ],
        );
    }
}
