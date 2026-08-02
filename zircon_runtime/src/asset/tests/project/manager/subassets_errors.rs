use super::*;

#[test]
fn project_manager_imports_labeled_subassets_as_separate_artifacts() {
    let root = unique_temp_project_root("project_manager_multi_asset_labels");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "MultiAssetSandbox",
        AssetUri::parse("res://bundles/atlas.multi").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("atlas.multi");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "atlas").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.multi.bundle", "test.multi", AssetKind::Data, 1)
                .with_source_extensions(["multi"])
                .with_additional_output_kinds([AssetKind::Texture]),
            import_multi_asset_bundle,
        ))
        .unwrap();

    let imported = manager.scan_and_import().unwrap();

    let root_uri = AssetUri::parse("res://bundles/atlas.multi").unwrap();
    let texture_uri = AssetUri::parse("res://bundles/atlas.multi#Texture0").unwrap();
    let root_record = manager
        .registry()
        .get_by_locator(&root_uri)
        .expect("root record");
    let texture_record = manager
        .registry()
        .get_by_locator(&texture_uri)
        .expect("labeled texture record");
    let meta = AssetMetaDocument::load(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("bundles")
            .join("atlas.multi.zmeta"),
    )
    .unwrap();

    assert_eq!(imported.len(), 2);
    assert_eq!(meta.entries.len(), 2);
    let root_entry = meta
        .entries
        .iter()
        .find(|entry| entry.url == root_uri)
        .expect("root entry");
    let texture_entry = meta
        .entries
        .iter()
        .find(|entry| entry.url == texture_uri)
        .expect("texture entry");
    assert_eq!(root_entry.uuid, meta.uuid);
    assert_ne!(texture_entry.uuid, meta.uuid);
    assert_eq!(root_record.id(), AssetId::from_asset_uuid(root_entry.uuid));
    assert_eq!(
        texture_record.id(),
        AssetId::from_asset_uuid(texture_entry.uuid)
    );
    assert_ne!(
        root_record.artifact_locator(),
        texture_record.artifact_locator()
    );
    assert!(
        meta.entries
            .iter()
            .any(|entry| entry.url == texture_uri && entry.asset_kind == AssetKind::Texture)
    );

    match manager.load_artifact(&root_uri).unwrap() {
        ImportedAsset::Data(asset) => assert_eq!(asset.text, "atlas"),
        other => panic!("unexpected root artifact: {other:?}"),
    }
    match manager.load_artifact(&texture_uri).unwrap() {
        ImportedAsset::Texture(asset) => assert_eq!(asset.rgba, vec![255, 0, 255, 255]),
        other => panic!("unexpected subasset artifact: {other:?}"),
    }

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted.scan_and_import().unwrap();
    let restored_texture = restarted
        .registry()
        .get_by_locator(&texture_uri)
        .expect("restored labeled texture record");
    assert_eq!(restored_texture.id(), texture_record.id());
    assert_eq!(
        restarted.load_artifact(&texture_uri).unwrap(),
        manager.load_artifact(&texture_uri).unwrap()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_records_duplicate_imported_asset_label_as_failed_import() {
    let root = unique_temp_project_root("project_manager_duplicate_labels");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "DuplicateLabelSandbox",
        AssetUri::parse("res://bundles/duplicate.multi").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("duplicate.multi");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "duplicate").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.multi.duplicate", "test.multi", AssetKind::Data, 1)
                .with_source_extensions(["multi"])
                .with_additional_output_kinds([AssetKind::Texture]),
            import_duplicate_label_bundle,
        ))
        .unwrap();

    manager.scan_and_import().unwrap();

    let root_record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://bundles/duplicate.multi").unwrap())
        .expect("failed root record");
    assert_eq!(root_record.state, ResourceState::Error);
    assert!(root_record.artifact_locator().is_none());
    assert!(root_record.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("duplicate asset label Texture0")
    }));
    assert!(
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://bundles/duplicate.multi#Texture0").unwrap())
            .is_none()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_returns_structured_error_for_unknown_label_load() {
    let root = unique_temp_project_root("project_manager_unknown_label");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "UnknownLabelSandbox",
        AssetUri::parse("res://bundles/atlas.multi").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("atlas.multi");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "atlas").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.multi.bundle", "test.multi", AssetKind::Data, 1)
                .with_source_extensions(["multi"])
                .with_additional_output_kinds([AssetKind::Texture]),
            import_multi_asset_bundle,
        ))
        .unwrap();
    manager.scan_and_import().unwrap();

    let error = manager
        .load_artifact(&AssetUri::parse("res://bundles/atlas.multi#Missing").unwrap())
        .expect_err("missing label should be structured");

    match error {
        AssetImportError::MissingAssetLabel { source_uri, label } => {
            assert_eq!(
                source_uri,
                AssetUri::parse("res://bundles/atlas.multi").unwrap()
            );
            assert_eq!(label, "Missing");
        }
        other => panic!("unexpected missing-label error: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}
