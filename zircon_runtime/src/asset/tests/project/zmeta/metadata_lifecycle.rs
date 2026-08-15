use super::*;

#[test]
fn project_manager_writes_zmeta_schema_and_ignores_old_meta_toml_sidecars() {
    let root = unique_temp_project_root("project_manager_zmeta_schema");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "ZMetaSandbox",
        AssetUri::parse("res://data/settings.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let data_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data")
        .join("settings.json");
    fs::create_dir_all(data_path.parent().unwrap()).unwrap();
    fs::write(&data_path, r#"{ "answer": 42 }"#).unwrap();
    fs::write(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("data")
            .join("settings.json.meta.toml"),
        "legacy sidecar must stay ignored",
    )
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let uri = AssetUri::parse("res://data/settings.json").unwrap();
    let record = manager.registry().get_by_locator(&uri).unwrap();
    let meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data")
        .join("settings.json.zmeta");
    let meta = AssetMetaDocument::load(&meta_path).unwrap();

    assert!(meta_path.exists());
    assert_eq!(meta.format_version, 7);
    assert_eq!(meta.url, uri);
    assert_eq!(meta.asset_kind, AssetKind::Data);
    assert_eq!(meta.unit, AssetSourceUnit::Single);
    assert!(meta.included_files.is_empty());
    assert_eq!(meta.entries.len(), 1);
    assert_eq!(meta.entries[0].uuid, meta.uuid);
    assert_eq!(meta.entries[0].url, uri);
    assert_eq!(meta.entries[0].asset_kind, AssetKind::Data);
    assert_eq!(record.id(), AssetId::from_asset_uuid(meta.uuid));
    assert_eq!(record.state, ResourceState::Ready);
    assert!(manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://data/settings.json.meta.toml").unwrap())
        .is_none());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_watch_remove_commits_resource_and_asset_registries_together() {
    let root = unique_temp_project_root("project_manager_watch_remove");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://data/remove.json").unwrap();
    ProjectManifest::new("WatchRemove", uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    let source = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/remove.json");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "{}").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let meta_path = source.with_file_name("remove.json.zmeta");
    let uuid = AssetMetaDocument::load(&meta_path).unwrap().uuid;

    fs::remove_file(&source).unwrap();
    fs::remove_file(&meta_path).unwrap();
    manager
        .scan_and_import_watch_changes(&[crate::asset::watch::AssetChange::new(
            crate::asset::watch::AssetChangeKind::Removed,
            uri.clone(),
            None,
        )])
        .unwrap();

    assert!(manager.registry().get_by_locator(&uri).is_none());
    assert!(manager
        .asset_registry()
        .resolve_asset_id_by_path(&uri)
        .is_err());
    assert!(manager
        .asset_registry()
        .resolve_asset_id_by_uuid(uuid)
        .is_err());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_watch_rename_preserves_guid_and_replaces_both_registry_paths() {
    let root = unique_temp_project_root("project_manager_watch_rename");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let old_uri = AssetUri::parse("res://data/old.json").unwrap();
    let new_uri = AssetUri::parse("res://data/new.json").unwrap();
    ProjectManifest::new("WatchRename", new_uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    let old_source = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/old.json");
    fs::create_dir_all(old_source.parent().unwrap()).unwrap();
    fs::write(&old_source, "{}").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let old_meta = old_source.with_file_name("old.json.zmeta");
    let uuid = AssetMetaDocument::load(&old_meta).unwrap().uuid;
    let new_source = old_source.with_file_name("new.json");
    let new_meta = old_source.with_file_name("new.json.zmeta");
    fs::rename(&old_source, &new_source).unwrap();
    fs::rename(&old_meta, &new_meta).unwrap();

    manager
        .scan_and_import_watch_changes(&[crate::asset::watch::AssetChange::new(
            crate::asset::watch::AssetChangeKind::Renamed,
            new_uri.clone(),
            Some(old_uri.clone()),
        )])
        .unwrap();

    assert!(manager.registry().get_by_locator(&old_uri).is_none());
    let record = manager.registry().get_by_locator(&new_uri).unwrap();
    assert_eq!(record.id(), AssetId::from_asset_uuid(uuid));
    assert!(manager
        .asset_registry()
        .resolve_asset_id_by_path(&old_uri)
        .is_err());
    assert_eq!(
        manager.asset_registry().resolve_asset_id_by_path(&new_uri),
        Ok(AssetId::from_asset_uuid(uuid))
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_registry_commit_failure_keeps_both_live_registries_unchanged() {
    let root = unique_temp_project_root("project_manager_registry_commit_rollback");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    let uri = AssetUri::parse("res://data/rollback.json").unwrap();
    ProjectManifest::new("WatchRollback", uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();
    let source = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data/rollback.json");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "{}").unwrap();
    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let meta_path = source.with_file_name("rollback.json.zmeta");
    let before_asset_registry = manager.asset_registry().clone();

    fs::remove_file(&source).unwrap();
    fs::remove_file(&meta_path).unwrap();
    manager
        .scan_and_import_watch_changes_with_registry_fault(
            &[crate::asset::watch::AssetChange::new(
                crate::asset::watch::AssetChangeKind::Removed,
                uri.clone(),
                None,
            )],
            crate::core::resource::io::AtomicWriteFault::Replace,
        )
        .unwrap_err();

    assert!(manager.registry().get_by_locator(&uri).is_some());
    assert_eq!(manager.asset_registry(), &before_asset_registry);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_resolves_asset_references_by_uuid_before_stale_url() {
    let root = unique_temp_project_root("project_manager_reference_uuid_first");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "ReferenceSandbox",
        AssetUri::parse("res://data/renamed.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let data_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data")
        .join("renamed.json");
    fs::create_dir_all(data_path.parent().unwrap()).unwrap();
    fs::write(&data_path, r#"{ "renamed": true }"#).unwrap();
    let uuid = AssetUuid::new();
    let stale_url = AssetUri::parse("res://data/original.json").unwrap();
    AssetMetaDocument::new(uuid, stale_url.clone(), AssetKind::Data)
        .save(
            paths
                .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
                .join("data")
                .join("renamed.json.zmeta"),
        )
        .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();

    let current_url = AssetUri::parse("res://data/renamed.json").unwrap();
    let current_id = manager
        .asset_registry()
        .resolve_asset_id_by_path(&current_url)
        .unwrap();

    assert_eq!(
        manager.asset_registry().resolve_asset_id_by_uuid(uuid),
        Ok(current_id)
    );
    assert_eq!(
        manager
            .asset_registry()
            .resolve_asset_id_for_reference(uuid, &stale_url),
        Ok(current_id)
    );
    assert_eq!(
        manager
            .asset_registry()
            .stale_path_for_uuid(uuid, &stale_url),
        Some(&current_url)
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_restore_refreshes_zmeta_entry_urls_after_source_rename() {
    let root = unique_temp_project_root("project_manager_rename_restore_zmeta_urls");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "RenameRestoreSandbox",
        AssetUri::parse("res://bundles/renamed.multi").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let original_source = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("atlas.multi");
    let renamed_source = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("renamed.multi");
    fs::create_dir_all(original_source.parent().unwrap()).unwrap();
    fs::write(&original_source, "atlas").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(multi_asset_importer("multi"))
        .unwrap();
    manager.scan_and_import().unwrap();

    let original_root_uri = AssetUri::parse("res://bundles/atlas.multi").unwrap();
    let original_texture_uri = AssetUri::parse("res://bundles/atlas.multi#Texture0").unwrap();
    let original_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("atlas.multi.zmeta");
    let renamed_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("renamed.multi.zmeta");
    let original_meta = AssetMetaDocument::load(&original_meta_path).unwrap();
    let original_texture_uuid = original_meta
        .entries
        .iter()
        .find(|entry| entry.url == original_texture_uri)
        .expect("original texture entry")
        .uuid;
    let original_texture_id = manager
        .registry()
        .get_by_locator(&original_texture_uri)
        .expect("original texture record")
        .id();

    fs::rename(&original_source, &renamed_source).unwrap();
    fs::rename(&original_meta_path, &renamed_meta_path).unwrap();

    let mut restarted = ProjectManager::open(&root).unwrap();
    restarted.scan_and_import().unwrap();

    let renamed_root_uri = AssetUri::parse("res://bundles/renamed.multi").unwrap();
    let renamed_texture_uri = AssetUri::parse("res://bundles/renamed.multi#Texture0").unwrap();
    let restored_meta = AssetMetaDocument::load(&renamed_meta_path).unwrap();
    let restored_root = restarted
        .registry()
        .get_by_locator(&renamed_root_uri)
        .expect("restored root record should use renamed URL");
    let restored_texture = restarted
        .registry()
        .get_by_locator(&renamed_texture_uri)
        .expect("restored texture record should use renamed URL");

    assert!(restarted
        .registry()
        .get_by_locator(&original_root_uri)
        .is_none());
    assert!(restarted
        .registry()
        .get_by_locator(&original_texture_uri)
        .is_none());
    assert_eq!(restored_meta.url, renamed_root_uri);
    assert!(restored_meta
        .entries
        .iter()
        .any(|entry| entry.uuid == restored_meta.uuid && entry.url == renamed_root_uri));
    assert!(restored_meta
        .entries
        .iter()
        .any(|entry| entry.uuid == original_texture_uuid && entry.url == renamed_texture_uri));
    assert_eq!(restored_texture.id(), original_texture_id);
    assert_eq!(
        restored_root.id(),
        AssetId::from_asset_uuid(restored_meta.uuid)
    );
    assert_eq!(
        restarted
            .asset_registry()
            .resolve_asset_id_for_reference(original_texture_uuid, &original_texture_uri),
        Ok(restored_texture.id())
    );
    assert_eq!(
        restarted
            .asset_registry()
            .stale_path_for_uuid(original_texture_uuid, &original_texture_uri),
        Some(&renamed_texture_uri)
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_preserves_zmeta_subasset_uuids_across_failed_reimport() {
    let root = unique_temp_project_root("project_manager_failed_reimport_zmeta_uuid");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "FailedReimportSandbox",
        AssetUri::parse("res://bundles/atlas.flaky").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("atlas.flaky");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "atlas").unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager
        .register_asset_importer(flaky_multi_asset_importer())
        .unwrap();
    manager.scan_and_import().unwrap();

    let root_uri = AssetUri::parse("res://bundles/atlas.flaky").unwrap();
    let texture_uri = AssetUri::parse("res://bundles/atlas.flaky#Texture0").unwrap();
    let meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("bundles")
        .join("atlas.flaky.zmeta");
    let ready_meta = AssetMetaDocument::load(&meta_path).unwrap();
    let ready_texture_uuid = ready_meta
        .entries
        .iter()
        .find(|entry| entry.url == texture_uri)
        .expect("ready texture entry")
        .uuid;
    let ready_texture_id = manager
        .registry()
        .get_by_locator(&texture_uri)
        .expect("ready texture record")
        .id();

    fs::write(&source_path, "fail").unwrap();
    manager.scan_and_import().unwrap();

    let failed_meta = AssetMetaDocument::load(&meta_path).unwrap();
    assert_eq!(
        failed_meta.preview_state,
        crate::asset::project::PreviewState::Error
    );
    assert!(manager.registry().get_by_locator(&texture_uri).is_none());
    assert!(failed_meta
        .entries
        .iter()
        .any(|entry| entry.uuid == ready_texture_uuid && entry.url == texture_uri));

    fs::write(&source_path, "atlas-fixed").unwrap();
    manager.scan_and_import().unwrap();

    let recovered_meta = AssetMetaDocument::load(&meta_path).unwrap();
    let recovered_texture = manager
        .registry()
        .get_by_locator(&texture_uri)
        .expect("recovered texture record");
    assert_eq!(
        recovered_meta.preview_state,
        crate::asset::project::PreviewState::Ready
    );
    assert_eq!(
        recovered_meta
            .entries
            .iter()
            .find(|entry| entry.url == texture_uri)
            .expect("recovered texture entry")
            .uuid,
        ready_texture_uuid
    );
    assert_eq!(recovered_texture.id(), ready_texture_id);
    assert_eq!(
        manager
            .asset_registry()
            .resolve_asset_id_for_reference(ready_texture_uuid, &root_uri),
        Ok(ready_texture_id)
    );

    let _ = fs::remove_dir_all(root);
}
