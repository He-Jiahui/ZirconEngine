use crate::{package_manifest, plugin_registration, MODULE_NAME, PLUGIN_ID, RUNTIME_CAPABILITY};

#[test]
fn package_declares_texture_importers() {
    let manifest = package_manifest();

    assert_eq!(manifest.id, PLUGIN_ID);
    assert!(manifest
        .capabilities
        .contains(&RUNTIME_CAPABILITY.to_string()));
    assert!(manifest
        .asset_importers
        .iter()
        .any(|importer| importer.source_extensions.contains(&"ktx2".to_string())));
}

#[test]
fn registration_contributes_module_and_importers() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == MODULE_NAME));
    assert_eq!(report.extensions.asset_importers().descriptors().len(), 4);
}
