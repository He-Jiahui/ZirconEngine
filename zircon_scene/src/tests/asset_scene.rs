use std::fs;

use zircon_asset::{AssetUri, ImportedAsset, ProjectManager};
use zircon_manager::LevelManager;

use crate::{DefaultLevelManager, NodeKind, World};

use super::support::{
    create_test_project, project_material_handle, project_model_handle, unique_temp_project_root,
};

#[test]
fn scene_assets_instantiate_world_with_asset_bound_meshes() {
    let root = unique_temp_project_root("scene_asset");
    let project = create_test_project(&root);
    let world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();

    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap();
    let mesh = mesh_node.mesh.as_ref().unwrap();
    assert_eq!(
        mesh.model,
        project_model_handle(&project, "res://models/triangle.obj")
    );
    assert_eq!(
        mesh.material,
        project_material_handle(&project, "res://materials/grid.material.toml")
    );

    let saved = world.to_scene_asset(&project).unwrap();
    let saved_mesh = saved
        .entities
        .iter()
        .find_map(|entity| entity.mesh.as_ref())
        .unwrap();
    assert_eq!(saved_mesh.model.to_string(), "res://models/triangle.obj");
    assert_eq!(
        saved_mesh.material.to_string(),
        "res://materials/grid.material.toml"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_extract_keeps_gizmo_overlay_for_asset_bound_meshes() {
    let root = unique_temp_project_root("scene_gizmo");
    let project = create_test_project(&root);
    let mut world = World::load_scene_from_uri(
        &project,
        &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
    )
    .unwrap();
    let mesh_node = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap()
        .id;
    world.set_selected(Some(mesh_node));

    let extract = world.to_render_extract();
    let mesh = extract
        .scene
        .meshes
        .iter()
        .find(|mesh| mesh.node_id == mesh_node)
        .unwrap();
    assert_eq!(
        mesh.model,
        project_model_handle(&project, "res://models/triangle.obj")
    );
    assert_eq!(
        mesh.material,
        project_material_handle(&project, "res://materials/grid.material.toml")
    );
    assert!(extract
        .overlays
        .selection
        .iter()
        .any(|highlight| highlight.owner == mesh_node && highlight.outline));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn level_manager_instantiates_and_saves_scene_assets() {
    let root = unique_temp_project_root("scene_manager");
    let project = create_test_project(&root);
    let manager = DefaultLevelManager::default();
    let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();

    let level = manager.load_level(&project, &scene_uri).unwrap();
    let summary = manager.level_summary(level.handle()).unwrap();
    assert!(summary.entity_count >= 2);

    let saved_uri = AssetUri::parse("res://scenes/saved.scene.toml").unwrap();
    manager
        .save_level(level.handle(), &project, &saved_uri)
        .unwrap();

    let mut reloaded_project = ProjectManager::open(&root).unwrap();
    reloaded_project.scan_and_import().unwrap();
    let ImportedAsset::Scene(scene) = reloaded_project.load_artifact(&saved_uri).unwrap() else {
        panic!("saved scene did not reimport as scene asset");
    };
    assert_eq!(scene.entities.len(), 3);

    let _ = fs::remove_dir_all(root);
}
