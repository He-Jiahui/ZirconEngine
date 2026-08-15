use std::time::Duration;

use zircon_runtime::asset::importer::{
    EnvironmentIblSourceStagingOutput, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming,
};

use super::{
    consume_ready_title_update, load_status_refresh_deadline, load_status_refresh_is_due,
    loading_window_title, ready_window_title, request_redraw_transition, OrbitCamera,
    PbrMirrorSceneIblLoadReport,
};

const APP_SOURCE: &str = include_str!("app.rs");

fn ensure_window_source() -> &'static str {
    APP_SOURCE
        .split("fn ensure_window(")
        .nth(1)
        .and_then(|source| source.split("fn start_scene_load(").next())
        .expect("viewer app should retain an ensure_window owner")
}

fn start_scene_load_source() -> &'static str {
    APP_SOURCE
        .split("fn start_scene_load(")
        .nth(1)
        .and_then(|source| source.split("fn finish_scene_load(").next())
        .expect("viewer app should retain a scene-load start owner")
}

fn resize_source() -> &'static str {
    APP_SOURCE
        .split("fn resize(")
        .nth(1)
        .and_then(|source| source.split("fn bind_scene_viewport_surface(").next())
        .expect("viewer app should retain a resize owner")
}

fn scene_load_failure_source() -> &'static str {
    APP_SOURCE
        .split("fn handle_scene_load_failure(")
        .nth(1)
        .and_then(|source| source.split("fn request_redraw(").next())
        .expect("viewer app should retain a scene-load failure owner")
}

fn about_to_wait_source() -> &'static str {
    APP_SOURCE
        .split("fn about_to_wait(")
        .nth(1)
        .and_then(|source| source.split("\n    }\n}").next())
        .expect("viewer app should retain an about_to_wait owner")
}

fn production_source() -> &'static str {
    APP_SOURCE
        .split("#[cfg(test)]")
        .next()
        .expect("viewer app should retain production source before tests")
}

fn render_source() -> &'static str {
    APP_SOURCE
        .split("fn render_and_present(")
        .nth(1)
        .and_then(|source| source.split("fn present_startup_frame(").next())
        .expect("viewer must retain a render owner")
}

fn assert_source_order(anchors: &[&str]) {
    assert_source_order_in(render_source(), anchors);
}

fn assert_source_order_in(source: &str, anchors: &[&str]) {
    let mut offset = 0;
    for anchor in anchors {
        let index = source[offset..]
            .find(anchor)
            .unwrap_or_else(|| panic!("render source should retain `{anchor}`"));
        offset += index + anchor.len();
    }
}

#[test]
fn loading_title_reports_elapsed_time_and_responsive_state() {
    assert_eq!(
        loading_window_title(Duration::from_millis(12_999)),
        "Zircon PBR HDRI Mirror Viewer - preparing HDRI/PMREM - 12s - window responsive"
    );
}

#[test]
fn ready_title_reports_current_orbit_angles() {
    assert_eq!(
        ready_window_title(
            OrbitCamera::from_angles(120.0, -120.0),
            Some(PbrMirrorSceneIblLoadReport::new(
                EnvironmentIblSourceStagingStatus::Reused,
                Duration::from_millis(125),
                EnvironmentIblSourceStagingTiming::default(),
                EnvironmentIblSourceStagingOutput::default(),
                Duration::from_millis(250),
                512,
                10,
                256,
                9,
            )),
        ),
        "Zircon PBR HDRI Mirror Viewer - Ready - IBL Reused staging 125ms total 250ms - yaw 120 pitch -120"
    );
}

#[test]
fn missing_ibl_report_never_claims_ready() {
    assert_eq!(
        ready_window_title(OrbitCamera::default(), None),
        "Zircon PBR HDRI Mirror Viewer - Loading - IBL unavailable - yaw 0 pitch 0"
    );
}

#[test]
fn one_shot_runs_exit_after_a_scene_load_failure() {
    assert!(super::one_shot_run_exits_after_load_failure(
        true, false, false
    ));
    assert!(super::one_shot_run_exits_after_load_failure(
        false, true, true
    ));
    assert!(!super::one_shot_run_exits_after_load_failure(
        false, false, false
    ));
    assert!(!super::one_shot_run_exits_after_load_failure(
        false, true, false
    ));
}

#[test]
fn production_ready_titles_require_a_loaded_ibl_report() {
    let production = production_source();

    assert!(production.contains("let Some(report) = ibl_load_report else {"));
    assert!(production.contains("Some(scene.ibl_load_report()),"));
    assert!(!production.contains("ready_window_title(self.camera, None)"));
}

#[test]
fn wait_callback_does_not_repeat_an_already_queued_redraw_request() {
    let wait_callback = about_to_wait_source();

    assert!(wait_callback.contains("self.finish_scene_load(event_loop);"));
    assert!(wait_callback.contains("self.refresh_scene_load_status(event_loop);"));
    assert!(wait_callback.contains("self.request_base_pipeline_recheck_if_due(event_loop);"));
    assert!(
        !wait_callback.contains("window.request_redraw()"),
        "request_redraw() owns the only OS redraw transition; about_to_wait must not repeat it"
    );
}

#[test]
fn redraw_transition_coalesces_requests_until_the_frame_is_consumed() {
    let mut redraw_requested = false;

    assert!(request_redraw_transition(&mut redraw_requested));
    assert!(redraw_requested);
    assert!(!request_redraw_transition(&mut redraw_requested));

    redraw_requested = false;
    assert!(request_redraw_transition(&mut redraw_requested));
}

#[test]
fn loading_title_refresh_is_rate_limited_without_deadline_drift() {
    let started_at = std::time::Instant::now();

    assert!(!load_status_refresh_is_due(Some(started_at), started_at));
    assert!(!load_status_refresh_is_due(
        Some(started_at),
        started_at + Duration::from_millis(999)
    ));
    assert!(load_status_refresh_is_due(
        Some(started_at),
        started_at + Duration::from_secs(1)
    ));
    assert!(load_status_refresh_is_due(None, started_at));
    assert_eq!(
        load_status_refresh_deadline(Some(started_at), started_at + Duration::from_millis(250)),
        started_at + Duration::from_secs(1)
    );
}

#[test]
fn base_pipeline_recheck_backoff_resets_after_readiness_or_load_failure() {
    let render = render_source();

    assert_source_order(&[
        "if environment_only_base_pipeline_ready {",
        "self.reset_base_pipeline_recheck();",
    ]);
    assert!(start_scene_load_source().contains("self.reset_base_pipeline_recheck();"));
    assert!(scene_load_failure_source().contains("self.reset_base_pipeline_recheck();"));
}

#[test]
fn ready_title_updates_are_coalesced_until_a_successful_present() {
    let mut dirty = false;

    assert!(!consume_ready_title_update(&mut dirty));
    dirty = true;
    assert!(consume_ready_title_update(&mut dirty));
    assert!(!consume_ready_title_update(&mut dirty));
}

#[test]
fn screenshot_timing_requests_the_next_scene_frame_before_rendering() {
    let render = render_source();
    let screenshot_flag = render
        .find("let screenshot_requested = self.screenshot_path.is_some()")
        .expect("screenshot request flag");
    let timing_request = render
        .find("scene.request_next_frame_timing_report();")
        .expect("screenshot should request next-frame timing");
    let render_call = render
        .find("match scene.render(&self.camera, self.size)")
        .expect("scene render call");

    assert!(
        screenshot_flag < timing_request && timing_request < render_call,
        "the timing request must arm the screenshot or one-shot capture frame before scene rendering"
    );
    assert!(render.contains("if write_screenshot || capture_this_frame {"));
}

#[test]
fn orbit_input_defers_ready_title_writes_until_after_present() {
    let pointer = APP_SOURCE
        .split("fn update_pointer_position(")
        .nth(1)
        .and_then(|source| source.split("fn handle_pointer_button(").next())
        .expect("viewer must retain a pointer-input owner");
    let wheel = APP_SOURCE
        .split("fn handle_mouse_wheel(")
        .nth(1)
        .and_then(|source| source.split("fn mark_ready_window_title_dirty(").next())
        .expect("viewer must retain a wheel-input owner");
    let render = render_source();

    assert!(pointer.contains("self.mark_ready_window_title_dirty();"));
    assert!(!pointer.contains("set_title"));
    assert!(wheel.contains("self.mark_ready_window_title_dirty();"));
    assert!(!wheel.contains("set_title"));
    let direct = render
        .split("if self.direct_present_enabled && !write_screenshot {")
        .nth(1)
        .and_then(|source| source.split("let ready_frame_render_started").next())
        .expect("native presentation branch");
    let cpu = render
        .split("let ready_frame_render_started")
        .nth(1)
        .expect("CPU presentation branch");
    assert!(
        direct
            .find("scene.render_to_viewport_surface(&self.camera, self.size)")
            .expect("direct present call")
            < direct
                .find("self.flush_ready_window_title();")
                .expect("direct ready-title flush"),
        "the Ready title must not update before the direct surface presents"
    );
    assert!(
        cpu.find("presenter.present(&frame)")
            .expect("CPU present call")
            < cpu
                .find("self.flush_ready_window_title();")
                .expect("CPU ready-title flush"),
        "the Ready title must not update before the CPU surface presents"
    );
}

#[test]
fn startup_status_frame_does_not_schedule_a_second_loading_redraw() {
    let ensure_window = ensure_window_source();
    assert!(ensure_window.contains("self.present_startup_frame(event_loop);"));
    assert!(ensure_window.contains("self.start_scene_load(event_loop);"));
    assert!(
        !ensure_window.contains("self.request_redraw();"),
        "the startup status frame is already presented synchronously"
    );
    assert!(
        !ensure_window.contains("window.request_redraw();"),
        "loading must wait for the background loader instead of redrawing the same status frame"
    );
}

#[test]
fn completed_scene_load_presents_the_first_ready_frame_synchronously() {
    let finish_scene_load = APP_SOURCE
        .split("fn finish_scene_load(")
        .nth(1)
        .and_then(|source| source.split("fn refresh_scene_load_status(").next())
        .expect("viewer app should retain a scene-load completion owner");

    let scene_assignment = finish_scene_load
        .find("self.scene = Some(scene);")
        .expect("scene assignment");
    let first_ready_redraw = finish_scene_load
        .find("self.redraw_requested = true;")
        .expect("first ready redraw state");
    let first_ready_present = finish_scene_load
        .find("self.render_and_present(event_loop);")
        .expect("first ready present");
    assert!(
        scene_assignment < first_ready_redraw && first_ready_redraw < first_ready_present,
        "the first ready frame must render after the loaded scene is installed"
    );
    assert!(
        !finish_scene_load.contains("self.request_redraw();"),
        "scene completion must not rely on a platform redraw after publishing Ready"
    );
    assert!(
        finish_scene_load.contains("self.ready_title_dirty = true;"),
        "the successful first present must own the Ready title transition"
    );
    assert!(finish_scene_load.contains("self.first_ready_frame_started_at = Some(Instant::now());"));
    assert!(
        !finish_scene_load.contains("PBR viewer readiness:"),
        "the loader must not report a first presented frame before one exists"
    );
}

#[test]
fn readiness_timing_logs_after_the_first_successful_presentation() {
    let render = render_source();
    let direct = render
        .split("if self.direct_present_enabled && !write_screenshot {")
        .nth(1)
        .and_then(|source| source.split("let ready_frame_render_started").next())
        .expect("native presentation branch");
    let cpu = render
        .split("let ready_frame_render_started")
        .nth(1)
        .expect("CPU presentation branch");

    assert_eq!(
        render
            .matches("self.log_first_ready_frame_presented();")
            .count(),
        2,
        "direct and CPU first frames must share one post-present readiness logger"
    );
    assert!(
        direct
            .find("self.flush_ready_window_title();")
            .expect("direct present completion")
            < direct
                .find("self.log_first_ready_frame_presented();")
                .expect("direct readiness log"),
        "direct readiness timing must follow a successful presentation"
    );
    assert!(
        cpu.find("presenter.present(&frame)")
            .expect("CPU present call")
            < cpu
                .find("self.log_first_ready_frame_presented();")
                .expect("CPU readiness log"),
        "CPU readiness timing must follow a successful presentation"
    );
}

#[test]
fn screenshot_export_occurs_once_after_the_ready_frame_is_rendered() {
    let render = render_source();

    assert!(render.contains("let screenshot_metadata = if write_screenshot {"));
    assert!(
        render.contains("self.write_ready_frame_screenshot(&frame, screenshot_metadata.as_ref())")
    );
    assert!(
        render
            .find("self.write_ready_frame_screenshot(&frame, screenshot_metadata.as_ref())")
            .expect("screenshot write")
            < render
                .find("presenter.present(&frame)")
                .expect("present call"),
        "the screenshot must encode the same ready frame passed to the presenter"
    );
    assert!(render.contains("if self.exit_after_screenshot()"));
}

#[test]
fn bridged_renderdoc_capture_requires_an_actual_capture_record() {
    let production = production_source();
    let finish_capture = production
        .split("fn finish_graphics_debugger_capture(")
        .nth(1)
        .and_then(|source| source.split("fn viewport_surface_descriptor(").next())
        .expect("viewer must retain one shared capture completion owner");

    for expected in [
        "scene.stop_graphics_debugger_capture()",
        "bridge.capture_report()",
        "report.capture_path_for_evidence()",
        "graphics debugger capture completed",
    ] {
        assert!(
            finish_capture.contains(expected),
            "bridged RenderDoc capture completion must retain `{expected}`"
        );
    }
    assert_eq!(
        production
            .matches("finish_graphics_debugger_capture(")
            .count(),
        3,
        "both direct and CPU capture paths must share the actual-record gate"
    );
}

#[test]
fn screenshot_export_writes_versioned_ibl_provenance() {
    let production = production_source();
    let render = render_source();

    for expected in [
        "IBL_BAKE_ALGORITHM_VERSION",
        "scene.ibl_load_report()",
        "scene.renderer_backend_name()",
        "interactive_direct_present_enabled",
        "scene.environment_only_base_pipeline_ready()",
        "let screenshot_input = write_screenshot.then(|| {",
        "self.hdri_path.display().to_string()",
        "requested_source_face_size",
        "requested_pmrem_face_size",
        "ibl_report.source_cubemap_face_size()",
        "ibl_report.source_cubemap_mip_count()",
        "ibl_report.pmrem_face_size()",
        "ibl_report.pmrem_mip_count()",
        "PBR_VIEWER_RENDER_PROFILE",
        "scene.base_prewarm_report()",
        "scene.shader_variant_miss_report()",
        "scene.startup_timing()",
        "base_prewarm_report.pipeline_ready()",
        "environment_only_base_pipeline_ready_at_capture:",
        "environment_only_base_pipeline_ready,",
        "base_prewarm_report.cache_hit()",
        "base_prewarm_report.shader_source_resolution()",
        "base_prewarm_report.pipeline_creation()",
        "base_prewarm_report.elapsed()",
        "scene_startup_renderer_deferred_standard_pipeline: startup_timing",
        "scene_startup_total: startup_timing.total()",
        "ibl_staging_equirect_projection_parallel_work_items: ibl_staging_output",
        "ibl_staging_source_mip_build_parallel_work_items: ibl_staging_output",
        "ibl_staging_pmrem_build_parallel_work_items: ibl_staging_output",
        "ibl_staging_irradiance_cube_build_parallel_work_items: ibl_staging_output",
        "ibl_staging_irradiance_cube_source_sample_visits: ibl_staging_output",
        "one_shot_base_pipeline_wait_elapsed",
        "viewer_scene_load_elapsed",
        "viewer_ready_started_at",
        "viewer_ready_elapsed",
        "shader_variant_miss_report,",
        "ReadyFrameEvidenceMetadata {",
        "write_ready_frame_evidence(path, frame.width, frame.height, &frame.rgba, metadata)",
    ] {
        assert!(
            production.contains(expected),
            "Ready-frame screenshot provenance must retain `{expected}`"
        );
    }
    assert!(
        render
            .find("let screenshot_metadata = if write_screenshot {")
            .expect("metadata should be built for screenshot evidence")
            < render
                .find("self.write_ready_frame_screenshot(&frame, screenshot_metadata.as_ref())")
                .expect("metadata should be written with the screenshot"),
        "provenance must be captured from the same Ready frame before the evidence files are written"
    );
    let ready_frame_pipeline_gate = render
        .split(
            "let write_screenshot = screenshot_requested && environment_only_base_pipeline_ready;",
        )
        .nth(1)
        .and_then(|source| {
            source
                .split("let screenshot_input = write_screenshot.then(|| {")
                .next()
        })
        .expect("Ready-frame evidence must retain the Base pipeline gate before screenshot input");
    assert!(
        ready_frame_pipeline_gate
            .find("let one_shot_base_pipeline_wait_elapsed = write_screenshot")
            .expect("Ready-frame evidence must capture the one-shot Base pipeline wait")
            < ready_frame_pipeline_gate
                .find("if environment_only_base_pipeline_ready {")
                .expect("Ready-frame evidence must reset the Base pipeline recheck state"),
        "the one-shot Base pipeline wait must be captured before retry state is reset"
    );
}

#[test]
fn ready_frame_provenance_measures_through_base_pipeline_wait_and_render() {
    let production = production_source();
    let render = render_source();
    let viewer_ready_started_at = render
        .find("let viewer_ready_started_at = write_screenshot.then(|| {")
        .expect("Ready-frame evidence must retain the viewer load start instant");
    let pipeline_wait = render
        .find("if defer_one_shot_until_base_pipeline_ready {")
        .expect("Ready-frame evidence must retain the async Base pipeline wait");
    let ready_frame_render = render
        .find("let ready_frame_render_started = write_screenshot.then(Instant::now);")
        .expect("Ready-frame evidence must retain the Ready render boundary");
    let viewer_ready_elapsed = render
        .find("let viewer_ready_elapsed = viewer_ready_started_at")
        .expect("Ready-frame evidence must measure time to Ready after rendering");
    let render_elapsed = render
        .find("let render_elapsed = ready_frame_render_elapsed")
        .expect("Ready-frame evidence must retain the Ready render duration");
    let metadata = render
        .find("viewer_ready_elapsed,")
        .expect("Ready-frame metadata must emit the Ready total");

    assert!(
        viewer_ready_started_at < pipeline_wait
            && pipeline_wait < ready_frame_render
            && ready_frame_render < render_elapsed
            && render_elapsed < viewer_ready_elapsed
            && viewer_ready_elapsed < metadata,
        "time-to-Ready must include async Base admission and the completed Ready render"
    );
    assert!(production.contains("first_ready_scene_load_started_at: Option<Instant>"));
    assert!(production.contains("self.first_ready_scene_load_started_at = scene_load_started_at;"));
    assert!(production.contains(
        "let Some(scene_load_started_at) = self.first_ready_scene_load_started_at else {"
    ));
    assert!(production
        .contains("let Some(scene_load_elapsed) = self.first_ready_scene_load_elapsed else {"));
    assert!(!production.contains(
        "let Some(scene_load_started_at) = self.first_ready_scene_load_started_at.take() else {"
    ));
    assert!(!production.contains(
        "let Some(scene_load_elapsed) = self.first_ready_scene_load_elapsed.take() else {"
    ));
    assert!(production.contains("self.first_ready_scene_load_started_at = None;"));
}

#[test]
fn ready_frame_timing_is_gated_to_the_one_shot_screenshot_path() {
    let render = render_source();

    assert!(render.contains(
        "let screenshot_requested = self.screenshot_path.is_some() && !self.screenshot_written;"
    ));
    assert!(render.contains(
        "let write_screenshot = screenshot_requested && environment_only_base_pipeline_ready;"
    ));
    assert_eq!(
        render
            .matches("write_screenshot.then(Instant::now)")
            .count(),
        3,
        "render, PNG encode, and present timing must only start for one screenshot frame"
    );
}

#[test]
fn one_shot_evidence_waits_for_the_async_base_pipeline_without_blocking_the_event_loop() {
    let render = render_source();

    assert_source_order(&[
        "let needs_environment_only_base_pipeline = screenshot_requested || capture_requested;",
        "scene.environment_only_base_pipeline_ready()",
        "let defer_one_shot_until_base_pipeline_ready = needs_environment_only_base_pipeline",
        "if defer_one_shot_until_base_pipeline_ready {",
        "if self.one_shot_base_pipeline_wait_has_expired(Instant::now()) {",
        "environment-only PBR Base pipeline startup timed out",
        "self.reset_base_pipeline_recheck();",
        "event_loop.exit();",
        "return;",
        ".expect(\"the one-shot timeout check must initialize its deadline\");",
        "self.schedule_base_pipeline_recheck(event_loop, Some(deadline));",
        "return;",
        "let screenshot_input = write_screenshot.then(|| {",
        "let capture_this_frame = capture_requested && environment_only_base_pipeline_ready;",
    ]);
    assert!(render.contains("if write_screenshot {"));
    assert!(
        render.contains("self.write_ready_frame_screenshot(&frame, screenshot_metadata.as_ref())")
    );
}

#[test]
fn pending_base_pipeline_retries_admission_before_scheduling_the_next_recheck() {
    let render = render_source();

    assert_source_order(&[
        "match scene.environment_only_base_pipeline_ready() {",
        "Ok(false) => {",
        "scene.retry_environment_only_base_pipeline_admission()",
        "environment-only PBR Base pipeline admission retry failed",
        "false",
        "let defer_one_shot_until_base_pipeline_ready = needs_environment_only_base_pipeline",
    ]);
}

#[test]
fn interactive_presentation_rechecks_a_pending_base_pipeline_after_presenting() {
    let render = render_source();
    let direct = render
        .split("if self.direct_present_enabled && !write_screenshot {")
        .nth(1)
        .and_then(|source| source.split("let ready_frame_render_started").next())
        .expect("native presentation branch");
    let cpu = render
        .split("let ready_frame_render_started")
        .nth(1)
        .expect("CPU presentation branch");

    assert!(render.contains("let recheck_base_pipeline_after_present ="));
    assert_source_order(&[
        "let recheck_base_pipeline_after_present =",
        "if defer_one_shot_until_base_pipeline_ready {",
        "self.schedule_base_pipeline_recheck(event_loop, Some(deadline));",
    ]);
    for branch in [direct, cpu] {
        assert_source_order_in(
            branch,
            &[
                "self.flush_ready_window_title();",
                "if recheck_base_pipeline_after_present {",
                "self.schedule_base_pipeline_recheck(event_loop, None);",
            ],
        );
    }
}

#[test]
fn interactive_frames_prefer_native_gpu_presentation_without_changing_screenshot_export() {
    let render = render_source();
    let direct = render
        .split("if self.direct_present_enabled && !write_screenshot {")
        .nth(1)
        .and_then(|source| source.split("let ready_frame_render_started").next())
        .expect("native presentation branch");

    assert!(direct.contains("scene.render_to_viewport_surface(&self.camera, self.size)"));
    assert!(!direct.contains("scene.render(&self.camera, self.size)"));
    assert!(direct.contains("PBR viewer Direct-present timing:"));
    assert!(direct.contains("render_and_present_call="));
    assert!(
        !direct.contains("render_submission="),
        "the direct viewer log must not mislabel its CPU wall-clock interval as GPU submission"
    );
    assert!(
        !direct.contains("renderer_frame_call="),
        "the direct viewer log must include surface presentation in its timing label"
    );
    assert!(render.contains(
        "let write_screenshot = screenshot_requested && environment_only_base_pipeline_ready;"
    ));
    assert!(render.contains("match scene.render(&self.camera, self.size)"));
}

#[test]
fn direct_presentation_releases_cpu_staging_and_rebuilds_it_for_a_fallback() {
    let finish_scene_load = APP_SOURCE
        .split("fn finish_scene_load(")
        .nth(1)
        .and_then(|source| source.split("fn refresh_scene_load_status(").next())
        .expect("viewer app should retain a scene-load completion owner");
    let render = render_source();

    assert!(finish_scene_load.contains("self.direct_present_enabled = true;"));
    assert!(finish_scene_load.contains("self.presenter = None;"));
    assert!(render.contains("if !self.direct_present_enabled || screenshot_requested {"));
    assert!(render.contains("self.ensure_cpu_presenter()"));
    assert!(
        render
            .find("self.ensure_cpu_presenter()")
            .expect("CPU presenter fallback")
            < render
                .find("PBR scene must remain available after startup-state query")
                .expect("rendering scene borrow"),
        "CPU fallback setup must complete before the rendering scene borrow"
    );
}

#[test]
fn screenshot_scoped_gpu_timing_is_opt_in_and_preserves_the_matching_frame_identity() {
    let load = start_scene_load_source();
    let render = render_source();

    assert_source_order_in(
        load,
        &[
            "let gpu_timing_enabled = self.gpu_timing_report_path.is_some();",
            "PbrMirrorScene::new(",
            "gpu_timing_enabled,",
        ],
    );
    assert_source_order(&[
        "self.write_ready_frame_screenshot(&frame, screenshot_metadata.as_ref())",
        "self.begin_gpu_timing_evidence(frame.generation);",
        "let gpu_timing_report = scene.take_completed_gpu_timing_report();",
        "let gpu_timing_status = scene.last_gpu_timing_status();",
        "self.resolve_gpu_timing_evidence(gpu_timing_report, gpu_timing_status)",
    ]);
    assert!(APP_SOURCE.contains("GpuTimingEvidenceRequest::new(frame_generation)"));
    assert!(include_str!("gpu_timing_evidence.rs")
        .contains("report.frame_generation() == self.target_generation"));
}

#[test]
fn screenshot_exit_waits_for_the_nonblocking_gpu_timing_result() {
    let render = render_source();

    assert_source_order_in(
        APP_SOURCE,
        &[
            "self.request_redraw();",
            "return Ok(false);",
            "self.gpu_timing_request = None;",
        ],
    );
    assert!(render.contains("&& !self.gpu_timing_evidence_pending()"));
    assert!(render.contains("if self.exit_after_screenshot()"));
}

#[test]
fn resize_requests_a_direct_surface_redraw_without_cpu_staging() {
    let resize = resize_source();

    assert!(resize.contains("if let Some(presenter) = self.presenter.as_mut()"));
    assert!(resize.contains("self.request_redraw();"));
}

#[test]
fn native_presentation_failure_requests_cpu_fallback_on_the_next_redraw() {
    let render = render_source();
    let direct = render
        .split("if self.direct_present_enabled && !write_screenshot {")
        .nth(1)
        .and_then(|source| source.split("let ready_frame_render_started").next())
        .expect("native presentation branch");
    let fallback = direct
        .rsplit("Err(error) =>")
        .next()
        .expect("native presentation error branch");

    assert!(fallback.contains("self.direct_present_enabled = false;"));
    assert!(fallback.contains("scene.detach_viewport_surface();"));
    assert!(fallback.contains("self.redraw_requested = true;"));
    assert!(fallback.contains("window.request_redraw();"));
    assert!(!fallback.contains("event_loop.exit();"));
}
