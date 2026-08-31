use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_app_source, runtime_application_handler_source, runtime_frame_loop_source,
    runtime_product_diagnostics_source, runtime_surface_present_source,
    runtime_surface_redraw_source, runtime_surface_resize_source, runtime_window_creation_source,
    runtime_window_events_source,
};

#[test]
fn runtime_surface_present_bind_resize_redraw_and_teardown_paths_stay_source_visible() {
    let runtime_app_source = runtime_app_source();
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_frame_loop_source = runtime_frame_loop_source();
    let runtime_product_diagnostics_source = runtime_product_diagnostics_source();
    let runtime_surface_present_source = runtime_surface_present_source();
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_creation_source = runtime_window_creation_source();

    assert!(
        runtime_app_source.contains("mod frame_capture;"),
        "runtime entry app should declare the shared first-frame capture writer at its root"
    );
    assert!(
        runtime_surface_present_source
            .contains("super::super::frame_capture::write_runtime_frame_png("),
        "surface-present redraw should reach the root frame-capture writer through the runtime entry app"
    );
    assert!(
        runtime_surface_present_source.contains("if self.surface_present_enabled {"),
        "surface-present helper should use native present only after a successful bind"
    );
    assert_source_order(
        runtime_window_creation_source,
        &[
            "fn create_primary_window_surface",
            "let viewport_size = ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));",
            "if let Err(error) = self.resize_viewport(viewport_size)",
            "self.bind_window_surface(window.as_ref())",
        ],
        "runtime entry should resize the runtime viewport before initial surface binding",
    );
    for bind_path in [
        "runtime_native_surface_target(window)",
        "ZrRuntimeBindViewportSurfaceRequestV1::new(",
        "self.session\n            .bind_viewport_surface",
    ] {
        assert!(
            runtime_surface_present_source.contains(bind_path),
            "runtime entry surface bind path should preserve `{bind_path}`"
        );
    }
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::SurfaceResized(size)",
            "self.resize_surface_presenter(event_loop, size);",
        ],
        "runtime surface resize event handling should delegate presenter resize and rebind work",
    );
    assert_source_order(
        runtime_surface_present_source.as_str(),
        &[
            "fn resize_surface_presenter",
            "let viewport_size = ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));",
            "if let Err(error) = self.resize_viewport(viewport_size)",
            "if self.surface_present_enabled {",
            "self.bind_current_window_surface()",
            "if let Err(error) = presenter.resize(viewport_size)",
        ],
        "runtime surface resize should resize the runtime viewport before rebinding the active surface",
    );
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::RedrawRequested",
            "self.present_redraw_frame(event_loop);",
        ],
        "runtime redraw event handling should delegate presenter integration to the surface-present module",
    );
    assert_source_order(
        runtime_surface_present_source.as_str(),
        &[
            "fn present_redraw_frame",
            "if self.surface_present_enabled {",
            ".present_viewport(self.viewport, self.viewport_size)",
            "self.ensure_reference_cpu_presenter(event_loop)",
            ".capture_frame(self.viewport, self.viewport_size)",
        ],
        "runtime redraw should use capture_frame plus the explicit reference CPU presenter only when selected",
    );
    for required_path in [
        "self.complete_presented_frame(event_loop);",
        "fn complete_presented_frame(&mut self, event_loop: &dyn ActiveEventLoop)",
        "fn emit_first_frame_product_diagnostics_once(&mut self)",
    ] {
        assert!(
            runtime_surface_present_source.contains(required_path),
            "runtime first-frame startup smoke should keep `{required_path}` in the surface-present redraw owner"
        );
    }
    assert_source_order(
        runtime_surface_present_source.as_str(),
        &[
            "fn complete_presented_frame",
            "self.capture_first_presented_frame_if_requested()",
            "if self.require_persisted_scene_diagnostics {",
            "self.emit_first_frame_product_diagnostics_once()",
            "self.presented_frame_count = self.presented_frame_count.saturating_add(1);",
            "presented_frame_exit_diagnostic(",
        ],
        "runtime first-frame startup smoke should complete diagnostics before applying its exit policy",
    );
    assert_source_order(
        runtime_surface_present_source.as_str(),
        &[
            "fn emit_first_frame_product_diagnostics_once",
            "if should_emit_first_frame_product_diagnostics(",
            "self.emit_first_frame_product_diagnostics()?;",
            "self.first_frame_product_diagnostics_emitted = true;",
        ],
        "runtime product diagnostics should be emitted once for the first successful present",
    );
    for required_path in [
        "ProfileControlCommand::RuntimeDiagnosticsSnapshot",
        "project_identity",
        "scene_uri",
        "render_backend_name",
        "render.graph.executed_pass_count",
        "render.mesh.queue.draw_count",
        "render.light.directional.count",
    ] {
        assert!(
            runtime_product_diagnostics_source.contains(required_path),
            "runtime product diagnostics should preserve `{required_path}`"
        );
    }
    assert!(
        runtime_surface_present_source.contains("runtime_product_teardown surface_unbind="),
        "runtime surface-present lifecycle should record its product teardown result"
    );
    assert!(
        !runtime_surface_present_source.contains("fn fail_surface_present(&mut self)"),
        "explicit native surface failures must not retain a helper that converts them into fallback success"
    );
    for unbind_path in [
        "self.session.unbind_viewport_surface(self.viewport)",
        "fn teardown_surface_present(&mut self)",
        "format!(\"runtime surface unbind failed: {error}\")",
        "fn drop(&mut self)",
        "self.teardown_primary_window();",
    ] {
        assert!(
            runtime_surface_present_source.contains(unbind_path),
            "runtime surface-present teardown should preserve `{unbind_path}`"
        );
    }
    assert_source_order(
        runtime_surface_present_source.as_str(),
        &[
            "fn teardown_primary_window(&mut self) -> bool",
            "let surface_released = self.teardown_surface_present();",
            "if let Some(presenter) = self.presenter.take() {",
            "presenter.publish_summary();",
            "self.window = None;",
            "surface_released",
            "fn disable_surface_present",
            "write_warn(",
            "fn teardown_surface_present(&mut self) -> bool",
            "self.report_fatal_failure(",
            "fn release_surface_present",
        ],
        "primary-window teardown should release runtime surface ownership and publish degraded-presenter cost before invalidating host presentation state",
    );
    let drop_source = runtime_surface_present_source
        .split("impl Drop for RuntimeEntryApp")
        .nth(1)
        .expect("runtime entry app should retain explicit teardown");
    assert!(
        drop_source.contains("let _ = self.teardown_primary_window();"),
        "RuntimeEntryApp Drop should release the presenter and report terminal surface-unbind failures"
    );
    assert!(
        !runtime_surface_present_source.contains("close_primary_window_after_request"),
        "window teardown should have one generic owner instead of a close-request-only helper"
    );
    for diagnostic in [
        "runtime_surface_present_enabled",
        "runtime_reference_cpu_presenter_enabled",
    ] {
        assert!(
            runtime_surface_present_source.contains(diagnostic),
            "runtime surface-present diagnostic `{diagnostic}` should remain source-visible"
        );
    }
    assert_source_order(
        runtime_handler_source,
        &["fn about_to_wait", "self.pump_frame_loop(event_loop);"],
        "runtime about-to-wait hook should delegate frame-loop pumping",
    );
    for required_path in ["tick_frame", "request_redraw"] {
        assert!(
            runtime_frame_loop_source.contains(required_path),
            "runtime entry surface-present switch should preserve `{required_path}`"
        );
    }
    for required_path in [
        "present_viewport",
        "capture_frame",
        "resize_surface_presenter",
    ] {
        assert!(
            runtime_surface_present_source.contains(required_path),
            "runtime surface-present module should preserve `{required_path}`"
        );
    }
    assert!(
        runtime_surface_present_source.contains("ReferenceCpuPresenter::new"),
        "runtime surface-present owner should preserve ReferenceCpuPresenter construction"
    );
}

#[test]
fn explicit_native_surface_failure_branches_remain_fail_closed_before_fallback() {
    let window_creation = runtime_window_creation_source();
    let redraw = runtime_surface_redraw_source();
    let resize = runtime_surface_resize_source();
    let initial_bind = window_creation
        .split("match self.bind_window_surface(window.as_ref())")
        .nth(1)
        .expect("window creation must retain the initial native surface bind match");

    assert_source_order(
        initial_bind,
        &[
            "Err(error) => {",
            "self.report_fatal_failure(",
            "runtime window surface bind failed: {error}",
            "event_loop.exit();",
            "return false;",
        ],
        "an explicit initial native surface bind error must terminate product startup",
    );
    assert_source_order(
        redraw,
        &[
            ".present_viewport(self.viewport, self.viewport_size)",
            "Ok(false) => {",
            "self.report_fatal_failure(",
            "native surface presentation returned unavailable after a successful bind",
            "event_loop.exit();",
            "return;",
            "Err(error) => {",
            "self.report_fatal_failure(",
            "native surface presentation failed: {error}",
            "event_loop.exit();",
            "return;",
            "self.ensure_reference_cpu_presenter(event_loop)",
        ],
        "native presentation failures must terminate before the explicit reference CPU path",
    );
    assert_source_order(
        resize,
        &[
            "self.bind_current_window_surface()",
            "Ok(false) => {",
            "self.report_fatal_failure(",
            "native surface rebind returned unavailable after a successful bind",
            "event_loop.exit();",
            "return;",
            "Err(error) => {",
            "self.report_fatal_failure(",
            "native surface rebind failed: {error}",
            "event_loop.exit();",
            "return;",
        ],
        "native rebind failures must terminate instead of degrading into fallback success",
    );
}
