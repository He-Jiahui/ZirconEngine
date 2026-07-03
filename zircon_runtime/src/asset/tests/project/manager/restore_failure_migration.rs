use super::*;

#[test]
fn project_manager_restores_ready_artifacts_from_meta_after_restart() {
    let root = unique_temp_project_root("project_manager_restart_restore");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://data/settings.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let data_path = paths.assets_root().join("data").join("settings.counted");
    if let Some(parent) = data_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&data_path, r#"{ "answer": 42 }"#).unwrap();

    COUNTED_IMPORT_CALLS.store(0, Ordering::SeqCst);
    let uri = AssetUri::parse("res://data/settings.counted").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    assert_eq!(COUNTED_IMPORT_CALLS.load(Ordering::SeqCst), 1);

    let record = manager.registry().get_by_locator(&uri).unwrap();
    let artifact_locator = record.artifact_locator().cloned().unwrap();
    assert_binary_library_artifact(paths.library_root(), &artifact_locator);
    let meta = AssetMetaDocument::load(
        paths
            .assets_root()
            .join("data")
            .join("settings.counted.zmeta"),
    )
    .unwrap();
    assert_eq!(
        meta.preview_state,
        crate::asset::project::PreviewState::Ready
    );
    assert_eq!(meta.artifact_locator.as_ref(), Some(&artifact_locator));
    assert_eq!(meta.importer_id, "test.counted.data");
    assert!(!meta.config_hash.is_empty());

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted.scan_and_import().unwrap();
    assert_eq!(
        COUNTED_IMPORT_CALLS.load(Ordering::SeqCst),
        1,
        "restart scan should restore the ready artifact without the custom importer"
    );

    let recovered = restarted.registry().get_by_locator(&uri).unwrap();
    assert_eq!(recovered.state, ResourceState::Ready);
    assert_eq!(recovered.importer_id, "test.counted.data");
    assert_eq!(recovered.artifact_locator(), Some(&artifact_locator));

    let imported = restarted.load_artifact(&uri).unwrap();
    match imported {
        ImportedAsset::Data(asset) => assert!(asset.text.contains("\"answer\"")),
        other => panic!("unexpected imported asset: {other:?}"),
    }
    assert_library_files_are_zassets(paths.library_root());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_reimports_material_when_ready_artifact_payload_is_stale() {
    let root = unique_temp_project_root("project_manager_stale_material_cache");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://materials/grid.zmaterial").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_default_material(paths.assets_root().join("materials").join("grid.zmaterial"));

    let uri = AssetUri::parse("res://materials/grid.zmaterial").unwrap();
    let mut manager = project_manager_with_first_wave_plugin_fixtures(&root);
    manager.scan_and_import().unwrap();
    let record = manager.registry().get_by_locator(&uri).unwrap();
    let artifact_locator = record.artifact_locator().cloned().unwrap();
    let artifact_path = paths.library_root().join(artifact_locator.path());

    let stale_enum_tag = 36_u32.to_le_bytes();
    let stale_payload = zstd::stream::encode_all(&stale_enum_tag[..], 1).unwrap();
    let mut payload = b"ZRARTZ01".to_vec();
    payload.extend_from_slice(&stale_payload);
    fs::write(&artifact_path, payload).unwrap();

    let mut restarted = project_manager_with_first_wave_plugin_fixtures(&root);
    restarted.scan_and_import().unwrap();

    let recovered = restarted.registry().get_by_locator(&uri).unwrap();
    assert_eq!(recovered.state, ResourceState::Ready);
    assert_eq!(recovered.artifact_locator(), Some(&artifact_locator));
    assert!(matches!(
        restarted.load_artifact(&uri).unwrap(),
        ImportedAsset::Material(_)
    ));

    let rewritten = fs::read(&artifact_path).unwrap();
    let decompressed = zstd::stream::decode_all(&rewritten[b"ZRARTZ01".len()..]).unwrap();
    assert_ne!(
        decompressed.get(..stale_enum_tag.len()),
        Some(&stale_enum_tag[..]),
        "stale cache payload should be replaced by a freshly imported material artifact"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_records_failed_imports_and_continues_scanning() {
    let root = unique_temp_project_root("project_manager_failed_import");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    fs::create_dir_all(paths.assets_root().join("models")).unwrap();
    fs::write(
        paths
            .assets_root()
            .join("models")
            .join("missing_backend.fbx"),
        b"fbx",
    )
    .unwrap();

    let mut manager = project_manager_with_first_wave_plugin_fixtures(&root);
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(imported.len(), 2);
    let shader = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://shaders/pbr.wgsl").unwrap())
        .expect("valid shader should still import after another file fails");
    assert_eq!(shader.state, ResourceState::Ready);
    assert!(shader.artifact_locator().is_some());

    let failed = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://models/missing_backend.fbx").unwrap())
        .expect("failed import should still have a registry record");
    assert_eq!(failed.kind, crate::asset::AssetKind::Model);
    assert_eq!(failed.state, ResourceState::Error);
    assert!(failed.artifact_locator().is_none());
    assert!(failed
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("backend is not installed")));

    let failed_meta = AssetMetaDocument::load(
        paths
            .assets_root()
            .join("models")
            .join("missing_backend.fbx.zmeta"),
    )
    .unwrap();
    assert_eq!(failed_meta.importer_id, "zircon.optional.model.fbx");
    assert_eq!(
        failed_meta.preview_state,
        crate::asset::project::PreviewState::Error
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_clears_stale_migration_meta_for_non_migrating_importer() {
    let root = unique_temp_project_root("project_manager_clear_stale_migration");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://data/settings.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let data_path = paths.assets_root().join("data").join("settings.json");
    if let Some(parent) = data_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&data_path, r#"{ "answer": 42 }"#).unwrap();

    let uri = AssetUri::parse("res://data/settings.json").unwrap();
    let mut stale_meta = AssetMetaDocument::new(AssetUuid::new(), uri, AssetKind::Data);
    stale_meta.source_schema_version = Some(1);
    stale_meta.target_schema_version = Some(99);
    stale_meta.migration_summary = "stale migration data".to_string();
    stale_meta
        .save(paths.assets_root().join("data").join("settings.json.zmeta"))
        .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let meta =
        AssetMetaDocument::load(paths.assets_root().join("data").join("settings.json.zmeta"))
            .unwrap();
    assert_eq!(meta.importer_id, "zircon.builtin.data.json");
    assert_eq!(meta.source_schema_version, None);
    assert_eq!(meta.target_schema_version, None);
    assert!(meta.migration_summary.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_records_import_dependency_ids_and_missing_dependency_diagnostics() {
    let root = unique_temp_project_root("project_manager_dependencies");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "DependencySandbox",
        AssetUri::parse("res://materials/grid.dep").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let material_path = paths.assets_root().join("materials").join("grid.dep");
    let texture_path = paths.assets_root().join("textures").join("checker.deptex");
    fs::create_dir_all(material_path.parent().unwrap()).unwrap();
    fs::create_dir_all(texture_path.parent().unwrap()).unwrap();
    fs::write(&material_path, "material").unwrap();
    fs::write(&texture_path, "texture").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.dep.material", "test.dep", AssetKind::Material, 1)
                .with_source_extensions(["dep"]),
            import_material_with_dependencies,
        ))
        .unwrap();
    manager
        .register_asset_importer(FunctionAssetImporter::new(
            AssetImporterDescriptor::new("test.dep.texture", "test.dep", AssetKind::Texture, 1)
                .with_source_extensions(["deptex"]),
            import_texture_dependency,
        ))
        .unwrap();

    manager.scan_and_import().unwrap();

    let texture = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://textures/checker.deptex").unwrap())
        .expect("texture record");
    let material = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://materials/grid.dep").unwrap())
        .expect("material record");

    assert_eq!(material.dependency_ids, vec![texture.id()]);
    assert!(material.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unresolved asset dependency res://textures/missing.deptex")
    }));

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted.scan_and_import().unwrap();
    let restarted_material = restarted
        .registry()
        .get_by_locator(&AssetUri::parse("res://materials/grid.dep").unwrap())
        .expect("restarted material record");
    assert_eq!(restarted_material.dependency_ids, vec![texture.id()]);
    assert!(restarted_material.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("unresolved asset dependency res://textures/missing.deptex")
    }));

    let _ = fs::remove_dir_all(root);
}
