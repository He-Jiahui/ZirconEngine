use super::*;

#[test]
fn asset_manager_acquire_release_unloads_and_rehydrates_runtime_resources() {
    let root = unique_temp_project_root("asset_manager_runtime_leases");
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

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let material_id = manager
        .resolve_asset_id(&AssetUri::parse("res://materials/grid.zmaterial").unwrap())
        .expect("material asset id");

    assert_eq!(
        manager.runtime_resource_state(material_id),
        Some(RuntimeResourceState::Loaded)
    );
    assert_eq!(manager.runtime_ref_count(material_id), Some(0));

    {
        let lease = manager.acquire_material_asset(material_id).unwrap();
        assert_eq!(lease.base_color, [0.8, 0.8, 0.8, 1.0]);
        assert_eq!(manager.runtime_ref_count(material_id), Some(1));
    }

    assert_eq!(manager.runtime_ref_count(material_id), Some(0));
    assert_eq!(
        manager.runtime_resource_state(material_id),
        Some(RuntimeResourceState::Unloaded)
    );

    let rehydrated = manager.acquire_material_asset(material_id).unwrap();
    assert_eq!(rehydrated.base_color, [0.8, 0.8, 0.8, 1.0]);
    assert_eq!(
        manager.runtime_resource_state(material_id),
        Some(RuntimeResourceState::Loaded)
    );

    drop(rehydrated);
    let _ = fs::remove_dir_all(root);
}
