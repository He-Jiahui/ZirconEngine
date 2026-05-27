use super::super::source_assertions::assert_source_order;
use super::sources::{
    runtime_application_handler_source, runtime_frame_loop_source, runtime_surface_present_source,
    runtime_window_creation_source, runtime_window_events_source,
};

#[test]
fn runtime_surface_present_bind_resize_redraw_and_teardown_paths_stay_source_visible() {
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_frame_loop_source = runtime_frame_loop_source();
    let runtime_surface_present_source = runtime_surface_present_source();
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_creation_source = runtime_window_creation_source();

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
            "self.resize_viewport(viewport_size).is_err()",
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
            "self.resize_viewport(viewport_size).is_err()",
            "self.surface_present_enabled && !self.surface_present_failed",
            "self.bind_current_window_surface()",
            "presenter.resize(viewport_size).is_err()",
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
