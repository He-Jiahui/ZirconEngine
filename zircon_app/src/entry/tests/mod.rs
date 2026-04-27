mod builtin_engine_entry;
mod profile_bootstrap;

#[test]
fn runtime_sources_route_preview_through_render_framework_without_wgpu_surface_bindings() {
    let lib_source = include_str!("../../lib.rs");
    let production_lib_source = lib_source
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or(lib_source);
    let presenter_source = include_str!("../../runtime_presenter.rs");
    let manifest = include_str!("../../../Cargo.toml");

    assert!(
        presenter_source.contains("resolve_render_framework"),
        "runtime presenter should resolve RenderFramework from core"
    );
    assert!(
        presenter_source.contains("softbuffer"),
        "runtime presenter should blit RenderFramework output through softbuffer"
    );
    assert!(
        presenter_source.contains("submit_frame_extract"),
        "runtime presenter should submit RenderFrameExtract through RenderFramework"
    );
    assert!(
        presenter_source.contains("capture_frame"),
        "runtime presenter should read captured frames through RenderFramework"
    );

    for forbidden in [
        "wgpu::",
        "RuntimePreviewRenderer",
        "create_runtime_preview_renderer",
        "SharedTextureRenderService",
        "RenderService",
    ] {
        assert!(
            !production_lib_source.contains(forbidden),
            "runtime entry source should not reference `{forbidden}` after RenderFramework migration"
        );
        assert!(
            !presenter_source.contains(forbidden),
            "runtime presenter source should not reference `{forbidden}` after RenderFramework migration"
        );
    }

    assert!(
        !manifest.contains("wgpu.workspace = true"),
        "zircon_app/Cargo.toml should not depend on wgpu directly"
    );
}

#[test]
fn runtime_viewport_interaction_is_private_to_entry_camera_controller() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_construct_source = include_str!("../runtime_entry_app/construct.rs");
    let runtime_handler_source = include_str!("../runtime_entry_app/application_handler.rs");

    assert!(
        runtime_app_source.contains("mod camera_controller;"),
        "runtime entry app should own a private camera controller module"
    );
    assert!(
        runtime_construct_source.contains("RuntimeCameraController"),
        "runtime entry construction should use the private runtime camera controller"
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
fn runtime_input_protocol_is_owned_by_input_subsystem() {
    let runtime_handler_source = include_str!("../runtime_entry_app/application_handler.rs");

    assert!(
        runtime_handler_source.contains("use zircon_runtime::input::{InputButton, InputEvent};"),
        "runtime window event handling should import input protocol types through zircon_runtime"
    );
    assert!(
        !runtime_handler_source
            .contains("use zircon_runtime::core::manager::{InputButton, InputEvent};"),
        "runtime window event handling should not import input protocol types from zircon_manager"
    );
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
