use super::*;

#[test]
fn asset_manager_opens_project_reports_assets_and_publishes_changes() {
    let root = unique_temp_project_root("asset_manager");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(paths.assets_root().join("materials").join("grid.zmaterial"));
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    let changes = manager.subscribe_asset_changes();
    let project = manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();

    assert_eq!(project.name, "Sandbox");
    assert_eq!(
        manager.current_project().unwrap().default_scene_uri,
        "res://scenes/main.scene.toml"
    );

    let status = manager
        .asset_status("res://models/triangle.obj")
        .expect("model status");
    assert!(status.imported);
    assert_eq!(status.kind, ResourceKind::Model);
    assert!(manager.list_assets().len() >= 5);

    let model_id = manager
        .resolve_asset_id(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .expect("model asset id");
    let material_id = manager
        .resolve_asset_id(&AssetUri::parse("res://materials/grid.zmaterial").unwrap())
        .expect("material asset id");
    assert_eq!(
        manager.load_model_asset(model_id).unwrap().primitives.len(),
        1
    );
    assert_eq!(
        manager
            .load_material_asset(material_id)
            .unwrap()
            .name
            .as_deref(),
        Some("Grid")
    );

    let change = changes.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(change.kind, AssetChangeKind::Added);
    assert!(change.uri.to_string().starts_with("res://"));

    let _ = fs::remove_dir_all(root);
}
