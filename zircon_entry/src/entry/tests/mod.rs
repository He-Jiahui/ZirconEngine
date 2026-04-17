mod builtin_module_sets;
mod profile_bootstrap;

#[test]
fn runtime_sources_route_preview_through_render_server_without_wgpu_surface_bindings() {
    let lib_source = include_str!("../lib.rs");
    let production_lib_source = lib_source
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or(lib_source);
    let presenter_source = include_str!("../runtime_presenter.rs");
    let manifest = include_str!("../../Cargo.toml");

    assert!(
        presenter_source.contains("resolve_render_server"),
        "runtime presenter should resolve RenderServer from core"
    );
    assert!(
        presenter_source.contains("softbuffer"),
        "runtime presenter should blit RenderServer output through softbuffer"
    );
    assert!(
        presenter_source.contains("submit_frame_extract"),
        "runtime presenter should submit RenderFrameExtract through RenderServer"
    );
    assert!(
        presenter_source.contains("capture_frame"),
        "runtime presenter should read captured frames through RenderServer"
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
            "runtime entry source should not reference `{forbidden}` after RenderServer migration"
        );
        assert!(
            !presenter_source.contains(forbidden),
            "runtime presenter source should not reference `{forbidden}` after RenderServer migration"
        );
    }

    assert!(
        !manifest.contains("wgpu.workspace = true"),
        "zircon_entry/Cargo.toml should not depend on wgpu directly"
    );
}

#[test]
fn runtime_viewport_interaction_is_private_to_entry_camera_controller() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_construct_source = include_str!("runtime_entry_app/construct.rs");
    let runtime_handler_source = include_str!("runtime_entry_app/application_handler.rs");

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
    let runtime_handler_source = include_str!("runtime_entry_app/application_handler.rs");

    assert!(
        runtime_handler_source.contains("use zircon_input::{InputButton, InputEvent};"),
        "runtime window event handling should import input protocol types from zircon_input"
    );
    assert!(
        !runtime_handler_source.contains("use zircon_manager::{InputButton, InputEvent};"),
        "runtime window event handling should not import input protocol types from zircon_manager"
    );
}

#[test]
fn entry_subsystem_is_split_into_module_sets_run_modes_and_runtime_app_tree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry");

    for relative in [
        "entry_runner/mod.rs",
        "entry_runner/editor.rs",
        "entry_runner/runtime.rs",
        "entry_runner/headless.rs",
        "module_set/mod.rs",
        "module_set/builtin_registry.rs",
        "module_set/profile_sets.rs",
        "module_set/editor_modules.rs",
        "module_set/runtime_modules.rs",
        "runtime_entry_app/mod.rs",
        "tests/mod.rs",
        "tests/profile_bootstrap.rs",
        "tests/builtin_module_sets.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected entry module {relative} under {:?}",
            root
        );
    }
}
