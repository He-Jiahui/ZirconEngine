use super::*;

#[test]
fn ui_hotspots_group_counters_by_scenario() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.idle_hover.redraw_region", 2.0));
    snapshot
        .counters
        .push(counter("ui.idle_hover.chrome_command_patch_count", 3.0));
    snapshot
        .counters
        .push(counter("ui.idle_hover.painted_pixels", 120.0));
    for (name, value) in [
        ("ui.idle_hover.presented_surface_pixels", 480.0),
        ("ui.idle_hover.host_invalidation_transaction_count", 4.0),
        ("ui.idle_hover.host_invalidation_scope_count", 5.0),
        (
            "ui.idle_hover.host_invalidation_legacy_dirty_transaction_count",
            1.0,
        ),
        ("ui.idle_hover.host_invalidation_full_target_count", 1.0),
        (
            "ui.idle_hover.host_invalidation_shell_content_target_count",
            1.0,
        ),
        (
            "ui.idle_hover.host_invalidation_workbench_projection_target_count",
            1.0,
        ),
        (
            "ui.idle_hover.host_invalidation_view_presentation_target_count",
            1.0,
        ),
        (
            "ui.idle_hover.host_invalidation_window_metrics_target_count",
            0.0,
        ),
        (
            "ui.idle_hover.host_invalidation_paint_only_target_count",
            0.0,
        ),
    ] {
        snapshot.counters.push(counter(name, value));
    }
    snapshot.counters.push(counter(
        "ui.idle_hover.asset_editor_pane_presentation_build_count",
        4.0,
    ));
    snapshot.counters.push(counter(
        "ui.idle_hover.asset_editor_pane_reflection_build_count",
        5.0,
    ));
    snapshot.counters.push(counter(
        "ui.idle_hover.asset_editor_pane_preview_build_count",
        6.0,
    ));
    snapshot.counters.push(counter(
        "ui.idle_hover.asset_editor_pane_source_build_count",
        7.0,
    ));
    snapshot.counters.push(counter(
        "ui.idle_hover.asset_editor_pane_inspector_build_count",
        8.0,
    ));
    snapshot.counters.push(counter(
        "ui.idle_hover.asset_editor_pane_style_build_count",
        9.0,
    ));
    snapshot.counters.push(counter(
        "ui.idle_hover.asset_editor_pane_theme_build_count",
        10.0,
    ));
    snapshot.counters.push(counter(
        "ui.idle_hover.asset_editor_pane_command_availability_build_count",
        11.0,
    ));
    for (name, value) in [
        ("ui.idle_hover.gpu_timestamp_supported_present_count", 3.0),
        ("ui.idle_hover.gpu_time_us", 100.0),
        ("ui.idle_hover.gpu_time_us", 200.0),
        ("ui.idle_hover.gpu_time_us", 400.0),
        ("ui.idle_hover.gpu_profile_latency_frames", 2.0),
    ] {
        snapshot.counters.push(counter(name, value));
    }
    snapshot
        .counters
        .push(counter("ui.drawer_resize.slow_path_rebuild_count", 1.0));
    snapshot
        .counters
        .push(counter("runtime.unrelated.counter", 1.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    assert_eq!(report.generated_from_counter_count, 27);
    let idle = report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario == "idle_hover")
        .expect("idle hover scenario");
    assert_eq!(idle.redraw_region_count, 2);
    assert_eq!(idle.chrome_command_patch_count, 3);
    assert_eq!(idle.painted_pixels, 120);
    assert_eq!(idle.presented_surface_pixels, 480);
    assert_eq!(idle.host_invalidation_transaction_count, 4);
    assert_eq!(idle.host_invalidation_scope_count, 5);
    assert_eq!(idle.host_invalidation_legacy_dirty_transaction_count, 1);
    assert_eq!(idle.host_invalidation_full_target_count, 1);
    assert_eq!(idle.host_invalidation_shell_content_target_count, 1);
    assert_eq!(idle.host_invalidation_workbench_projection_target_count, 1);
    assert_eq!(idle.host_invalidation_view_presentation_target_count, 1);
    assert_eq!(idle.host_invalidation_window_metrics_target_count, 0);
    assert_eq!(idle.host_invalidation_paint_only_target_count, 0);
    assert_eq!(idle.asset_editor_pane_presentation_build_count, 4);
    assert_eq!(idle.asset_editor_pane_reflection_build_count, 5);
    assert_eq!(idle.asset_editor_pane_preview_build_count, 6);
    assert_eq!(idle.asset_editor_pane_source_build_count, 7);
    assert_eq!(idle.asset_editor_pane_inspector_build_count, 8);
    assert_eq!(idle.asset_editor_pane_style_build_count, 9);
    assert_eq!(idle.asset_editor_pane_theme_build_count, 10);
    assert_eq!(idle.asset_editor_pane_command_availability_build_count, 11);
    assert_eq!(idle.gpu_timestamp_supported_present_count, 3);
    assert_eq!(idle.gpu_time_sample_count, 3);
    assert_eq!(idle.gpu_time_p50_us, 200);
    assert_eq!(idle.gpu_time_p95_us, 400);
    assert_eq!(idle.gpu_time_max_us, 400);
    assert_eq!(idle.gpu_profile_latency_max_frames, 2);
    assert!(report
        .alerts
        .iter()
        .any(|alert| alert.rule == "resize_triggered_slow_path_rebuild"));
    assert!(report.alerts.iter().any(|alert| {
        alert.rule == "non_structural_interaction_rebuilt_asset_editor_pane_presentation"
    }));
}
