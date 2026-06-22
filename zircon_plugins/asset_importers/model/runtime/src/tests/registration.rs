use crate::{
    package_manifest, plugin_registration, CAD_IMPORTER_CAPABILITY, MESH_IMPORTER_CAPABILITY,
    MODULE_NAME, PLUGIN_ID, RUNTIME_CAPABILITY,
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
