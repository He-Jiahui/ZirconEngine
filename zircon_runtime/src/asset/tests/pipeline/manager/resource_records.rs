use super::*;

#[test]
fn resource_server_reports_resource_records_for_project_assets() {
    let root = unique_temp_project_root("asset_manager_resource_status");
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

    let status = manager
        .resource_status("res://models/triangle.obj")
        .expect("model resource status");
    assert_eq!(status.kind, ResourceKind::Model);
    assert_eq!(status.state, ResourceState::Ready);
    assert_eq!(status.revision, 1);
    assert!(
        status
            .artifact_locator
            .as_ref()
            .is_some_and(|uri| uri.to_string().starts_with("lib://"))
    );
    assert!(status.diagnostics.is_empty());

    let mesh_status = manager
        .resource_status("res://models/triangle.obj#Mesh0/Primitive0")
        .expect("model mesh subasset resource status");
    assert_eq!(status.dependency_ids, vec![mesh_status.id]);
    assert_eq!(mesh_status.kind, ResourceKind::Mesh);
    assert_eq!(mesh_status.state, ResourceState::Ready);
    assert_eq!(
        mesh_status.primary_locator.to_string(),
        "res://models/triangle.obj#Mesh0/Primitive0"
    );
    assert!(
        mesh_status
            .artifact_locator
            .as_ref()
            .is_some_and(|uri| uri.to_string().starts_with("lib://"))
    );
    assert!(mesh_status.dependency_ids.is_empty());
    assert!(mesh_status.diagnostics.is_empty());
    assert_eq!(
        manager.resolve_resource_id("res://models/triangle.obj"),
        Some(status.id.to_string())
    );
    assert_eq!(
        manager.resource_revision("res://models/triangle.obj"),
        Some(status.revision)
    );

    let resources = manager.resource_management_generation().page(
        crate::core::framework::asset::ResourceManagementQuery::default(),
        0,
        usize::MAX,
    );
    assert!(
        resources
            .rows
            .iter()
            .any(|record| record.primary_locator.as_ref() == "builtin://shader/pbr.wgsl"),
        "builtin resources should be visible through ResourceManager"
    );
    assert!(
        resources
            .rows
            .iter()
            .any(|record| record.primary_locator.as_ref() == "res://models/triangle.obj"),
        "project resources should be visible through ResourceManager"
    );

    let _ = fs::remove_dir_all(root);
}
