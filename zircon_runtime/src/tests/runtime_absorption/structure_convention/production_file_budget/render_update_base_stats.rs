use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_update_base_stats_tests_are_child_owner() {
    let parent = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
    );
    let tests = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/tests.rs",
    );
    let post_process_diagnostics = read_runtime_src(
        "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/post_process_diagnostics.rs",
    );
    let plan_09 = read_repo("docs/plans/zircon_runtime/render/09-camera-render-ordering.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "base stats parent keeps submit stats production responsibilities and mounts tests",
        &parent,
        &[
            "pub(super) fn update_base_stats(",
            "fn update_visibility_stats(",
            "fn update_visibility_static_index_stats(",
            "fn update_hzb_occlusion_stats(",
            "fn graph_execution_coverage_report(",
            "mod post_process_diagnostics;",
            "effect_stack_resource_status",
            "particle_velocity_missing_sprite_count",
            "#[cfg(test)]\nmod tests;",
        ],
    );
    for moved_test in [
        "fn effect_stack_graph(",
        "fn graph_execution_coverage_report_counts_missing_unexpected_and_duplicate_passes",
        "fn update_visibility_stats_sums_per_view_culling_counts",
        "fn update_visibility_static_index_stats_records_latest_report",
        "fn update_hzb_occlusion_stats_records_latest_cull_report",
        "fn update_hzb_occlusion_stats_records_readback_and_overrides_visibility_occlusion_count",
        "fn update_hzb_occlusion_stats_resets_when_no_report",
        "fn effect_stack_resource_status_detects_graph_bound_ssr_normal",
        "fn effect_stack_resource_status_detects_executed_motion_vector_prepass_chain",
        "fn particle_velocity_gap_counts_sprites_only_when_reconstructed_velocity_is_requested",
        "fn particle_velocity_anonymous_stream_ambiguity_requires_velocity_diagnostics",
        "fn particle_velocity_diagnostics_enabled(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "base_stats.rs should mount the test/helper child instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "base stats test child owns submit stats helper coverage",
        &tests,
        &[
            "use super::{",
            "fn effect_stack_graph(",
            "fn graph_execution_coverage_report_counts_missing_unexpected_and_duplicate_passes",
            "fn update_visibility_stats_sums_per_view_culling_counts",
            "fn update_visibility_static_index_stats_records_latest_report",
            "fn update_hzb_occlusion_stats_records_latest_cull_report",
            "fn update_hzb_occlusion_stats_records_readback_and_overrides_visibility_occlusion_count",
            "fn update_hzb_occlusion_stats_resets_when_no_report",
            "fn effect_stack_resource_status_detects_graph_bound_ssr_normal",
            "fn effect_stack_resource_status_detects_executed_motion_vector_prepass_chain",
            "fn particle_velocity_gap_counts_sprites_only_when_reconstructed_velocity_is_requested",
            "fn particle_velocity_anonymous_stream_ambiguity_requires_velocity_diagnostics",
        ],
    );
    assert_contains_all(
        "base stats post-process diagnostics child owns effect-stack and particle helper logic",
        &post_process_diagnostics,
        &[
            "pub(super) fn effect_stack_resource_status(",
            "fn effect_stack_uses_resource(",
            "pub(super) fn particle_velocity_missing_sprite_count(",
            "pub(super) fn particle_velocity_anonymous_stream_ambiguity_count(",
            "fn particle_velocity_diagnostics_enabled(",
        ],
    );

    for (path, source) in [
        (
            "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
            parent.as_str(),
        ),
        (
            "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/tests.rs",
            tests.as_str(),
        ),
        (
            "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/post_process_diagnostics.rs",
            post_process_diagnostics.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production/test soft budget; got {line_count} lines"
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
                "Plan 09 update-base-stats test owner split",
                "render_plan09_update_base_stats_test_owner_split_static_passed_cargo_deferred_active_editor_lane",
                "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs",
                "graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats/tests.rs",
                "runtime_15_render_update_base_stats_tests_are_child_owner",
            ],
        );
    }
}
