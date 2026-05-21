use super::source_assertions::assert_source_order;

fn runtime_surface_present_source() -> String {
    [
        include_str!("../runtime_entry_app/surface_present/mod.rs"),
        include_str!("../runtime_entry_app/surface_present/binding.rs"),
        include_str!("../runtime_entry_app/surface_present/lifecycle.rs"),
        include_str!("../runtime_entry_app/surface_present/fallback.rs"),
        include_str!("../runtime_entry_app/surface_present/redraw.rs"),
        include_str!("../runtime_entry_app/surface_present/resize.rs"),
    ]
    .join("\n")
}

fn runtime_window_surface_source() -> String {
    [
        include_str!("../runtime_entry_app/window_surface/mod.rs"),
        include_str!("../runtime_entry_app/window_surface/native_target.rs"),
    ]
    .join("\n")
}

fn runtime_window_events_source() -> String {
    [
        include_str!("../runtime_entry_app/window_events/mod.rs"),
        include_str!("../runtime_entry_app/window_events/dispatch.rs"),
    ]
    .join("\n")
}

fn runtime_application_handler_source() -> &'static str {
    include_str!("../runtime_entry_app/application_handler/hooks.rs")
}

#[test]
fn runtime_sources_route_preview_through_dynamic_api_without_app_wgpu_dependency() {
    let lib_source = include_str!("../../lib.rs");
    let production_lib_source = lib_source
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or(lib_source);
    let presenter_source = include_str!("../../runtime_presenter.rs");
    let runtime_surface_present_source = runtime_surface_present_source();
    let manifest = include_str!("../../../Cargo.toml");

    assert!(
        presenter_source.contains("softbuffer"),
        "runtime presenter should blit runtime-owned frame output through softbuffer"
    );
    assert!(
        runtime_surface_present_source.contains("capture_frame"),
        "runtime preview should request frames through the runtime dynamic API"
    );
    assert!(
        presenter_source.contains("RuntimeFrame"),
        "runtime presenter should consume runtime interface frames, not runtime implementation frames"
    );

    for forbidden in [
        "wgpu::",
        "RenderFrameExtract",
        "RenderFrameworkRuntimeBridge",
        "RuntimePreviewRenderer",
        "create_runtime_preview_renderer",
        "SharedTextureRenderService",
        "RenderService",
    ] {
        assert!(
            !production_lib_source.contains(forbidden),
            "runtime entry source should not reference `{forbidden}` after dynamic runtime migration"
        );
        assert!(
            !presenter_source.contains(forbidden),
            "runtime presenter source should not reference `{forbidden}` after dynamic runtime migration"
        );
    }

    assert!(
        !manifest.contains("wgpu.workspace = true"),
        "zircon_app/Cargo.toml should not depend on wgpu directly"
    );
}

#[test]
fn runtime_preview_keeps_softbuffer_fallback_when_surface_api_is_optional() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_surface_present_source = runtime_surface_present_source();
    let runtime_session_source = include_str!("../runtime_library/runtime_session.rs");

    assert!(
        runtime_app_source.contains("surface_present_enabled: bool"),
        "runtime entry app should track whether optional surface present is active"
    );
    assert!(
        runtime_app_source.contains("presenter: Option<SoftbufferRuntimePresenter>"),
        "softbuffer presenter should remain available as the fallback path"
    );
    assert!(
        runtime_session_source.contains("return Ok(false);")
            && runtime_session_source.contains("bind_viewport_surface")
            && runtime_session_source.contains("present_viewport"),
        "runtime session optional surface wrappers should report unavailable APIs without error"
    );
    assert!(
        runtime_surface_present_source.contains("SoftbufferRuntimePresenter::new"),
        "runtime surface-present helper should still be able to create the softbuffer fallback presenter"
    );
    assert!(
        runtime_surface_present_source.contains("capture_frame"),
        "runtime redraw fallback should continue using capture_frame()"
    );
}

#[test]
fn runtime_entry_surface_present_switch_keeps_diagnostics_and_fallback_paths() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_frame_loop_source = include_str!("../runtime_entry_app/frame_loop.rs");
    let runtime_surface_present_source = runtime_surface_present_source();
    let runtime_window_surface_source = runtime_window_surface_source();
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_creation_source = include_str!("../runtime_entry_app/window_creation.rs");

    assert!(
        runtime_app_source.contains("surface_present_failed: bool"),
        "runtime entry app should remember failed surface-present state"
    );
    assert!(
        runtime_app_source.contains("mod surface_present;"),
        "runtime entry app should keep native surface-present binding and fallback in a child module"
    );
    assert!(
        runtime_app_source.contains("mod window_surface;"),
        "runtime entry app should keep native window-surface target extraction in a child module"
    );
    assert!(
        runtime_surface_present_source.contains("mod binding;")
            && runtime_surface_present_source.contains("mod fallback;")
            && runtime_surface_present_source.contains("mod lifecycle;")
            && runtime_surface_present_source.contains("mod redraw;")
            && runtime_surface_present_source.contains("mod resize;"),
        "runtime surface-present root should stay structural and declare focused helper families"
    );
    assert!(
        runtime_window_surface_source.contains("mod native_target;")
            && runtime_window_surface_source
                .contains("pub(in crate::entry::runtime_entry_app) use native_target::runtime_native_surface_target;"),
        "runtime window-surface root should stay structural and expose the native target helper"
    );
    assert!(
        runtime_app_source.contains("mod window_creation;"),
        "runtime entry app should keep primary winit window creation in a child module"
    );
    assert!(
        runtime_app_source.contains("mod window_events;"),
        "runtime entry app should keep concrete winit window-event dispatch in a child module"
    );
    assert!(
        !runtime_handler_source.contains("fn bind_window_surface"),
        "runtime winit event handling should not own native surface binding helper implementations"
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
    for native_surface_path in [
        "HasWindowHandle",
        "HasDisplayHandle",
        "RawWindowHandle::Win32",
        "RawDisplayHandle::Windows",
        "ZrRuntimeNativeSurfaceTargetV1::win32",
        "ZIRCON_RUNTIME_ABI_VERSION_V1",
    ] {
        assert!(
            runtime_window_surface_source.contains(native_surface_path),
            "runtime window-surface native target helper should preserve `{native_surface_path}`"
        );
    }
    assert!(
        !runtime_window_surface_source.contains("wgpu::"),
        "runtime window-surface target extraction should not create or configure render surfaces directly"
    );
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
    assert!(
        runtime_app_source.contains("mod frame_loop;"),
        "runtime entry app should keep frame-loop pumping in a child module"
    );
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
