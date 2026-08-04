use super::*;

#[test]
fn project_manager_scans_assets_imports_artifact_cache_and_loads_artifacts() {
    let root = unique_temp_project_root("project_manager");
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
    write_default_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("materials")
            .join("grid.zmaterial"),
    );
    write_default_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
    );

    let mut manager = project_manager_with_first_wave_plugin_fixtures(&root);
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(manager.manifest().name, "Sandbox");
    assert!(imported.len() >= 5);
    assert!(
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://models/triangle.obj").unwrap())
            .is_some()
    );
    assert!(
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://materials/grid.zmaterial").unwrap())
            .is_some()
    );

    let model = manager
        .load_artifact(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .unwrap();
    match model {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    assert!(paths.asset_artifact_root().join("models").is_dir());
    assert!(paths.asset_artifact_root().join("materials").is_dir());

    let model_meta_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("models")
        .join("triangle.obj.zmeta");
    assert!(
        model_meta_path.exists(),
        "expected sidecar meta file to be generated"
    );
    let model_meta = AssetMetaDocument::load(&model_meta_path).unwrap();
    let model_record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://models/triangle.obj").unwrap())
        .unwrap();
    let material_record = manager
        .registry()
        .get_by_locator(&AssetUri::parse("res://materials/grid.zmaterial").unwrap())
        .unwrap();
    assert_eq!(
        model_meta.url,
        AssetUri::parse("res://models/triangle.obj").unwrap()
    );
    assert_eq!(model_meta.asset_kind, AssetKind::Model);
    assert_eq!(model_record.id(), AssetId::from_asset_uuid(model_meta.uuid));
    assert_binary_artifact_cache(
        paths.asset_artifact_root(),
        model_record.artifact_locator().unwrap(),
    );
    assert_binary_artifact_cache(
        paths.asset_artifact_root(),
        material_record.artifact_locator().unwrap(),
    );
    assert_artifact_cache_files_are_zassets(paths.asset_artifact_root());

    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn project_manager_scan_resolves_a_case_virtualized_root_before_generating_asset_uris() {
    let root = unique_temp_project_root("project_manager_scan_case_virtualized_root");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Case virtualized scan",
        AssetUri::parse("res://materials/grid.zmaterial").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let material_path = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials/grid.zmaterial");
    write_default_material(&material_path);

    let virtualized_root = root.to_string_lossy().to_ascii_uppercase();
    let mut manager =
        project_manager_with_first_wave_plugin_fixtures(std::path::Path::new(&virtualized_root));
    manager.scan_and_import().unwrap();

    let expected_uri = AssetUri::parse("res://materials/grid.zmaterial").unwrap();
    assert!(manager.registry().get_by_locator(&expected_uri).is_some());
    assert_eq!(
        AssetMetaDocument::load(material_path.with_file_name("grid.zmaterial.zmeta"))
            .unwrap()
            .url,
        expected_uri
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_scan_rejects_an_internal_symlink_or_reparse_before_import() {
    let root = unique_temp_project_root("project_manager_scan_internal_link");
    let outside = unique_temp_project_root("project_manager_scan_internal_link_outside");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Internal link scan",
        AssetUri::parse("res://data/inside.json").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let mut manager = ProjectManager::open(&root).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("escaped.json"), "{}\n").unwrap();
    let linked = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("linked");
    if !create_directory_link(&outside, &linked) {
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
        return;
    }

    let error = manager.scan_and_import().unwrap_err();
    assert!(matches!(
        error,
        AssetImportError::UnsafeProjectAssetLink { path } if path == linked
    ));

    let _ = fs::remove_file(linked);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
}

#[cfg(unix)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => true,
        Err(error)
            if error.kind() == std::io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(1314) =>
        {
            false
        }
        Err(error) => panic!("create directory reparse fixture failed: {error}"),
    }
}

#[test]
fn project_manager_imports_physics_and_animation_assets_into_runtime_artifact_cache() {
    let root = unique_temp_project_root("project_manager_physics_animation");
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

    write_default_scene(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("scenes")
            .join("main.scene.toml"),
    );
    write_default_physics_material(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("physics")
            .join("materials")
            .join("default.physics_material.toml"),
    );
    write_default_animation_skeleton(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.skeleton.zranim"),
    );
    write_default_animation_clip(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.clip.zranim"),
    );
    write_default_animation_sequence(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.sequence.zranim"),
    );
    write_default_animation_graph(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.graph.zranim"),
    );
    write_default_animation_state_machine(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("animation")
            .join("hero.state_machine.zranim"),
    );

    let mut manager = ProjectManager::open(&root).unwrap();
    let imported = manager.scan_and_import().unwrap();

    assert!(imported.len() >= 6);
    assert!(
        manager
            .registry()
            .get_by_locator(
                &AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap()
            )
            .is_some()
    );
    assert!(
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://animation/hero.sequence.zranim").unwrap())
            .is_some()
    );
    assert!(
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://animation/hero.graph.zranim").unwrap())
            .is_some()
    );
    assert!(
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://animation/hero.state_machine.zranim").unwrap())
            .is_some()
    );

    let physics_material = manager
        .load_artifact(
            &AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap(),
        )
        .unwrap();
    let sequence = manager
        .load_artifact(&AssetUri::parse("res://animation/hero.sequence.zranim").unwrap())
        .unwrap();

    assert_eq!(
        physics_material,
        ImportedAsset::PhysicsMaterial(sample_physics_material_asset())
    );
    assert_eq!(
        sequence,
        ImportedAsset::AnimationSequence(sample_animation_sequence_asset())
    );
    assert_binary_artifact_cache(
        paths.asset_artifact_root(),
        manager
            .registry()
            .get_by_locator(
                &AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap(),
            )
            .unwrap()
            .artifact_locator()
            .unwrap(),
    );
    assert_binary_artifact_cache(
        paths.asset_artifact_root(),
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://animation/hero.sequence.zranim").unwrap())
            .unwrap()
            .artifact_locator()
            .unwrap(),
    );

    assert!(
        paths
            .asset_artifact_root()
            .join("physics")
            .join("materials")
            .is_dir()
    );
    assert!(
        paths
            .asset_artifact_root()
            .join("animation")
            .join("skeletons")
            .is_dir()
    );
    assert!(
        paths
            .asset_artifact_root()
            .join("animation")
            .join("clips")
            .is_dir()
    );
    assert!(
        paths
            .asset_artifact_root()
            .join("animation")
            .join("sequences")
            .is_dir()
    );
    assert!(
        paths
            .asset_artifact_root()
            .join("animation")
            .join("graphs")
            .is_dir()
    );
    assert!(
        paths
            .asset_artifact_root()
            .join("animation")
            .join("state_machines")
            .is_dir()
    );

    let physics_meta = AssetMetaDocument::load(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("physics")
            .join("materials")
            .join("default.physics_material.toml.zmeta"),
    )
    .unwrap();
    assert_eq!(
        physics_meta.url,
        AssetUri::parse("res://physics/materials/default.physics_material.toml").unwrap()
    );
    assert_artifact_cache_files_are_zassets(paths.asset_artifact_root());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manager_imports_sound_assets_into_runtime_artifact_cache() {
    let root = unique_temp_project_root("project_manager_sound");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://audio/ping.wav").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_test_wav(
        paths
            .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
            .join("audio")
            .join("ping.wav"),
    );

    let mut manager = project_manager_with_first_wave_plugin_fixtures(&root);
    let imported = manager.scan_and_import().unwrap();

    assert_eq!(imported.len(), 1);
    assert!(
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://audio/ping.wav").unwrap())
            .is_some()
    );

    let sound = manager
        .load_artifact(&AssetUri::parse("res://audio/ping.wav").unwrap())
        .unwrap();

    assert_eq!(
        sound,
        ImportedAsset::Sound(sample_sound_asset("res://audio/ping.wav"))
    );
    assert!(paths.asset_artifact_root().join("sound").is_dir());
    assert_binary_artifact_cache(
        paths.asset_artifact_root(),
        manager
            .registry()
            .get_by_locator(&AssetUri::parse("res://audio/ping.wav").unwrap())
            .unwrap()
            .artifact_locator()
            .unwrap(),
    );
    assert_artifact_cache_files_are_zassets(paths.asset_artifact_root());

    let _ = fs::remove_dir_all(root);
}
