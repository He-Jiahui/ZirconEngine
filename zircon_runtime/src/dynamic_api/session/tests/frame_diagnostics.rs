use crate::core::diagnostics::collect_runtime_diagnostics;
use zircon_runtime_interface::{
    ZrRuntimeFrameRequestV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::super::{
    extract_stats::{
        EXTRACT_CACHE_HITS_DIAGNOSTIC, EXTRACT_CACHE_MISSES_DIAGNOSTIC,
        EXTRACT_OUTPUT_BYTES_DIAGNOSTIC, EXTRACT_REBUILD_CLONES_DIAGNOSTIC,
    },
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
    let cache_hits =
        diagnostic_current(&diagnostics, EXTRACT_CACHE_HITS_DIAGNOSTIC).unwrap_or_default();
    let cache_misses =
        diagnostic_current(&diagnostics, EXTRACT_CACHE_MISSES_DIAGNOSTIC).unwrap_or_default();

    assert_eq!(
        rebuild_clones, 1.0,
        "headless capture should record one current full extract rebuild clone"
    );
    assert_eq!(
        cache_hits, 0.0,
        "initial headless capture should not report an extract cache hit"
    );
    assert_eq!(
        cache_misses, 1.0,
        "initial headless capture should report one extract cache miss"
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
    let cache_hits = diagnostic_series(&diagnostics, EXTRACT_CACHE_HITS_DIAGNOSTIC)
        .expect("extract cache-hit diagnostics");
    let cache_misses = diagnostic_series(&diagnostics, EXTRACT_CACHE_MISSES_DIAGNOSTIC)
        .expect("extract cache-miss diagnostics");

    assert_eq!(
        rebuilds.history.len(),
        2,
        "extract diagnostics should record one rebuild sample per capture"
    );
    assert_eq!(
        rebuilds.history[0].value, 1.0,
        "first capture should build the initial dynamic-session extract cache"
    );
    assert_eq!(
        rebuilds.history[1].value, 0.0,
        "unchanged headless capture should reuse the cached extract"
    );
    assert_eq!(output_bytes.history.len(), 2);
    assert!(output_bytes.history[0].value > 0.0);
    assert_eq!(
        output_bytes.history[0].value, output_bytes.history[1].value,
        "unchanged headless captures should keep the extract output byte baseline stable"
    );
    assert_eq!(cache_hits.history.len(), 2);
    assert_eq!(cache_misses.history.len(), 2);
    assert_eq!(
        cache_hits.history[0].value, 0.0,
        "first capture should miss the dynamic-session extract cache"
    );
    assert_eq!(
        cache_hits.history[1].value, 1.0,
        "unchanged follow-up capture should hit the dynamic-session extract cache"
    );
    assert_eq!(
        cache_misses.history[0].value, 1.0,
        "first capture should record the initial dynamic-session extract cache miss"
    );
    assert_eq!(
        cache_misses.history[1].value, 0.0,
        "unchanged follow-up capture should not rebuild the dynamic-session extract cache"
    );
}

#[test]
fn frame_extract_rebuilds_after_scene_change() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless session");

    session
        .capture_frame(small_headless_frame_request())
        .expect("first headless capture");
    session.level.with_world_mut(|world| {
        let camera = world.active_camera();
        let mut transform = world.world_transform(camera).unwrap_or_default();
        transform.translation.x += 0.25;
        world
            .update_transform(camera, transform)
            .expect("test camera transform should be mutable");
    });
    session
        .capture_frame(small_headless_frame_request())
        .expect("changed headless capture");

    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let rebuilds = diagnostic_series(&diagnostics, EXTRACT_REBUILD_CLONES_DIAGNOSTIC)
        .expect("extract rebuild diagnostics");
    let cache_hits = diagnostic_series(&diagnostics, EXTRACT_CACHE_HITS_DIAGNOSTIC)
        .expect("extract cache-hit diagnostics");
    let cache_misses = diagnostic_series(&diagnostics, EXTRACT_CACHE_MISSES_DIAGNOSTIC)
        .expect("extract cache-miss diagnostics");

    assert_eq!(
        rebuilds
            .history
            .iter()
            .map(|sample| sample.value)
            .collect::<Vec<_>>(),
        vec![1.0, 1.0],
        "scene mutations should invalidate the dynamic-session extract cache"
    );
    assert_eq!(
        cache_hits
            .history
            .iter()
            .map(|sample| sample.value)
            .collect::<Vec<_>>(),
        vec![0.0, 0.0],
        "scene mutations should not report a cache hit after invalidation"
    );
    assert_eq!(
        cache_misses
            .history
            .iter()
            .map(|sample| sample.value)
            .collect::<Vec<_>>(),
        vec![1.0, 1.0],
        "scene mutations should report a cache miss after invalidation"
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
    let frame_ms =
        frame_ms.expect("runtime diagnostics should report time.frame_time for the vampire scene");
    assert!(
        fps.is_finite() && fps > 0.0,
        "vampire runtime diagnostics should report a finite positive FPS after real-backend gameplay and capture, fps={fps:?} frame_ms={frame_ms:?}"
    );
    assert!(
        frame_ms.is_finite() && frame_ms > 0.0,
        "vampire runtime diagnostics should report a finite positive frame time after real-backend gameplay and capture, fps={fps:?} frame_ms={frame_ms:?}"
    );
}
