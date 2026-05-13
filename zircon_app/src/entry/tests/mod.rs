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

    assert!(
        runtime_handler_source.contains("ZrRuntimeEventV1"),
        "runtime window event handling should forward input through runtime interface events"
    );
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
