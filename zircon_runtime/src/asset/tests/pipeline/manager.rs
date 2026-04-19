use crossbeam_channel::RecvTimeoutError;
use std::fs;
use std::time::{Duration, Instant};

use crate::core::framework::asset::ResourceManager;
use crate::core::resource::{ResourceEventKind, ResourceKind, ResourceState, RuntimeResourceState};

use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::tests::support::{
    write_checker_png, write_default_material, write_default_scene, write_triangle_obj,
    write_valid_wgsl,
};
use crate::asset::watch::AssetChangeKind;
use crate::asset::{AssetManager, AssetUri, MaterialAsset, ProjectAssetManager};

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
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = ProjectAssetManager::default();
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
        .resolve_asset_id(&AssetUri::parse("res://materials/grid.material.toml").unwrap())
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
    let material_path = paths
        .assets_root()
        .join("materials")
        .join("grid.material.toml");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = ProjectAssetManager::default();
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
                && change.uri.to_string() == "res://materials/grid.material.toml"
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
        .resolve_asset_id(&AssetUri::parse("res://materials/grid.material.toml").unwrap())
        .expect("material asset id");
    assert_eq!(
        manager.load_material_asset(material_id).unwrap().base_color,
        material.base_color
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn resource_server_reports_resource_records_for_project_assets() {
    let root = unique_temp_project_root("asset_manager_resource_status");
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
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = ProjectAssetManager::default();
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
    assert!(status.dependency_ids.is_empty());
    assert!(status.diagnostics.is_empty());
    assert_eq!(
        manager.resolve_resource_id("res://models/triangle.obj"),
        Some(status.id.to_string())
    );
    assert_eq!(
        manager.resource_revision("res://models/triangle.obj"),
        Some(status.revision)
    );

    let resources = manager.list_resources();
    assert!(
        resources
            .iter()
            .any(|record| record.primary_locator.to_string() == "builtin://shader/pbr.wgsl"),
        "builtin resources should be visible through ResourceManager"
    );
    assert!(
        resources
            .iter()
            .any(|record| record.primary_locator.to_string() == "res://models/triangle.obj"),
        "project resources should be visible through ResourceManager"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn resource_server_reimport_bumps_revision_and_publishes_updated_event() {
    let root = unique_temp_project_root("asset_manager_resource_revision");
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
    let material_path = paths
        .assets_root()
        .join("materials")
        .join("grid.material.toml");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = ProjectAssetManager::default();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let resource_changes = manager.subscribe_resource_changes();
    let baseline_revision = manager
        .resource_revision("res://materials/grid.material.toml")
        .expect("baseline revision");

    let mut material =
        MaterialAsset::from_toml_str(&fs::read_to_string(&material_path).unwrap()).unwrap();
    material.base_color = [0.6, 0.2, 0.9, 1.0];
    fs::write(&material_path, material.to_toml_string().unwrap()).unwrap();

    manager
        .import_asset("res://materials/grid.material.toml")
        .unwrap();

    let next_status = manager
        .resource_status("res://materials/grid.material.toml")
        .expect("material resource status");
    assert_eq!(next_status.state, ResourceState::Ready);
    assert!(next_status.revision > baseline_revision);

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut updated = None;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if let Ok(event) = resource_changes.recv_timeout(remaining.min(Duration::from_millis(250)))
        {
            if event.kind == ResourceEventKind::Updated
                && event.locator.as_ref().is_some_and(|locator| {
                    locator.to_string() == "res://materials/grid.material.toml"
                })
            {
                updated = Some(event);
                break;
            }
        }
    }

    let updated = updated.expect("updated resource event");
    assert_eq!(updated.revision, next_status.revision);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importing_one_asset_does_not_bump_unrelated_resource_revisions() {
    let root = unique_temp_project_root("asset_manager_unrelated_revision");
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
    let model_path = paths.assets_root().join("models").join("triangle.obj");
    write_triangle_obj(model_path);
    let material_path = paths
        .assets_root()
        .join("materials")
        .join("grid.material.toml");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = ProjectAssetManager::default();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();

    let baseline_material_revision = manager
        .resource_revision("res://materials/grid.material.toml")
        .expect("material revision");
    let baseline_model_revision = manager
        .resource_revision("res://models/triangle.obj")
        .expect("model revision");

    let mut material =
        MaterialAsset::from_toml_str(&fs::read_to_string(&material_path).unwrap()).unwrap();
    material.base_color = [0.1, 0.6, 0.8, 1.0];
    fs::write(&material_path, material.to_toml_string().unwrap()).unwrap();

    manager
        .import_asset("res://materials/grid.material.toml")
        .unwrap();

    assert!(
        manager
            .resource_revision("res://materials/grid.material.toml")
            .expect("updated material revision")
            > baseline_material_revision
    );
    assert_eq!(
        manager.resource_revision("res://models/triangle.obj"),
        Some(baseline_model_revision),
        "reimporting one asset must not bump unrelated resource revisions",
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
    let material_path = paths
        .assets_root()
        .join("materials")
        .join("grid.material.toml");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = ProjectAssetManager::default();
    let asset_changes = manager.subscribe_asset_changes();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    while asset_changes
        .recv_timeout(Duration::from_millis(50))
        .is_ok()
    {}

    let baseline_revision = manager
        .resource_revision("res://materials/grid.material.toml")
        .expect("baseline material revision");
    let meta_path = material_path.with_file_name("grid.material.toml.meta.toml");
    let meta_before = fs::read_to_string(&meta_path).unwrap();
    fs::write(&meta_path, meta_before).unwrap();

    let deadline = Instant::now() + Duration::from_millis(800);
    let mut saw_material_change = false;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match asset_changes.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(change) => {
                if change.uri.to_string() == "res://materials/grid.material.toml" {
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
        manager.resource_revision("res://materials/grid.material.toml"),
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
    let material_path = paths
        .assets_root()
        .join("materials")
        .join("grid.material.toml");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = ProjectAssetManager::default();
    let asset_changes = manager.subscribe_asset_changes();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    while asset_changes
        .recv_timeout(Duration::from_millis(50))
        .is_ok()
    {}

    let baseline_material_revision = manager
        .resource_revision("res://materials/grid.material.toml")
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
                    && change.uri.to_string() == "res://materials/grid.material.toml"
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
        manager.resource_revision("res://materials/grid.material.toml"),
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
fn asset_manager_acquire_release_unloads_and_rehydrates_runtime_resources() {
    let root = unique_temp_project_root("asset_manager_runtime_leases");
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
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = ProjectAssetManager::default();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let material_id = manager
        .resolve_asset_id(&AssetUri::parse("res://materials/grid.material.toml").unwrap())
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
