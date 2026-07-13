use super::super::super::*;

#[test]
fn sound_plugin_registration_contributes_runtime_module() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == SOUND_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
}

#[test]
fn sound_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("sound plugin declares standalone distribution metadata");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, SOUND_DIST_CRATE_NAME);
    assert_eq!(distribution.runtime_entry, SOUND_DIST_RUNTIME_ENTRY);

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "sound.dist")
        .expect("sound package manifest exposes native dist module");
    assert_eq!(
        dist_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(dist_module.crate_name, SOUND_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(dist_module.capabilities, RUNTIME_CAPABILITIES);
}
