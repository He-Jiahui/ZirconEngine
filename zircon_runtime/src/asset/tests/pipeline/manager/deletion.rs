use super::*;

#[test]
fn project_source_deletion_retires_files_and_publishes_one_removed_generation() {
    let root = unique_temp_project_root("asset_manager_project_source_deletion");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "DeletionSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let assets_root =
        paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    write_valid_wgsl(assets_root.join("shaders/pbr.wgsl"));
    write_checker_png(assets_root.join("textures/checker.png"));
    write_triangle_obj(assets_root.join("models/triangle.obj"));
    write_default_material(assets_root.join("materials/grid.zmaterial"));
    write_default_scene(assets_root.join("scenes/main.scene.toml"));
    let deleted_source_path = assets_root.join("textures/orphan.png");
    let deleted_meta_path = assets_root.join("textures/orphan.png.zmeta");
    write_checker_png(&deleted_source_path);

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let deleted_meta = AssetMetaDocument::load(&deleted_meta_path).unwrap();
    let deleted_uri = AssetUri::parse("res://textures/orphan.png").unwrap();
    let asset_changes = manager.subscribe_asset_changes();
    let resource_changes = manager.subscribe_resource_changes();

    let removed = manager
        .delete_project_source(deleted_meta.uuid)
        .expect("unreferenced project source deletion should commit");

    assert!(
        removed
            .iter()
            .any(|status| status.uri == deleted_uri.as_str())
    );
    assert!(!deleted_source_path.exists());
    assert!(!deleted_meta_path.exists());
    assert!(manager.asset_status(deleted_uri.as_str()).is_none());
    assert!(manager.resource_status(deleted_uri.as_str()).is_none());
    assert!(
        !manager
            .list_assets()
            .iter()
            .any(|status| status.uri.starts_with(deleted_uri.as_str()))
    );
    assert!(
        !manager
            .resource_management_generation()
            .page(
                crate::core::resource::ResourceManagementQuery::default(),
                0,
                usize::MAX,
            )
            .rows
            .iter()
            .any(|record| record
                .primary_locator
                .as_ref()
                .starts_with(deleted_uri.as_str()))
    );

    let asset_change = asset_changes.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(asset_change.kind, AssetChangeKind::Removed);
    assert_eq!(asset_change.uri, deleted_uri);
    let resource_change = resource_changes
        .recv_timeout(Duration::from_secs(1))
        .unwrap();
    assert_eq!(resource_change.kind, ResourceEventKind::Removed);
    assert_eq!(
        resource_change.locator.as_ref().map(ToString::to_string),
        Some("res://textures/orphan.png".to_string())
    );

    let persisted = ProjectManager::open(&root).unwrap();
    assert!(
        persisted
            .asset_registry()
            .entry_by_path(&deleted_uri)
            .is_none()
    );
    assert!(persisted.registry().get_by_locator(&deleted_uri).is_none());

    let _ = fs::remove_dir_all(root);
}
