use zircon_editor::{EditorManager, EDITOR_MANAGER_NAME};
use zircon_runtime::core::manager::{
    resolve_config_manager, resolve_event_manager, resolve_input_manager, resolve_render_framework,
    resolve_rendering_manager, ManagerResolver,
};
use zircon_runtime::{
    asset::pipeline::manager::resolve_asset_manager,
    input::{InputButton, InputEvent},
    scene::create_default_level,
};

use super::super::{BuiltinEngineEntry, EngineEntry, EntryConfig, EntryProfile, EntryRunner};

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
    input_manager.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    assert_eq!(
        input_manager.snapshot().pressed_buttons,
        vec![InputButton::MouseLeft]
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
    let entry = BuiltinEngineEntry::for_profile(EntryProfile::Runtime);
    assert!(entry
        .module_descriptors()
        .iter()
        .all(|descriptor| descriptor.name != zircon_editor::EDITOR_MODULE_NAME));

    let core = EntryRunner::bootstrap(EntryConfig::new(EntryProfile::Runtime)).unwrap();
    assert!(resolve_render_framework(&core).is_ok());
    assert!(core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .is_err());
    assert!(ManagerResolver::new(core).rendering().is_ok());
}
