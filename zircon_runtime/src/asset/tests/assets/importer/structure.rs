use super::*;

#[test]
fn importer_subtree_uses_ingest_namespace_without_service_shell() {
    let importer_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/asset/importer");
    let importer_mod = fs::read_to_string(importer_root.join("mod.rs")).unwrap_or_default();

    assert!(
        importer_mod.contains("mod ingest;"),
        "asset importer root should declare the ingest subtree directly"
    );
    assert!(
        !importer_mod.contains("mod service;"),
        "asset importer root should not keep a migration-smell service subtree"
    );
    assert!(
        importer_root.join("ingest").exists(),
        "asset importer ingest subtree should exist after the hard cutover"
    );
    assert!(
        !importer_root.join("service").exists(),
        "asset importer service subtree should be deleted after the hard cutover"
    );
}

#[test]
fn import_from_source_moves_the_owned_root_asset() {
    let source = include_str!("../../../importer/ingest/import_from_source.rs");

    assert!(source.contains(".map(|entry| entry.asset)"));
    assert!(!source.contains("entry.asset.clone()"));
}

#[test]
fn model_mesh_subassets_move_the_owned_root_model() {
    let source = include_str!("../../../importer/ingest/model_mesh_subassets.rs");

    assert!(source.contains("ImportedAsset::Model(model)"));
    assert!(!source.contains("ImportedAsset::Model(model.clone())"));
}

#[test]
fn material_import_moves_payload_after_collecting_dependencies() {
    let source = include_str!("../../../importer/ingest/import_material.rs");

    assert!(source.contains("ImportedAsset::Material(material)"));
    assert!(!source.contains("ImportedAsset::Material(material.clone())"));
}
