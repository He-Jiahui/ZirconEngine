use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_framework_bridge_tests_are_child_owners() {
    let root = read_runtime_src("graphics/tests/render_framework_bridge.rs");
    let stats = read_runtime_src("graphics/tests/render_framework_bridge/stats.rs");
    let history = read_runtime_src("graphics/tests/render_framework_bridge/history.rs");
    let pipeline_profiles =
        read_runtime_src("graphics/tests/render_framework_bridge/pipeline_profiles.rs");
    let neural_compute =
        read_runtime_src("graphics/tests/render_framework_bridge/neural_compute.rs");
    let advanced_providers =
        read_runtime_src("graphics/tests/render_framework_bridge/advanced_providers.rs");

    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let render_submit = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let render_architecture =
        read_repo("docs/assets-and-rendering/render-framework-architecture.md");
    let advanced_doc = read_repo("docs/zircon_runtime/core/framework/render/advanced.md");
    let post_process_doc = read_repo("docs/zircon_runtime/core/framework/render/post_process.md");
    let diagnostics_doc = read_repo("docs/zircon_runtime/core/diagnostics.md");

    assert_contains_all(
        "render framework bridge root keeps shared fixtures and child mounts",
        &root,
        &[
            "mod advanced_providers;",
            "mod history;",
            "mod neural_compute;",
            "mod pipeline_profiles;",
            "mod stats;",
            "fn test_extract(",
            "fn test_ui_extract(",
            "fn missing_capabilities(",
            "fn capability_test_summary(",
            "fn neural_compute_render_feature_descriptor(",
            "fn render_hybrid_gi_history_capture(",
            "fn flagship_extract(",
            "fn virtual_geometry_cluster(",
            "fn hybrid_gi_probe(",
            "fn average_region_channel(",
        ],
    );

    for (moved_anchor, owner_name, owner_source) in [
        (
            "fn render_framework_tracks_viewports_and_accepts_frame_extract_submission(",
            "stats.rs",
            stats.as_str(),
        ),
        (
            "fn render_framework_stats_report_effect_stack_product_node_when_authored(",
            "stats.rs",
            stats.as_str(),
        ),
        (
            "fn render_framework_records_temporal_history_after_compatible_history_exists(",
            "history.rs",
            history.as_str(),
        ),
        (
            "fn render_framework_invalidates_history_when_dynamic_render_size_changes(",
            "history.rs",
            history.as_str(),
        ),
        (
            "fn headless_wgpu_server_falls_back_async_compute_passes_to_graphics(",
            "pipeline_profiles.rs",
            pipeline_profiles.as_str(),
        ),
        (
            "fn render_framework_registers_pipeline_assets_and_validates_reload(",
            "pipeline_profiles.rs",
            pipeline_profiles.as_str(),
        ),
        (
            "fn render_framework_rejects_active_pipeline_reload_when_asset_requires_missing_backend_caps(",
            "pipeline_profiles.rs",
            pipeline_profiles.as_str(),
        ),
        (
            "fn render_framework_rejects_neural_compute_plugin_descriptor_without_executor_registration(",
            "neural_compute.rs",
            neural_compute.as_str(),
        ),
        (
            "fn render_framework_rejects_neural_compute_plugin_pipeline_when_backend_capability_is_missing(",
            "neural_compute.rs",
            neural_compute.as_str(),
        ),
        (
            "fn headless_wgpu_server_exposes_current_m5_flagship_baselines_without_rt_capabilities(",
            "advanced_providers.rs",
            advanced_providers.as_str(),
        ),
        (
            "fn render_framework_hybrid_gi_second_frame_resolve_ignores_plugin_private_history(",
            "advanced_providers.rs",
            advanced_providers.as_str(),
        ),
    ] {
        assert!(
            !root.contains(moved_anchor),
            "render_framework_bridge.rs should delegate `{moved_anchor}` to {owner_name}"
        );
        assert!(
            owner_source.contains(moved_anchor),
            "{owner_name} should contain `{moved_anchor}`"
        );
    }

    for (label, source) in [
        ("stats child", stats.as_str()),
        ("history child", history.as_str()),
        ("pipeline profile child", pipeline_profiles.as_str()),
        ("neural compute child", neural_compute.as_str()),
        ("advanced provider child", advanced_providers.as_str()),
    ] {
        assert!(
            source.contains("use super::*;"),
            "{label} should import shared bridge fixtures from the root owner"
        );
    }

    for (path, source) in [
        ("graphics/tests/render_framework_bridge.rs", root.as_str()),
        (
            "graphics/tests/render_framework_bridge/stats.rs",
            stats.as_str(),
        ),
        (
            "graphics/tests/render_framework_bridge/history.rs",
            history.as_str(),
        ),
        (
            "graphics/tests/render_framework_bridge/pipeline_profiles.rs",
            pipeline_profiles.as_str(),
        ),
        (
            "graphics/tests/render_framework_bridge/neural_compute.rs",
            neural_compute.as_str(),
        ),
        (
            "graphics/tests/render_framework_bridge/advanced_providers.rs",
            advanced_providers.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render-framework bridge test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("render index", render_index.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("review findings", review_findings.as_str()),
        ("render submit docs", render_submit.as_str()),
        ("render architecture docs", render_architecture.as_str()),
        ("advanced render docs", advanced_doc.as_str()),
        ("post-process docs", post_process_doc.as_str()),
        ("diagnostics docs", diagnostics_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render framework bridge tests owner split",
                "render_framework_bridge_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/render_framework_bridge.rs",
                "graphics/tests/render_framework_bridge/stats.rs",
                "graphics/tests/render_framework_bridge/history.rs",
                "graphics/tests/render_framework_bridge/pipeline_profiles.rs",
                "graphics/tests/render_framework_bridge/neural_compute.rs",
                "graphics/tests/render_framework_bridge/advanced_providers.rs",
                "runtime_15_render_framework_bridge_tests_are_child_owners",
            ],
        );
    }
}
