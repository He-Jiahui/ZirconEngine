use super::*;

#[test]
fn live_interaction_compiled_plan_cache_hit_is_flagged() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.click.gpu_batch_plan_cache_hits", 1.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    assert!(report
        .alerts
        .iter()
        .any(|alert| alert.rule == "live_patch_reused_compiled_full_projection"));
}

#[test]
fn hover_presentation_rebuild_is_flagged() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.idle_hover.presentation_rebuild_count", 1.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    assert!(report
        .alerts
        .iter()
        .any(|alert| alert.rule == "non_structural_interaction_rebuilt_presentation"));
}

#[test]
fn hover_snapshot_or_model_rebuild_is_flagged() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.idle_hover.chrome_snapshot_count", 1.0));
    snapshot
        .counters
        .push(counter("ui.idle_hover.workbench_model_build_count", 1.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    let idle = report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario == "idle_hover")
        .expect("idle hover scenario");
    assert_eq!(idle.chrome_snapshot_count, 1);
    assert_eq!(idle.workbench_model_build_count, 1);
    assert!(report
        .alerts
        .iter()
        .any(|alert| alert.rule == "hover_rebuilt_chrome_snapshot_or_model"));
}

#[test]
fn region_request_that_repaints_full_frame_is_flagged() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.idle_hover.redraw_region", 1.0));
    snapshot
        .counters
        .push(counter("ui.idle_hover.full_paint_count", 1.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    assert!(report
        .alerts
        .iter()
        .any(|alert| alert.rule == "region_request_repainted_full_frame"));
}
