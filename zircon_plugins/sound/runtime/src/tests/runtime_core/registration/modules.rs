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
            zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
        ]
    );
}
