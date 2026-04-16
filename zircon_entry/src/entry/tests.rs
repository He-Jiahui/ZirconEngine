use zircon_editor::{EditorManager, EDITOR_MANAGER_NAME};
use zircon_manager::{
    resolve_asset_manager, resolve_config_manager, resolve_event_manager, resolve_input_manager,
    resolve_rendering_manager, ManagerResolver,
};
use zircon_render_server::resolve_render_server;
use zircon_scene::create_default_level;

use super::{BuiltinEntryModuleSet, EntryConfig, EntryProfile, EntryRunner};

#[test]
fn editor_bootstrap_registers_editor_and_primary_managers() {
    let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Editor)).unwrap();
    let asset_manager = resolve_asset_manager(&core).unwrap();
    let rendering_manager = resolve_rendering_manager(&core).unwrap();
    let input_manager = resolve_input_manager(&core).unwrap();
    let config_manager = resolve_config_manager(&core).unwrap();
    let event_manager = resolve_event_manager(&core).unwrap();
    let level = create_default_level(&core).unwrap();
    let _editor_manager = core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert!(asset_manager.pipeline_info().default_worker_count > 0);
    assert!(level.snapshot().nodes().len() >= 3);
    assert_eq!(rendering_manager.backend_info().backend_name, "wgpu");
    input_manager.submit_event(zircon_manager::InputEvent::ButtonPressed(
        zircon_manager::InputButton::MouseLeft,
    ));
    assert_eq!(
        input_manager.snapshot().pressed_buttons,
        vec![zircon_manager::InputButton::MouseLeft]
    );
    config_manager
        .set_value("editor.mode", serde_json::json!("docked"))
        .unwrap();
    assert_eq!(
        config_manager.get_value("editor.mode"),
        Some(serde_json::json!("docked"))
    );
    let receiver = event_manager.subscribe("editor.ready");
    event_manager.publish("editor.ready", serde_json::json!({ "booted": true }));
    assert_eq!(receiver.recv().unwrap().payload["booted"], true);
}

#[test]
fn runtime_bootstrap_excludes_editor_module() {
    let modules = BuiltinEntryModuleSet::for_profile(EntryProfile::Runtime);
    assert!(modules
        .descriptors()
        .iter()
        .all(|descriptor| descriptor.name != zircon_editor::EDITOR_MODULE_NAME));

    let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Runtime)).unwrap();
    assert!(resolve_render_server(&core).is_ok());
    assert!(core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .is_err());
    assert!(ManagerResolver::new(core).rendering().is_ok());
}

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
