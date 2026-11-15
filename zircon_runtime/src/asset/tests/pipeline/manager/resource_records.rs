//! 项目导入产物要同时出现在资源状态接口与管理代际中；内建资源和项目资源共用该查询入口。

use super::*;

// Unreal Engine/Plugins/Tests/CQTest/Source/CQTestTests/Private/Helpers/
// CQTestAssetHelperTests.cpp: BEFORE_ALL registers an asset; FindAssets_WithFilter_ReturnsAssetArray
// queries the catalog and calls GetAsset. Reuse ZE's project fixture and real texture reader.
#[test]
fn unreal_equivalent_prepared_project_publishes_uuid_identity_to_runtime_reader() {
    use crate::asset::TextureAsset;
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
    use std::sync::Arc;

    let root = unique_temp_project_root("r05_unreal_equivalent_project_publication");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let assets = zircon_runtime_interface::project::RelPath::project_assets();
    paths.ensure_layout(&[assets.clone()]).unwrap();
    ProjectManifest::new(
        "R05 publication",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let uri = AssetUri::parse("res://textures/checker.png").unwrap();
    write_checker_png(paths.asset_root(&assets).join("textures/checker.png"));
    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    let runtime_reader = manager.resource_manager();
    let before = runtime_reader.projection_snapshot();
    assert!(runtime_reader.registry().get_by_locator(&uri).is_none());

    let project = ProjectManager::open(&root).unwrap();
    AssetManager::open_prepared_project(&manager, project).unwrap();

    let uuid = {
        let project = AssetManager::current_project_snapshot(&manager).unwrap();
        project.asset_registry().entry_by_path(&uri).unwrap().uuid()
    };
    let id = ResourceId::from_asset_uuid(uuid);
    assert_eq!(manager.resolve_asset_id(&uri), Some(id));
    let snapshot = manager.load_texture_asset_snapshot(id).unwrap();
    let runtime_payload = runtime_reader
        .get::<TextureMarker, TextureAsset>(ResourceHandle::new(id))
        .expect("the reader obtained before activation shares the published authority");
    assert!(Arc::ptr_eq(snapshot.resource(), &runtime_payload));
    assert!(!snapshot.rgba.is_empty());
    let published = runtime_reader.projection_snapshot();
    let by_id = published.management().row_by_id(id).unwrap();
    let by_path = published
        .management()
        .row_by_locator(&uri.to_string())
        .unwrap();
    assert!(Arc::ptr_eq(&by_id, &by_path));
    assert_eq!(by_id.kind, ResourceKind::Texture);
    assert_eq!(by_id.state, ResourceState::Ready);
    assert_eq!(by_id.revision, snapshot.revision());
    assert_eq!(
        runtime_reader.registry().get_by_locator(&uri).unwrap().id,
        id
    );
    assert!(published
        .readiness()
        .contains_kind(id, ResourceKind::Texture));
    assert!(before.management().row_by_id(id).is_none());

    drop(snapshot);
    drop(runtime_payload);
    manager.close_project().unwrap();
    drop(runtime_reader);
    drop(manager);
    fs::remove_dir_all(root).unwrap();
}

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
    assert!(status
        .artifact_locator
        .as_ref()
        .is_some_and(|uri| uri.to_string().starts_with("lib://")));
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
    assert!(mesh_status
        .artifact_locator
        .as_ref()
        .is_some_and(|uri| uri.to_string().starts_with("lib://")));
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
        crate::core::resource::ResourceManagementQuery::default(),
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
