use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_update_base_stats_post_process_diagnostics_is_child_owner() {
    let parent = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
    );
    let child = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/post_process_diagnostics.rs",
    );

    let plan_09 = read_repo(
        "docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "base stats parent delegates post-process diagnostics to child owner",
        &parent,
        &[
            "pub(super) fn update_base_stats(",
            "mod post_process_diagnostics;",
            "effect_stack_resource_status",
            "particle_velocity_missing_sprite_count",
            "particle_velocity_anonymous_stream_ambiguity_count",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_helper in [
        "fn effect_stack_resource_status(",
        "fn effect_stack_uses_resource(",
        "fn particle_velocity_missing_sprite_count(",
        "fn particle_velocity_anonymous_stream_ambiguity_count(",
        "fn particle_velocity_diagnostics_enabled(",
    ] {
        assert!(
            !parent.contains(moved_helper),
            "base_stats.rs should delegate {moved_helper} to post_process_diagnostics.rs"
        );
    }

    assert_contains_all(
        "base stats post-process diagnostics child owns effect-stack and particle velocity helpers",
        &child,
        &[
            "pub(super) fn effect_stack_resource_status(",
            "fn effect_stack_uses_resource(",
            "pub(super) fn particle_velocity_missing_sprite_count(",
            "pub(super) fn particle_velocity_anonymous_stream_ambiguity_count(",
            "fn particle_velocity_diagnostics_enabled(",
            "PostProcessGraphResourceNames::GBUFFER_NORMAL",
            "executor_id == \"particle.transparent\"",
        ],
    );

    for (path, source) in [
        (
            "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
            parent.as_str(),
        ),
        (
            "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/post_process_diagnostics.rs",
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 submit stats owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render product submit doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plan 09 update-base-stats post-process diagnostics owner split",
                "render_plan09_update_base_stats_post_process_diagnostics_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
                "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/post_process_diagnostics.rs",
                "runtime_15_render_update_base_stats_post_process_diagnostics_is_child_owner",
            ],
        );
    }
}
