use crate::runtime_diagnostics::collect_runtime_diagnostics;
use crate::scene::ecs::{
    ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC, ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC,
    ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC,
    ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC,
};
use zircon_runtime_interface::{
    ZrRuntimeFrameRequestV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::super::{
    extract_stats::{
        EXTRACT_CACHE_HITS_DIAGNOSTIC, EXTRACT_CACHE_MISSES_DIAGNOSTIC,
        EXTRACT_FULL_CLONES_DIAGNOSTIC, EXTRACT_FULL_CLONE_BYTES_DIAGNOSTIC,
        EXTRACT_OUTPUT_BYTES_DIAGNOSTIC, EXTRACT_REBUILD_CLONES_DIAGNOSTIC,
        EXTRACT_STATS_PAYLOAD_SCANS_DIAGNOSTIC,
    },
    RuntimeDynamicSession, RuntimeDynamicSessionProfile,
};
use super::vampire_runtime_support::*;

#[test]
fn headless_session_tick_publishes_ecs_frame_diagnostics() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless session");

    session.tick_frame().expect("headless tick");

    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    assert!(
        diagnostic_current(&diagnostics, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC).is_some(),
        "a completed runtime frame must publish the ECS query-cache sample even when the count is zero"
    );
    assert!(
        diagnostic_current(&diagnostics, ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC,).is_some(),
        "a completed runtime frame must publish the ECS change-detection sample even when the count is zero"
    );
}

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
    let full_clones =
        diagnostic_current(&diagnostics, EXTRACT_FULL_CLONES_DIAGNOSTIC).unwrap_or_default();
    let full_clone_bytes =
        diagnostic_current(&diagnostics, EXTRACT_FULL_CLONE_BYTES_DIAGNOSTIC).unwrap_or_default();
    let cache_hits =
        diagnostic_current(&diagnostics, EXTRACT_CACHE_HITS_DIAGNOSTIC).unwrap_or_default();
    let cache_misses =
        diagnostic_current(&diagnostics, EXTRACT_CACHE_MISSES_DIAGNOSTIC).unwrap_or_default();

    assert_eq!(
        rebuild_clones, 1.0,
        "headless capture should record one scene-generation rebuild"
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
    assert_eq!(
        full_clones, 0.0,
        "building the extract cache must retain shared scene handles without a deep frame clone"
    );
    assert_eq!(
        full_clone_bytes, 0.0,
        "cache population must not copy the scene-generation payload"
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
    let full_clones = diagnostic_series(&diagnostics, EXTRACT_FULL_CLONES_DIAGNOSTIC)
        .expect("extract full-clone diagnostics");
    let full_clone_bytes = diagnostic_series(&diagnostics, EXTRACT_FULL_CLONE_BYTES_DIAGNOSTIC)
        .expect("extract full-clone byte diagnostics");
    let cache_hits = diagnostic_series(&diagnostics, EXTRACT_CACHE_HITS_DIAGNOSTIC)
        .expect("extract cache-hit diagnostics");
    let cache_misses = diagnostic_series(&diagnostics, EXTRACT_CACHE_MISSES_DIAGNOSTIC)
        .expect("extract cache-miss diagnostics");
    let stats_payload_scans =
        diagnostic_series(&diagnostics, EXTRACT_STATS_PAYLOAD_SCANS_DIAGNOSTIC)
            .expect("extract payload-stat scan diagnostics");

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
    assert_eq!(
        full_clones
            .history
            .iter()
            .map(|sample| sample.value)
            .collect::<Vec<_>>(),
        vec![0.0, 0.0],
        "cache population and stable reuse must clone only shared scene handles"
    );
    assert_eq!(full_clone_bytes.history.len(), 2);
    assert_eq!(
        full_clone_bytes
            .history
            .iter()
            .map(|sample| sample.value)
            .collect::<Vec<_>>(),
        vec![0.0, 0.0],
        "neither cache population nor stable reuse may copy scene payload bytes"
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
    assert_eq!(
        stats_payload_scans
            .history
            .iter()
            .map(|sample| sample.value)
            .collect::<Vec<_>>(),
        vec![1.0, 0.0],
        "a stable cache hit must reuse its immutable extract summary instead of rescanning the payload"
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

#[ignore = "real ZrVM coverage moved to the zr_vm_language plugin owner"]
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
    let query_cache_hits =
        diagnostic_current(&diagnostics, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC);
    let query_cache_misses =
        diagnostic_current(&diagnostics, ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC);
    let query_cache_rebuilds =
        diagnostic_current(&diagnostics, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC);
    let query_candidates =
        diagnostic_current(&diagnostics, ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC);
    let query_matches = diagnostic_current(&diagnostics, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC);
    let change_scanned =
        diagnostic_current(&diagnostics, ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC);
    let change_added =
        diagnostic_current(&diagnostics, ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC);
    let change_changed = diagnostic_current(
        &diagnostics,
        ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC,
    );
    let extract_rebuilds = diagnostic_current(&diagnostics, EXTRACT_REBUILD_CLONES_DIAGNOSTIC);
    let extract_output_bytes = diagnostic_current(&diagnostics, EXTRACT_OUTPUT_BYTES_DIAGNOSTIC);
    let extract_cache_hits = diagnostic_current(&diagnostics, EXTRACT_CACHE_HITS_DIAGNOSTIC);
    let extract_cache_misses = diagnostic_current(&diagnostics, EXTRACT_CACHE_MISSES_DIAGNOSTIC);
    let render_stats = diagnostics
        .render
        .stats
        .as_ref()
        .expect("render stats should be available after capture");

    println!(
        "vampire_runtime_perf ticks={} capture={}x{} fps_current={:?} frame_ms_current={:?} submitted_frames={} graph_passes={} ui_passes={} particle_passes={} shadow_passes={} mesh_draws={} ui_commands={} query_cache_hits={:?} query_cache_misses={:?} query_cache_rebuilds={:?} query_candidates={:?} query_matches={:?} change_scanned={:?} change_added={:?} change_changed={:?} extract_rebuilds={:?} extract_output_bytes={:?} extract_cache_hits={:?} extract_cache_misses={:?}",
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
        query_cache_hits,
        query_cache_misses,
        query_cache_rebuilds,
        query_candidates,
        query_matches,
        change_scanned,
        change_added,
        change_changed,
        extract_rebuilds,
        extract_output_bytes,
        extract_cache_hits,
        extract_cache_misses,
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
