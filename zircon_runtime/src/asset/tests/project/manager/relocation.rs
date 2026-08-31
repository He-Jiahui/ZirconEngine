use super::*;

#[test]
fn project_source_relocation_moves_authoring_files_and_preserves_live_identity() {
    let root = unique_temp_project_root("project_manager_source_relocation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "SourceRelocationSandbox",
        AssetUri::parse("res://data/original.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/original.counted");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "relocation-source-v1").unwrap();

    let source = AssetUri::parse("res://data/original.counted").unwrap();
    let target = AssetUri::parse("res://moved/renamed.counted").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let source_uuid = manager
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    let source_record = manager.registry().get_by_locator(&source).unwrap().clone();
    let artifact_path = paths.asset_artifact_root().join(
        source_record
            .artifact_locator()
            .expect("source imports an artifact")
            .path(),
    );
    let artifact_before = fs::read(&artifact_path).unwrap();
    let source_meta_path = source_path.with_file_name("original.counted.zmeta");
    let target_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("moved/renamed.counted");
    let target_meta_path = target_path.with_file_name("renamed.counted.zmeta");

    let updated = manager
        .relocate_project_source_for_test(source_uuid, target.clone())
        .expect("project source relocation should commit");

    assert!(!source_path.exists());
    assert!(!source_meta_path.exists());
    assert_eq!(fs::read(&target_path).unwrap(), b"relocation-source-v1");
    assert_eq!(
        AssetMetaDocument::load(&target_meta_path).unwrap().url,
        target
    );
    assert_eq!(fs::read(&artifact_path).unwrap(), artifact_before);
    assert!(manager.registry().get_by_locator(&source).is_none());
    assert_eq!(
        manager.registry().get_by_locator(&target).unwrap().id(),
        source_record.id()
    );
    assert_eq!(
        manager
            .asset_registry()
            .entry_by_path(&target)
            .unwrap()
            .uuid(),
        source_uuid
    );
    assert!(updated
        .iter()
        .any(|record| record.primary_locator() == &target));
    assert_eq!(
        manager
            .catalog_input_generation()
            .record(source_record.id())
            .unwrap()
            .source_path(),
        target_path
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn interrupted_project_source_relocation_recovers_the_original_generation() {
    let root = unique_temp_project_root("project_manager_source_relocation_recovery");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "SourceRelocationRecoverySandbox",
        AssetUri::parse("res://data/original.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/original.counted");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "relocation-source-v1").unwrap();
    let source_meta_path = source_path.with_file_name("original.counted.zmeta");
    let source = AssetUri::parse("res://data/original.counted").unwrap();
    let target = AssetUri::parse("res://moved/renamed.counted").unwrap();
    let target_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("moved/renamed.counted");
    let target_meta_path = target_path.with_file_name("renamed.counted.zmeta");
    let registry_path = paths.registry_root().join("asset-registry.json");

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager.scan_and_import().unwrap();
    let source_uuid = manager
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    let source_before = fs::read(&source_path).unwrap();
    let source_meta_before = fs::read(&source_meta_path).unwrap();
    let registry_before = fs::read(&registry_path).unwrap();

    manager
        .relocate_project_source_with_source_retirement_interruption(source_uuid, target.clone())
        .expect_err("the injected interruption must leave recovery evidence");
    assert!(target_path.exists());
    assert!(!source_path.exists());
    drop(manager);

    let reopened = ProjectManager::open(&root)
        .expect("project recovery must accept and roll back a source relocation transaction");
    assert_eq!(fs::read(&source_path).unwrap(), source_before);
    assert_eq!(fs::read(&source_meta_path).unwrap(), source_meta_before);
    assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
    assert!(!target_path.exists());
    assert!(!target_meta_path.exists());
    assert!(reopened.registry().get_by_locator(&source).is_some());
    assert!(reopened.registry().get_by_locator(&target).is_none());

    let _ = fs::remove_dir_all(root);
}
