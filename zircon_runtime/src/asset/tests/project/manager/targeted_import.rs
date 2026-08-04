use super::*;

#[test]
fn targeted_import_preserves_unrelated_generation_records_after_source_deletion() {
    let root = unique_temp_project_root("project_manager_targeted_import");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "TargetedImportSandbox",
        AssetUri::parse("res://data/target.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let assets = paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    let target_path = assets.join("data/target.counted");
    let unrelated_path = assets.join("data/unrelated.counted");
    fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    fs::write(&target_path, "target-v1").unwrap();
    fs::write(&unrelated_path, "unrelated-v1").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let target_uri = AssetUri::parse("res://data/target.counted").unwrap();
    let unrelated_uri = AssetUri::parse("res://data/unrelated.counted").unwrap();
    let unrelated_before = manager
        .registry()
        .get_by_locator(&unrelated_uri)
        .cloned()
        .unwrap();

    fs::write(&target_path, "target-v2").unwrap();
    fs::remove_file(&unrelated_path).unwrap();
    let imported = manager
        .import_targeted_source(&target_uri, &target_path)
        .unwrap();

    assert_eq!(imported.len(), 1);
    assert_eq!(
        manager.registry().get_by_locator(&unrelated_uri),
        Some(&unrelated_before)
    );
    match manager.load_artifact(&target_uri).unwrap() {
        ImportedAsset::Data(asset) => assert_eq!(asset.text, "target-v2"),
        other => panic!("unexpected targeted artifact: {other:?}"),
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn targeted_import_commit_failure_restores_disk_and_project_generation() {
    let root = unique_temp_project_root("project_manager_targeted_rollback");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "TargetedRollbackSandbox",
        AssetUri::parse("res://data/target.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let target_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/target.counted");
    fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    fs::write(&target_path, "target-v1").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let target_uri = AssetUri::parse("res://data/target.counted").unwrap();
    let record = manager
        .registry()
        .get_by_locator(&target_uri)
        .cloned()
        .unwrap();
    let artifact_path = paths
        .asset_artifact_root()
        .join(record.artifact_locator().unwrap().path());
    let meta_path = target_path.with_file_name("target.counted.zmeta");
    let registry_path = paths.registry_root().join("asset-registry.json");
    let artifact_before = fs::read(&artifact_path).unwrap();
    let meta_before = fs::read(&meta_path).unwrap();
    let persisted_registry_before = fs::read(&registry_path).unwrap();
    let mut resource_registry_before = manager.registry().values().cloned().collect::<Vec<_>>();
    resource_registry_before.sort_by_key(|record| record.id().to_string());
    let asset_registry_before = manager.asset_registry().clone();

    fs::write(&target_path, "target-v2").unwrap();
    manager
        .import_targeted_source_with_commit_failure(&target_uri, &target_path, 2)
        .unwrap_err();

    assert_eq!(fs::read(artifact_path).unwrap(), artifact_before);
    assert_eq!(fs::read(meta_path).unwrap(), meta_before);
    assert_eq!(fs::read(registry_path).unwrap(), persisted_registry_before);
    let mut resource_registry_after = manager.registry().values().cloned().collect::<Vec<_>>();
    resource_registry_after.sort_by_key(|record| record.id().to_string());
    assert_eq!(resource_registry_after, resource_registry_before);
    assert_eq!(manager.asset_registry(), &asset_registry_before);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn targeted_generation_prepare_defers_disk_publication_until_commit() {
    let root = unique_temp_project_root("project_manager_targeted_prepare");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "TargetedPrepareSandbox",
        AssetUri::parse("res://data/target.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let target_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/target.counted");
    fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    fs::write(&target_path, "target-v1").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let target_uri = AssetUri::parse("res://data/target.counted").unwrap();
    let record = manager
        .registry()
        .get_by_locator(&target_uri)
        .cloned()
        .unwrap();
    let artifact_path = paths
        .asset_artifact_root()
        .join(record.artifact_locator().unwrap().path());
    let meta_path = target_path.with_file_name("target.counted.zmeta");
    let registry_path = paths.registry_root().join("asset-registry.json");
    let artifact_before = fs::read(&artifact_path).unwrap();
    let meta_before = fs::read(&meta_path).unwrap();
    let registry_before = fs::read(&registry_path).unwrap();

    fs::write(&target_path, "target-v2").unwrap();
    let mut candidate = manager.clone();
    let prepared = candidate
        .prepare_targeted_generation(&target_uri, &target_path)
        .unwrap();

    assert_eq!(fs::read(&artifact_path).unwrap(), artifact_before);
    assert_eq!(fs::read(&meta_path).unwrap(), meta_before);
    assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
    assert_ne!(
        candidate
            .registry()
            .get_by_locator(&target_uri)
            .unwrap()
            .source_hash,
        record.source_hash,
        "preparation must produce a distinct candidate before its files are published",
    );

    drop(prepared);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn targeted_watch_removal_prepare_defers_registry_publication_until_commit() {
    let root = unique_temp_project_root("project_manager_targeted_watch_removal_prepare");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "TargetedWatchRemovalPrepareSandbox",
        AssetUri::parse("res://data/target.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let target_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/target.counted");
    fs::create_dir_all(target_path.parent().unwrap()).unwrap();
    fs::write(&target_path, "target-v1").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let target_uri = AssetUri::parse("res://data/target.counted").unwrap();
    let record = manager
        .registry()
        .get_by_locator(&target_uri)
        .cloned()
        .unwrap();
    let artifact_path = paths
        .asset_artifact_root()
        .join(record.artifact_locator().unwrap().path());
    let meta_path = target_path.with_file_name("target.counted.zmeta");
    let registry_path = paths.registry_root().join("asset-registry.json");
    let artifact_before = fs::read(&artifact_path).unwrap();
    let meta_before = fs::read(&meta_path).unwrap();
    let registry_before = fs::read(&registry_path).unwrap();

    let mut candidate = manager.clone();
    let (_, prepared) = candidate
        .prepare_targeted_watch_changes(&[crate::asset::watch::AssetChange::new(
            crate::asset::watch::AssetChangeKind::Removed,
            target_uri.clone(),
            None,
        )])
        .unwrap();

    assert!(candidate.registry().get_by_locator(&target_uri).is_none());
    assert_eq!(fs::read(&artifact_path).unwrap(), artifact_before);
    assert_eq!(fs::read(&meta_path).unwrap(), meta_before);
    assert_eq!(fs::read(&registry_path).unwrap(), registry_before);

    drop(prepared);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn targeted_compound_import_rejects_changed_member_topology() {
    let root = unique_temp_project_root("project_manager_targeted_compound_topology");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://bundles/source_bundle").unwrap();
    ProjectManifest::new("TargetedCompoundSandbox", uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    let assets = paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    let compound_root = assets.join("bundles/source_bundle");
    fs::create_dir_all(&compound_root).unwrap();
    fs::write(compound_root.join("first.json"), "{}").unwrap();
    let mut meta = AssetMetaDocument::new(AssetUuid::new(), uri.clone(), AssetKind::Data);
    meta.unit = crate::asset::project::AssetSourceUnit::Compound;
    meta.included_files = vec![AssetUri::parse("res://bundles/source_bundle/first.json").unwrap()];
    meta.save(assets.join("bundles/source_bundle.zmeta"))
        .unwrap();
    #[cfg(windows)]
    let manager_root = root.to_string_lossy().to_ascii_uppercase();
    #[cfg(windows)]
    let manager = ProjectManager::open(std::path::Path::new(&manager_root)).unwrap();
    #[cfg(not(windows))]
    let manager = ProjectManager::open(&root).unwrap();

    manager
        .validate_targeted_source_topology(&uri, &compound_root)
        .unwrap();
    fs::write(compound_root.join("second.json"), "{}").unwrap();
    let error = manager
        .validate_targeted_source_topology(&uri, &compound_root)
        .unwrap_err();

    assert!(matches!(
        error,
        AssetImportError::TargetedImportRequiresFullScan { uri: rejected, .. }
            if rejected == uri
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn targeted_shader_import_refreshes_import_path_consumers_without_a_full_scan() {
    let root = unique_temp_project_root("project_manager_targeted_shader_dependencies");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let provider_uri = AssetUri::parse("res://shaders/provider").unwrap();
    let consumer_uri = AssetUri::parse("res://shaders/consumer").unwrap();
    ProjectManifest::new("TargetedShaderSandbox", consumer_uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    let provider_path =
        write_targeted_include_shader(&paths, "provider", "targeted::provider", &[]);
    let consumer_path = write_targeted_include_shader(
        &paths,
        "consumer",
        "targeted::consumer",
        &["targeted::provider"],
    );
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let provider_id = manager
        .registry()
        .get_by_locator(&provider_uri)
        .unwrap()
        .id();
    let consumer_id = manager
        .registry()
        .get_by_locator(&consumer_uri)
        .unwrap()
        .id();
    assert!(
        manager
            .registry()
            .get(consumer_id)
            .unwrap()
            .dependency_ids
            .contains(&provider_id)
    );

    write_targeted_include_shader(&paths, "provider", "targeted::renamed_provider", &[]);
    manager
        .import_targeted_source(&provider_uri, &provider_path)
        .unwrap();

    assert!(
        !manager
            .registry()
            .get(consumer_id)
            .unwrap()
            .dependency_ids
            .contains(&provider_id)
    );

    write_targeted_include_shader(
        &paths,
        "consumer",
        "targeted::consumer",
        &["targeted::renamed_provider"],
    );
    manager
        .import_targeted_source(&consumer_uri, &consumer_path)
        .unwrap();

    assert!(
        manager
            .registry()
            .get(consumer_id)
            .unwrap()
            .dependency_ids
            .contains(&provider_id)
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn targeted_shader_import_reports_conflict_when_multiple_owners_already_exist() {
    let root = unique_temp_project_root("project_manager_targeted_shader_conflict");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let first_uri = AssetUri::parse("res://shaders/first").unwrap();
    ProjectManifest::new("TargetedShaderConflict", first_uri, 1)
        .save(paths.manifest_path())
        .unwrap();
    write_targeted_include_shader(&paths, "first", "targeted::shared", &[]);
    write_targeted_include_shader(&paths, "second", "targeted::shared", &[]);
    let third_uri = AssetUri::parse("res://shaders/third").unwrap();
    let third_path = write_targeted_include_shader(&paths, "third", "targeted::third", &[]);
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    write_targeted_include_shader(&paths, "third", "targeted::shared", &[]);

    let imported = manager
        .import_targeted_source(&third_uri, &third_path)
        .unwrap();

    assert!(imported[0].diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("import_path `targeted::shared` conflicts")
    }));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn targeted_shader_provider_resolves_a_consumer_indexed_as_unresolved() {
    let root = unique_temp_project_root("project_manager_targeted_shader_new_provider");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let consumer_uri = AssetUri::parse("res://shaders/consumer").unwrap();
    ProjectManifest::new("TargetedShaderNewProvider", consumer_uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    write_targeted_include_shader(
        &paths,
        "consumer",
        "targeted::consumer",
        &["targeted::missing_provider"],
    );
    let provider_uri = AssetUri::parse("res://shaders/provider").unwrap();
    let provider_path =
        write_targeted_include_shader(&paths, "provider", "targeted::other_provider", &[]);
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let consumer_id = manager
        .registry()
        .get_by_locator(&consumer_uri)
        .unwrap()
        .id();
    assert!(
        manager
            .registry()
            .get(consumer_id)
            .unwrap()
            .dependency_ids
            .is_empty()
    );
    write_targeted_include_shader(&paths, "provider", "targeted::missing_provider", &[]);

    manager
        .import_targeted_source(&provider_uri, &provider_path)
        .unwrap();

    let provider_id = manager
        .registry()
        .get_by_locator(&provider_uri)
        .unwrap()
        .id();
    assert!(
        manager
            .registry()
            .get(consumer_id)
            .unwrap()
            .dependency_ids
            .contains(&provider_id)
    );
    let _ = fs::remove_dir_all(root);
}

fn write_targeted_include_shader(
    paths: &ProjectPaths,
    name: &str,
    import_path: &str,
    imports: &[&str],
) -> std::path::PathBuf {
    let assets = paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    let shader_dir = assets.join("shaders").join(name);
    fs::create_dir_all(&shader_dir).unwrap();
    let meta_path = assets.join("shaders").join(format!("{name}.zmeta"));
    if !meta_path.exists() {
        let mut meta = AssetMetaDocument::new(
            AssetUuid::new(),
            AssetUri::parse(&format!("res://shaders/{name}")).unwrap(),
            AssetKind::Shader,
        );
        meta.unit = crate::asset::project::AssetSourceUnit::Compound;
        meta.save(&meta_path).unwrap();
    }
    let imports = imports
        .iter()
        .map(|source| format!("\n[[imports]]\nsource = \"{source}\"\n"))
        .collect::<String>();
    fs::write(
        shader_dir.join(format!("{name}.zshader")),
        format!(
            "kind = \"include\"\nversion = 2\nimport_path = \"{import_path}\"\nwgsl_files = [\"{name}.wgsl\"]\n{imports}"
        ),
    )
    .unwrap();
    fs::write(
        shader_dir.join(format!("{name}.wgsl")),
        format!("fn {name}_value() -> f32 {{ return 1.0; }}\n"),
    )
    .unwrap();
    shader_dir
}
