use std::fs;

use zircon_asset::AssetUri;

use crate::components::NodeKind;
use crate::world::World;

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
fn render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay() {
    let root = unique_temp_project_root("scene_gizmo");
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
        .unwrap()
        .id;

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
    assert!(extract.overlays.selection.is_empty());
    assert!(
        extract
            .scene
            .meshes
            .iter()
            .any(|mesh| mesh.node_id == mesh_node)
    );

    let _ = fs::remove_dir_all(root);
}
