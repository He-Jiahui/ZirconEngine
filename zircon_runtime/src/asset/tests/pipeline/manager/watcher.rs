use super::*;

#[test]
fn asset_manager_watcher_reports_changes_from_the_second_manifest_root() {
    let root = unique_temp_project_root("asset_manager_second_root_watch");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let game_assets = zircon_runtime_interface::project::RelPath::parse("game-assets").unwrap();
    let shared_assets = zircon_runtime_interface::project::RelPath::parse("shared-assets").unwrap();
    paths
        .ensure_layout(&[game_assets.clone(), shared_assets.clone()])
        .unwrap();
    let mut manifest = ProjectManifest::new(
        "DualRootWatcher",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.asset_roots = vec![game_assets.clone(), shared_assets.clone()];
    manifest.save(paths.manifest_path()).unwrap();

    let game_root = paths.asset_root(&game_assets);
    write_valid_wgsl(game_root.join("shaders/pbr.wgsl"));
    write_checker_png(game_root.join("textures/checker.png"));
    write_triangle_obj(game_root.join("models/triangle.obj"));
    let material_path = paths
        .asset_root(&shared_assets)
        .join("materials/grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(game_root.join("scenes/main.scene.toml"));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    let changes = manager.subscribe_asset_changes();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    while changes.recv_timeout(Duration::from_millis(50)).is_ok() {}

    let mut material = read_project_material(&material_path);
    material.base_color = [0.1, 0.4, 0.8, 1.0];
    write_project_material(&material_path, &material);

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut observed_uri = None;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match changes.recv_timeout(remaining.min(Duration::from_millis(150))) {
            Ok(change) if change.kind == AssetChangeKind::Modified => {
                if change.uri.to_string() == "res://materials/grid.zmaterial" {
                    observed_uri = Some(change.uri);
                    break;
                }
            }
            Ok(_) => {}
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    assert_eq!(
        observed_uri,
        Some(AssetUri::parse("res://materials/grid.zmaterial").unwrap()),
        "watcher must translate a second-root source event into its res:// URI",
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn asset_manager_watcher_reimports_modified_assets() {
    let root = unique_temp_project_root("asset_manager_watch");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("pbr.wgsl"),
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_triangle_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("triangle.obj"),
    );
    let material_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
    );

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    let changes = manager.subscribe_asset_changes();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    while changes.recv_timeout(Duration::from_millis(50)).is_ok() {}

    let mut material = read_project_material(&material_path);
    material.base_color = [0.2, 0.7, 0.9, 1.0];
    write_project_material(&material_path, &material);

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
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("pbr.wgsl"),
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_triangle_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("triangle.obj"),
    );
    let material_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
    );

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
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("shaders")
            .join("pbr.wgsl"),
    );
    write_checker_png(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("textures")
            .join("checker.png"),
    );
    write_triangle_obj(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("models")
            .join("triangle.obj"),
    );
    let material_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
    );

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

    let mut material = read_project_material(&material_path);
    material.base_color = [0.7, 0.3, 0.2, 1.0];
    write_project_material(&material_path, &material);

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

#[test]
fn project_manager_split_move_events_reconcile_sidecar_identity_as_rename() {
    let root = unique_temp_project_root("project_manager_added_move_identity");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let asset_root =
        paths.asset_root(&zircon_runtime_interface::project::RelPath::project_assets());
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let old_uri = AssetUri::parse("res://data/original.json").unwrap();
    let new_uri = AssetUri::parse("res://data/moved.json").unwrap();
    ProjectManifest::new("AddedMoveIdentity", new_uri.clone(), 1)
        .save(paths.manifest_path())
        .unwrap();

    let old_source = asset_root.join("data/original.json");
    fs::create_dir_all(old_source.parent().unwrap()).unwrap();
    fs::write(&old_source, r#"{ "moved": true }"#).unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    manager.scan_and_import().unwrap();
    let old_meta = old_source.with_file_name("original.json.zmeta");
    let original_uuid = AssetMetaDocument::load(&old_meta).unwrap().uuid;

    let new_source = old_source.with_file_name("moved.json");
    let new_meta = old_source.with_file_name("moved.json.zmeta");
    fs::rename(&old_source, &new_source).unwrap();
    fs::rename(&old_meta, &new_meta).unwrap();

    manager
        .scan_and_import_watch_changes(&[
            AssetChange::new(AssetChangeKind::Removed, old_uri.clone(), None),
            AssetChange::new(AssetChangeKind::Added, new_uri.clone(), None),
        ])
        .unwrap();

    assert!(manager.registry().get_by_locator(&old_uri).is_none());
    assert_eq!(
        manager
            .registry()
            .get_by_locator(&new_uri)
            .expect("moved resource record")
            .id(),
        crate::asset::AssetId::from_asset_uuid(original_uuid),
    );
    assert!(manager
        .asset_registry()
        .resolve_asset_id_by_path(&old_uri)
        .is_err());
    assert_eq!(
        manager.asset_registry().resolve_asset_id_by_path(&new_uri),
        Ok(crate::asset::AssetId::from_asset_uuid(original_uuid)),
    );
    assert_eq!(
        AssetMetaDocument::load(&new_meta).unwrap().uuid,
        original_uuid
    );

    let _ = fs::remove_dir_all(root);
}
