use crate::{
    package_manifest, plugin_registration, CAD_IMPORTER_CAPABILITY, MESH_IMPORTER_CAPABILITY,
    MODEL_ASSET_IMPORTER_DIST_CRATE_NAME, MODEL_ASSET_IMPORTER_DIST_RUNTIME_ENTRY, MODULE_NAME,
    PLUGIN_ID, RUNTIME_CAPABILITY,
};

#[test]
fn package_declares_model_importer_capabilities() {
    let manifest = package_manifest();

    assert_eq!(manifest.id, PLUGIN_ID);
    assert!(manifest
        .asset_importers
        .iter()
        .any(|importer| importer.source_extensions.contains(&"stl".to_string())));
    assert!(manifest
        .capabilities
        .contains(&RUNTIME_CAPABILITY.to_string()));
    assert!(manifest
        .capabilities
        .contains(&MESH_IMPORTER_CAPABILITY.to_string()));
    assert!(manifest
        .capabilities
        .contains(&CAD_IMPORTER_CAPABILITY.to_string()));
    assert_eq!(manifest.modules.len(), 1);
    assert_eq!(manifest.modules[0].crate_name, crate::RUNTIME_CRATE_NAME);
}

#[test]
fn model_asset_importer_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("model importer package exposes dist metadata");

    assert!(manifest
        .default_packaging
        .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(
        distribution.dist_crate,
        MODEL_ASSET_IMPORTER_DIST_CRATE_NAME
    );
    assert_eq!(
        distribution.runtime_entry,
        MODEL_ASSET_IMPORTER_DIST_RUNTIME_ENTRY
    );

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "asset_importer.model.dist")
        .expect("model importer package includes native dist module");
    assert_eq!(
        dist_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(dist_module.crate_name, MODEL_ASSET_IMPORTER_DIST_CRATE_NAME);
    assert!(dist_module
        .target_modes
        .contains(&zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime));
    assert!(dist_module
        .target_modes
        .contains(&zircon_runtime::builtin::RuntimeTargetMode::EditorHost));
    assert!(dist_module
        .capabilities
        .contains(&MESH_IMPORTER_CAPABILITY.to_string()));
}

#[test]
fn registration_contributes_stl_ply_and_dxf_importers() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == MODULE_NAME));
    assert_eq!(report.extensions.asset_importers().descriptors().len(), 5);
    assert_eq!(
        report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("mesh.stl"))
            .unwrap()
            .descriptor()
            .id
            .as_str(),
        "asset_importer.model.mesh"
    );
    assert_eq!(
        report
            .extensions
            .asset_importers()
            .select(std::path::Path::new("mesh.dxf"))
            .unwrap()
            .descriptor()
            .id
            .as_str(),
        "asset_importer.model.cad"
    );
}
