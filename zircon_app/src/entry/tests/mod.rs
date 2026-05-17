mod builtin_engine_entry;
mod profile_bootstrap;

#[test]
fn runtime_sources_route_preview_through_dynamic_api_without_app_wgpu_dependency() {
    let lib_source = include_str!("../../lib.rs");
    let production_lib_source = lib_source
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or(lib_source);
    let presenter_source = include_str!("../../runtime_presenter.rs");
    let manifest = include_str!("../../../Cargo.toml");

    assert!(
        presenter_source.contains("softbuffer"),
        "runtime presenter should blit runtime-owned frame output through softbuffer"
    );
    assert!(
        runtime_handler_source_for_tests().contains("capture_frame"),
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
    let runtime_handler_source = include_str!("../runtime_entry_app/application_handler.rs");
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
        runtime_handler_source.contains("SoftbufferRuntimePresenter::new"),
        "runtime redraw path should still be able to create the softbuffer fallback presenter"
    );
    assert!(
        runtime_handler_source.contains("capture_frame"),
        "runtime redraw fallback should continue using capture_frame()"
    );
}

#[test]
fn runtime_entry_surface_present_switch_keeps_diagnostics_and_fallback_paths() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_handler_source = include_str!("../runtime_entry_app/application_handler.rs");

    assert!(
        runtime_app_source.contains("surface_present_failed: bool"),
        "runtime entry app should remember failed surface-present state"
    );
    assert!(
        runtime_handler_source
            .contains("self.surface_present_enabled && !self.surface_present_failed"),
        "redraw should skip native present after a surface-present failure"
    );
    assert_source_order(
        runtime_handler_source,
        &[
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
            runtime_handler_source.contains(bind_path),
            "runtime entry surface bind path should preserve `{bind_path}`"
        );
    }
    assert_source_order(
        runtime_handler_source,
        &[
            "WindowEvent::SurfaceResized(size)",
            "self.resize_viewport(viewport_size).is_err()",
            "self.surface_present_enabled && !self.surface_present_failed",
            "self.bind_current_window_surface()",
        ],
        "runtime surface resize should resize the runtime viewport before rebinding the active surface",
    );
    assert_source_order(
        runtime_handler_source,
        &[
            "WindowEvent::RedrawRequested",
            "self.surface_present_enabled && !self.surface_present_failed",
            ".present_viewport(self.viewport, self.viewport_size)",
            "self.fail_surface_present();",
            "self.ensure_fallback_presenter(event_loop)",
            ".capture_frame(self.viewport, self.viewport_size)",
        ],
        "runtime redraw should fall back to capture_frame plus softbuffer after native present failure in the same branch",
    );
    assert_source_order(
        runtime_handler_source,
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
            runtime_handler_source.contains(unbind_path),
            "runtime surface-present teardown should preserve `{unbind_path}`"
        );
    }
    for diagnostic in [
        "runtime_surface_present_enabled",
        "runtime_surface_present_fallback",
        "runtime_surface_present_failed",
    ] {
        assert!(
            runtime_handler_source.contains(diagnostic),
            "runtime surface-present diagnostic `{diagnostic}` should remain source-visible"
        );
    }
    for required_path in [
        "present_viewport",
        "capture_frame",
        "tick_frame",
        "SoftbufferRuntimePresenter::new",
        "about_to_wait",
        "request_redraw",
    ] {
        assert!(
            runtime_handler_source.contains(required_path),
            "runtime entry surface-present switch should preserve `{required_path}`"
        );
    }
}

#[test]
fn runtime_entry_ticks_dynamic_runtime_time_before_redraw_request() {
    let runtime_handler_source = include_str!("../runtime_entry_app/application_handler.rs");
    let runtime_session_source = include_str!("../runtime_library/runtime_session.rs");

    assert!(
        runtime_session_source.contains("pub(crate) fn tick_frame"),
        "runtime session should expose an optional dynamic runtime tick wrapper"
    );
    assert_source_order(
        runtime_handler_source,
        &[
            "fn about_to_wait",
            "self.session.tick_frame()",
            "self.apply_runtime_host_requests(event_loop)",
            "window.request_redraw();",
        ],
        "runtime entry should advance runtime time and apply host requests before requesting the next redraw",
    );
}

#[test]
fn runtime_runner_forwards_session_profile_to_dynamic_runtime() {
    let runtime_runner_source = include_str!("../entry_runner/runtime.rs");
    let runtime_session_args_source = include_str!("../entry_runner/runtime_session_args.rs");
    let runtime_session_source = include_str!("../runtime_library/runtime_session.rs");

    assert!(
        runtime_session_args_source.contains("--runtime-session-profile"),
        "runtime runner should expose an explicit dynamic session profile argument"
    );
    assert!(
        runtime_session_args_source.contains("\"dev\"")
            && runtime_session_args_source.contains("\"minimal\"")
            && runtime_session_args_source.contains("\"headless\""),
        "runtime session profile parser should accept the dynamic runtime's named profiles"
    );
    assert!(
        runtime_session_args_source.contains("RUNTIME_SESSION_STARTUP_HELP")
            && runtime_session_args_source.contains("ZIRCON_RUNTIME_LIBRARY")
            && runtime_session_args_source.contains("ZIRCON_LOG_FILTER")
            && runtime_session_args_source.contains("ZIRCON_LOG")
            && runtime_session_args_source.contains("RUST_LOG")
            && runtime_session_args_source.contains("ZIRCON_LOG_LEVEL"),
        "runtime session profile parser should expose startup help for profiles, logging, and runtime library override"
    );
    assert_source_order(
        runtime_runner_source,
        &[
            "parse_diagnostic_log_startup_args(args)?",
            "parse_runtime_session_startup_args",
            "if runtime_session_args.help_requested",
            "return Ok(());",
            "LoadedRuntime::load_default()",
            "RuntimeSession::create_with_profile(runtime, runtime_session_args.profile.as_bytes())",
        ],
        "runtime runner should parse logging first, allow help before dynamic loading, then pass the selected session profile to the dynamic runtime",
    );
    assert!(
        runtime_session_source.contains("profile: ZrByteSlice::from_static(profile)"),
        "runtime session creation should pass the selected profile bytes through ZrRuntimeSessionConfigV1"
    );
}

#[test]
fn runtime_viewport_interaction_is_owned_by_dynamic_runtime_session() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_construct_source = include_str!("../runtime_entry_app/construct.rs");
    let runtime_handler_source = include_str!("../runtime_entry_app/application_handler.rs");

    assert!(
        runtime_app_source.contains("RuntimeSession"),
        "runtime entry app should own a runtime session wrapper"
    );
    assert!(
        !runtime_app_source.contains("mod camera_controller;"),
        "runtime camera control should live in zircon_runtime dynamic session state"
    );
    assert!(
        runtime_construct_source.contains("ZrRuntimeEventV1::viewport_resized"),
        "runtime entry construction should forward viewport changes through ABI events"
    );
    assert!(
        !runtime_app_source.contains("zircon_graphics::ViewportController"),
        "runtime entry app should not depend on zircon_graphics::ViewportController"
    );
    assert!(
        !runtime_construct_source
            .contains("zircon_graphics::{ViewportController, ViewportInput, ViewportState}"),
        "runtime construction should not import graphics viewport interaction types"
    );
    assert!(
        !runtime_handler_source.contains("use zircon_graphics::ViewportInput;"),
        "runtime window event handling should not import graphics viewport input types"
    );
}

#[test]
fn runtime_input_protocol_crosses_through_runtime_interface_events() {
    let runtime_handler_source = include_str!("../runtime_entry_app/application_handler.rs");
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_gamepad_source = include_str!("../runtime_entry_app/gamepad.rs");

    assert!(
        runtime_handler_source.contains("ZrRuntimeEventV1"),
        "runtime window event handling should forward input through runtime interface events"
    );
    for required in [
        "WindowEvent::KeyboardInput",
        "ZrRuntimeEventV1::keyboard",
        "WindowEvent::Focused",
        "ZrRuntimeEventV1::lifecycle",
        "WindowEvent::CloseRequested",
        "WindowEvent::Destroyed",
        "WindowEvent::Moved",
        "WindowEvent::Occluded",
        "WindowEvent::ThemeChanged",
        "WindowEvent::ScaleFactorChanged",
        "ZrRuntimeEventV1::window_close_requested",
        "ZrRuntimeEventV1::window_destroyed",
        "ZrRuntimeEventV1::window_moved",
        "ZrRuntimeEventV1::window_occluded",
        "ZrRuntimeEventV1::window_theme_changed",
        "ZrRuntimeEventV1::window_backend_scale_factor_changed",
        "ZrRuntimeEventV1::window_scale_factor_changed",
        "WindowEvent::PointerEntered",
        "WindowEvent::PointerLeft",
        "ZrRuntimeEventV1::cursor_entered",
        "ZrRuntimeEventV1::cursor_left",
        "WindowEvent::DragEntered",
        "WindowEvent::DragDropped",
        "WindowEvent::DragLeft",
        "ZrRuntimeEventV1::file_hovered",
        "ZrRuntimeEventV1::file_dropped",
        "ZrRuntimeEventV1::file_drag_cancelled",
        "PointerSource::Touch",
        "PointerKind::Touch",
        "ZrRuntimeEventV1::touch",
        "DeviceEvent::PointerMotion",
        "ZrRuntimeEventV1::mouse_motion",
        "WindowEvent::MouseWheel",
        "MouseScrollDelta::LineDelta",
        "MouseScrollDelta::PixelDelta",
        "ZrRuntimeEventV1::mouse_wheel_delta",
        "ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1",
        "ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1",
        "WindowEvent::Ime",
        "Ime::Preedit",
        "Ime::DeleteSurrounding",
        "ZrRuntimeEventV1::ime_preedit",
        "ZrRuntimeEventV1::ime_commit",
        "ZrRuntimeEventV1::ime_delete_surrounding",
        "ZrRuntimeEventV1::ime_enabled",
        "ZrRuntimeEventV1::ime_disabled",
        "drain_host_requests",
        "ZrRuntimeHostRequestV1",
        "request_ime_update",
        "ImeRequest::Enable",
        "ImeRequest::Update",
        "ImeRequest::Disable",
        "ImeCapabilities::new",
        "ImeSurroundingText::new",
    ] {
        assert!(
            runtime_handler_source.contains(required),
            "runtime window event handling should preserve `{required}` translation"
        );
    }
    for required in [
        "mod gamepad;",
        "gamepads: Option<gilrs::Gilrs>",
        "self.poll_gamepads(event_loop)",
    ] {
        assert!(
            runtime_app_source.contains(required) || runtime_handler_source.contains(required),
            "runtime entry app should preserve gamepad host wiring `{required}`"
        );
    }
    for required in [
        "GilrsBuilder::new",
        "EventType::ButtonChanged",
        "EventType::AxisChanged",
        "ZrRuntimeEventV1::gamepad_connection_with_ids",
        "ZrRuntimeEventV1::gamepad_button",
        "ZrRuntimeEventV1::gamepad_axis",
    ] {
        assert!(
            runtime_gamepad_source.contains(required),
            "runtime gamepad host should preserve `{required}` translation"
        );
    }
    assert!(
        !runtime_handler_source.contains("zircon_runtime::input"),
        "runtime window event handling should not import runtime implementation input types"
    );
}

fn runtime_handler_source_for_tests() -> &'static str {
    include_str!("../runtime_entry_app/application_handler.rs")
}

fn assert_source_order(source: &str, needles: &[&str], message: &str) {
    let mut offset = 0;
    for needle in needles {
        let Some(index) = source[offset..].find(needle) else {
            panic!("{message}: missing `{needle}`");
        };
        offset += index + needle.len();
    }
}

#[test]
fn entry_subsystem_is_split_into_builtin_modules_run_modes_and_runtime_app_tree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry");

    for relative in [
        "builtin_modules.rs",
        "entry_runner/mod.rs",
        "entry_runner/editor.rs",
        "entry_runner/runtime.rs",
        "entry_runner/headless.rs",
        "runtime_entry_app/mod.rs",
        "runtime_library/mod.rs",
        "runtime_library/loaded_runtime.rs",
        "runtime_library/runtime_session.rs",
        "tests/mod.rs",
        "tests/profile_bootstrap.rs",
        "tests/builtin_engine_entry.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected entry module {relative} under {:?}",
            root
        );
    }
}

#[test]
fn entry_uses_runtime_owned_builtin_module_list_without_manual_graphics_insertion() {
    let builtin_modules_source = include_str!("../builtin_modules.rs");

    assert!(
        builtin_modules_source.contains("runtime_modules_for_target"),
        "entry bootstrap should source runtime modules through target-aware runtime loader"
    );
    for forbidden in [
        "use zircon_runtime::graphics::GraphicsModule;",
        "modules.insert(4, Arc::new(GraphicsModule));",
    ] {
        assert!(
            !builtin_modules_source.contains(forbidden),
            "entry builtin module bootstrap should stop keeping runtime-owned graphics registration detail `{forbidden}`"
        );
    }
}
