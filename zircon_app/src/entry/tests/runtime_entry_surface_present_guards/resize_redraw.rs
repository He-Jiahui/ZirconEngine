use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_app_source, runtime_application_handler_source, runtime_frame_loop_source,
    runtime_product_diagnostics_source, runtime_surface_present_source,
    runtime_window_creation_source, runtime_window_events_source,
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
        runtime_surface_present_source
            .contains("self.surface_present_enabled && !self.surface_present_failed"),
        "surface-present helper should skip native present or rebind after a surface-present failure"
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
            "self.surface_present_enabled && !self.surface_present_failed",
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
            "self.surface_present_enabled && !self.surface_present_failed",
            ".present_viewport(self.viewport, self.viewport_size)",
            "self.fail_surface_present();",
            "self.ensure_fallback_presenter(event_loop)",
            ".capture_frame(self.viewport, self.viewport_size)",
        ],
        "runtime redraw should fall back to capture_frame plus softbuffer after native present failure in the same branch",
    );
    for required_path in [
        "self.complete_first_presented_frame(event_loop);",
        "fn complete_first_presented_frame(&mut self, event_loop: &dyn ActiveEventLoop)",
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
            "fn complete_first_presented_frame",
            "self.capture_first_presented_frame_if_requested()",
            "if self.require_persisted_scene_diagnostics {",
            "self.emit_first_frame_product_diagnostics_once()",
            "first_presented_frame_diagnostic(self.exit_after_first_presented_frame)",
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
    assert_source_order(
        runtime_surface_present_source.as_str(),
        &[
            "fn fail_surface_present(&mut self)",
            "self.surface_present_failed = true;",
            "self.fallback_surface_present();",
            "fn ensure_fallback_presenter",
        ],
        "surface-present failure should mark failure before entering the softbuffer fallback path",
    );
    for unbind_path in [
        "self.session.unbind_viewport_surface(self.viewport)",
        "fn drop(&mut self)",
        "self.disable_surface_present();",
    ] {
        assert!(
            runtime_surface_present_source.contains(unbind_path),
            "runtime surface-present teardown should preserve `{unbind_path}`"
        );
    }
    for diagnostic in [
        "runtime_surface_present_enabled",
        "runtime_surface_present_fallback",
        "runtime_surface_present_failed",
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
        runtime_surface_present_source.contains("SoftbufferRuntimePresenter::new"),
        "runtime surface-present fallback should preserve SoftbufferRuntimePresenter construction"
    );
}
