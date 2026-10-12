use super::*;

#[test]
fn ui_hotspots_collect_gpu_presenter_counters() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.viewport_image.gpu_upload_bytes", 1024.0));
    snapshot
        .counters
        .push(counter("ui.viewport_image.gpu_draw_calls", 7.0));
    snapshot
        .counters
        .push(counter("ui.viewport_image.gpu_visible_commands", 11.0));
    snapshot
        .counters
        .push(counter("ui.viewport_image.gpu_visible_draw_items", 13.0));
    snapshot
        .counters
        .push(counter("ui.viewport_image.gpu_batch_layers", 2.0));
    snapshot
        .counters
        .push(counter("ui.viewport_image.gpu_batch_dependencies", 3.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    let viewport = report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario == "viewport_image")
        .expect("viewport image scenario");
    assert_eq!(viewport.gpu_upload_bytes, 1024);
    assert_eq!(viewport.gpu_draw_calls, 7);
    assert_eq!(viewport.gpu_visible_commands, 11);
    assert_eq!(viewport.gpu_visible_draw_items, 13);
    assert_eq!(viewport.gpu_batch_layers, 2);
    assert_eq!(viewport.gpu_batch_dependencies, 3);
}

#[test]
fn software_fallback_present_is_flagged_for_gpu_profile() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot.counters.push(counter(
        "ui.idle_hover.software_fallback_present_count",
        1.0,
    ));

    let report = super::analyze_ui_hotspots(&snapshot);

    assert!(report
        .alerts
        .iter()
        .any(|alert| alert.rule == "gpu_presenter_fell_back_to_software"));
}

#[test]
fn gpu_command_stream_without_draw_calls_is_flagged() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.idle_hover.chrome_command_patch_count", 1.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    assert!(report
        .alerts
        .iter()
        .any(|alert| alert.rule == "gpu_presenter_recorded_no_draw_calls"));
}

#[test]
fn viewport_image_command_without_gpu_upload_is_flagged() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.viewport_image.chrome_command_patch_count", 1.0));
    snapshot
        .counters
        .push(counter("ui.viewport_image.gpu_draw_calls", 1.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    assert!(report
        .alerts
        .iter()
        .any(|alert| alert.rule == "viewport_image_missing_gpu_upload"));
}

#[test]
fn independent_gpu_items_without_batch_reduction_are_flagged() {
    let mut snapshot = ProfileSnapshot::default();
    snapshot
        .counters
        .push(counter("ui.idle_hover.gpu_draw_calls", 5.0));
    snapshot
        .counters
        .push(counter("ui.idle_hover.gpu_visible_draw_items", 5.0));

    let report = super::analyze_ui_hotspots(&snapshot);

    assert!(report
        .alerts
        .iter()
        .any(|alert| { alert.rule == "gpu_ui_batching_degenerated_without_depth_dependencies" }));
}
