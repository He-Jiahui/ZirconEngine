use super::*;

#[test]
fn editor_host_minimal_contract_freezes_core_and_extension_capability_boundaries() {
    let contract = editor_host_minimal_contract();

    assert_eq!(
        contract.minimal_capability_ids(),
        vec![
            "editor.host.ui_shell",
            "editor.host.asset_core",
            "editor.host.scene_interaction",
            "editor.host.runtime_render_embed",
            "editor.host.plugin_management",
            "editor.host.capability_bridge",
        ]
    );
    assert!(contract.is_minimal("editor.host.ui_shell"));
    assert!(contract.is_extension_blacklisted("editor.extension.animation_authoring"));
    assert!(contract.is_extension_blacklisted("editor.extension.ui_asset_authoring"));
    assert!(contract
        .minimal_capability_ids()
        .iter()
        .all(|capability| !contract.is_extension_blacklisted(capability)));
}

#[test]
fn editor_manager_reports_minimal_host_capabilities_even_without_vm_bridge() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_minimal_host_no_vm");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let minimal = manager.minimal_host_report();
    assert!(minimal.missing_capabilities().is_empty());
    assert_eq!(
        minimal.loaded_capabilities(),
        editor_host_minimal_contract().minimal_capability_ids()
    );

    let bridge = manager.vm_extension_capability_report();
    assert!(bridge.loaded_capabilities().is_empty());
    assert!(bridge
        .diagnostics()
        .iter()
        .any(|message| message.contains("ScriptModule.Driver.PluginHostDriver")));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}
