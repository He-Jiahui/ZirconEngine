use super::*;

#[test]
fn project_generation_lazy_residency_publishes_metadata_before_payload() {
    let root = unique_temp_project_root("project_generation_lazy_residency");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Lazy Residency",
        AssetUri::parse("res://data/startup.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("data")
        .join("startup.json");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, r#"{"startup":"lazy"}"#).unwrap();

    let manager = ProjectAssetManager::default();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let locator = AssetUri::parse("res://data/startup.json").unwrap();
    let id = manager
        .resolve_asset_id(&locator)
        .expect("published metadata");
    let resources = manager.resource_manager();

    assert_eq!(
        resources.registry().get(id).unwrap().state,
        ResourceState::Ready
    );
    assert!(resources.get_untyped(id).is_none());
    assert_eq!(
        resources.runtime_state(id),
        Some(RuntimeResourceState::Unloaded)
    );

    let loaded = manager.load_data_asset(id).unwrap();
    assert_eq!(loaded.text, r#"{"startup":"lazy"}"#);
    assert!(resources.get_untyped(id).is_some());
    assert_eq!(
        resources.runtime_state(id),
        Some(RuntimeResourceState::Loaded)
    );

    drop(manager);
    fs::remove_dir_all(root).unwrap();
}
