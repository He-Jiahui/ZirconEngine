use crate::core::diagnostics::collect_runtime_diagnostics;
use zircon_runtime_interface::{
    ZrRuntimeFrameRequestV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::super::{
    extract_stats::{EXTRACT_OUTPUT_BYTES_DIAGNOSTIC, EXTRACT_REBUILD_CLONES_DIAGNOSTIC},
    RuntimeDynamicSession, RuntimeDynamicSessionProfile,
};
use super::helpers::*;

#[test]
fn headless_session_capture_records_frame_extract_diagnostics() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless session");

    session
        .capture_frame(small_headless_frame_request())
        .expect("headless capture");

    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let rebuild_clones =
        diagnostic_current(&diagnostics, EXTRACT_REBUILD_CLONES_DIAGNOSTIC).unwrap_or_default();
    let output_bytes =
        diagnostic_current(&diagnostics, EXTRACT_OUTPUT_BYTES_DIAGNOSTIC).unwrap_or_default();

    assert_eq!(
        rebuild_clones, 1.0,
        "headless capture should record one current full extract rebuild clone"
    );
    assert!(
        output_bytes > 0.0,
        "headless capture should record a non-empty extract output byte estimate"
    );
}

#[test]
fn frame_extract_rebuild_skips_unchanged_entities() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless session");

    session
        .capture_frame(small_headless_frame_request())
        .expect("first headless capture");
    session
        .capture_frame(small_headless_frame_request())
        .expect("second headless capture");

    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let rebuilds = diagnostic_series(&diagnostics, EXTRACT_REBUILD_CLONES_DIAGNOSTIC)
        .expect("extract rebuild diagnostics");
    let output_bytes = diagnostic_series(&diagnostics, EXTRACT_OUTPUT_BYTES_DIAGNOSTIC)
        .expect("extract output byte diagnostics");

    assert_eq!(
        rebuilds.history.len(),
        2,
        "current baseline records one extract rebuild sample per capture"
    );
    assert!(
        rebuilds.history.iter().all(|sample| sample.value == 1.0),
        "current baseline still rebuilds the full extract for unchanged captures"
    );
    assert_eq!(output_bytes.history.len(), 2);
    assert!(output_bytes.history[0].value > 0.0);
    assert_eq!(
        output_bytes.history[0].value, output_bytes.history[1].value,
        "unchanged headless captures should keep the extract output byte baseline stable"
    );
}

#[test]
fn vampire_project_session_reports_runtime_fps_and_render_work() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);
    let tick_count = vampire_diagnostic_tick_count();

    for _ in 0..tick_count {
        session.tick_frame().unwrap();
    }

    let frame = session
        .capture_frame(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            vampire_capture_viewport_size(),
        ))
        .unwrap();
    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let fps = diagnostic_current(&diagnostics, "time.fps");
    let frame_ms = diagnostic_current(&diagnostics, "time.frame_time");
    let render_stats = diagnostics
        .render
        .stats
        .as_ref()
        .expect("render stats should be available after capture");

    println!(
        "vampire_runtime_perf ticks={} capture={}x{} fps_current={:?} frame_ms_current={:?} submitted_frames={} graph_passes={} ui_passes={} particle_passes={} shadow_passes={} mesh_draws={} ui_commands={}",
        tick_count,
        frame.width,
        frame.height,
        fps,
        frame_ms,
        render_stats.submitted_frames,
        render_stats.last_graph_executed_pass_count,
        render_stats.last_ui_graph_executed_pass_count,
        render_stats.last_particle_graph_executed_pass_count,
        render_stats.last_shadow_graph_executed_pass_count,
        render_stats.last_mesh_draw_count,
        render_stats.last_ui_command_count,
    );

    assert!(
        render_stats.submitted_frames > 0,
        "diagnostic run should submit at least one rendered frame"
    );
    let fps = fps.expect("runtime diagnostics should report time.fps for the vampire scene");
    assert!(
        fps >= 60.0,
        "vampire runtime diagnostics should remain at or above 60 FPS after hot-path trimming, fps={fps:?} frame_ms={frame_ms:?}"
    );
}
