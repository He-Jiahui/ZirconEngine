use super::*;

#[test]
fn asset_manager_watcher_reimports_modified_assets() {
    let root = unique_temp_project_root("asset_manager_watch");
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
    let material_path = paths.assets_root().join("materials").join("grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    let changes = manager.subscribe_asset_changes();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    while changes.recv_timeout(Duration::from_millis(50)).is_ok() {}

    let mut material =
        MaterialAsset::from_toml_str(&fs::read_to_string(&material_path).unwrap()).unwrap();
    material.base_color = [0.2, 0.7, 0.9, 1.0];
    fs::write(&material_path, material.to_toml_string().unwrap()).unwrap();

    let mut modified = None;
    for _ in 0..10 {
        if let Ok(change) = changes.recv_timeout(Duration::from_secs(1)) {
            if change.kind == AssetChangeKind::Modified
                && change.uri.to_string() == "res://materials/grid.zmaterial"
            {
                modified = Some(change);
                break;
            }
        }
    }

    assert!(
        modified.is_some(),
        "watcher did not report material modification"
    );
    let material_id = manager
        .resolve_asset_id(&AssetUri::parse("res://materials/grid.zmaterial").unwrap())
        .expect("material asset id");
    assert_eq!(
        manager.load_material_asset(material_id).unwrap().base_color,
        material.base_color
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn watcher_ignores_meta_sidecar_updates_for_revision_tracking() {
    let root = unique_temp_project_root("asset_manager_meta_sidecar");
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
    let material_path = paths.assets_root().join("materials").join("grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    let asset_changes = manager.subscribe_asset_changes();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    while asset_changes
        .recv_timeout(Duration::from_millis(50))
        .is_ok()
    {}

    let baseline_revision = manager
        .resource_revision("res://materials/grid.zmaterial")
        .expect("baseline material revision");
    let meta_path = material_path.with_file_name("grid.zmaterial.zmeta");
    let meta_before = fs::read_to_string(&meta_path).unwrap();
    fs::write(&meta_path, meta_before).unwrap();

    let deadline = Instant::now() + Duration::from_millis(800);
    let mut saw_material_change = false;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match asset_changes.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(change) => {
                if change.uri.to_string() == "res://materials/grid.zmaterial" {
                    saw_material_change = true;
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    assert!(
        !saw_material_change,
        "sidecar-only updates must not emit asset changes for the source asset"
    );
    assert_eq!(
        manager.resource_revision("res://materials/grid.zmaterial"),
        Some(baseline_revision),
        "sidecar-only updates must not bump resource revisions",
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn watcher_reimports_modified_asset_once_without_revision_loop() {
    let root = unique_temp_project_root("asset_manager_single_watch_reimport");
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
    let material_path = paths.assets_root().join("materials").join("grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    let asset_changes = manager.subscribe_asset_changes();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    while asset_changes
        .recv_timeout(Duration::from_millis(50))
        .is_ok()
    {}

    let baseline_material_revision = manager
        .resource_revision("res://materials/grid.zmaterial")
        .expect("baseline material revision");
    let baseline_model_revision = manager
        .resource_revision("res://models/triangle.obj")
        .expect("baseline model revision");

    let mut material =
        MaterialAsset::from_toml_str(&fs::read_to_string(&material_path).unwrap()).unwrap();
    material.base_color = [0.7, 0.3, 0.2, 1.0];
    fs::write(&material_path, material.to_toml_string().unwrap()).unwrap();

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut material_changes = 0;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match asset_changes.recv_timeout(remaining.min(Duration::from_millis(150))) {
            Ok(change) => {
                if change.kind == AssetChangeKind::Modified
                    && change.uri.to_string() == "res://materials/grid.zmaterial"
                {
                    material_changes += 1;
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    assert_eq!(
        material_changes, 1,
        "one source edit should produce one material change notification",
    );
    assert_eq!(
        manager.resource_revision("res://materials/grid.zmaterial"),
        Some(baseline_material_revision + 1),
        "one source edit should bump the changed asset revision once",
    );
    assert_eq!(
        manager.resource_revision("res://models/triangle.obj"),
        Some(baseline_model_revision),
        "watcher reimport should not bump unrelated resource revisions",
    );

    let _ = fs::remove_dir_all(root);
}
