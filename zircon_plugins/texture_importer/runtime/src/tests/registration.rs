use crate::{
    package_manifest, plugin_registration, MODULE_NAME, PLUGIN_ID, RUNTIME_CAPABILITY,
    TEXTURE_IMPORTER_DIST_CRATE_NAME, TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY,
};

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
fn package_manifest_declares_texture_importer_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));
    let distribution = manifest.distribution.as_ref().expect("dist metadata");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, TEXTURE_IMPORTER_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(
        distribution.runtime_entry,
        TEXTURE_IMPORTER_DIST_RUNTIME_ENTRY
    );
    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "texture_importer.dist")
        .expect("dist native module");
    assert_eq!(dist_module.crate_name, TEXTURE_IMPORTER_DIST_CRATE_NAME);
    assert_eq!(
        dist_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert!(manifest
        .asset_importers
        .iter()
        .any(|importer| importer.id == "texture_importer.container"));
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
