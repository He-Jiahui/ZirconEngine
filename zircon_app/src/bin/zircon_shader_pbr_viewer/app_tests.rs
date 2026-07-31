use std::time::Duration;

use zircon_runtime::asset::importer::EnvironmentIblSourceStagingStatus;

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

fn resize_source() -> &'static str {
    APP_SOURCE
        .split("fn resize(")
        .nth(1)
        .and_then(|source| source.split("fn bind_scene_viewport_surface(").next())
        .expect("viewer app should retain a resize owner")
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
                Duration::from_millis(250),
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
        .find("let write_screenshot = self.screenshot_path.is_some()")
        .expect("screenshot flag");
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
}

#[test]
fn screenshot_export_occurs_once_after_the_ready_frame_is_rendered() {
    let render = render_source();

    assert!(render.contains("self.write_ready_frame_screenshot(&frame)"));
    assert!(
        render
            .find("self.write_ready_frame_screenshot(&frame)")
            .expect("screenshot write")
            < render
                .find("presenter.present(&frame)")
                .expect("present call"),
        "the screenshot must encode the same ready frame passed to the presenter"
    );
    assert!(render.contains("if self.exit_after_screenshot()"));
}

#[test]
fn ready_frame_timing_is_gated_to_the_one_shot_screenshot_path() {
    let render = render_source();

    assert!(render.contains(
        "let write_screenshot = self.screenshot_path.is_some() && !self.screenshot_written;"
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
    assert!(render.contains("let write_screenshot = self.screenshot_path.is_some()"));
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
    assert!(render.contains("if !self.direct_present_enabled || write_screenshot {"));
    assert!(render.contains("self.ensure_cpu_presenter()"));
    assert!(
        render
            .find("self.ensure_cpu_presenter()")
            .expect("CPU presenter fallback")
            < render
                .find("let Some(scene) = self.scene.as_mut()")
                .expect("scene borrow"),
        "CPU fallback setup must complete before borrowing the scene for rendering"
    );
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
