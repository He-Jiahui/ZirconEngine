use std::fs;

use crate::asset::AssetUri;
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};
use crate::core::math::Vec3;
use crate::scene::components::{ColliderShape, JointKind, RigidBodyType};

use crate::scene::components::NodeKind;
use crate::scene::world::World;

use super::support::{
    create_test_project, project_animation_clip_handle, project_animation_graph_handle,
    project_animation_sequence_handle, project_animation_skeleton_handle,
    project_animation_state_machine_handle, project_material_handle, project_model_handle,
    project_physics_material_handle, unique_temp_project_root,
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
    assert!(extract
        .scene
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == mesh_node));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn scene_assets_roundtrip_asset_bound_physics_and_animation_components() {
    let root = unique_temp_project_root("scene_physics_animation");
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

    let rigid_body = world.rigid_body(mesh_node).unwrap();
    assert_eq!(rigid_body.body_type, RigidBodyType::Dynamic);
    assert_eq!(rigid_body.mass, 2.5);

    let collider = world.collider(mesh_node).unwrap();
    assert_eq!(
        collider.shape,
        ColliderShape::Box {
            half_extents: Vec3::new(0.5, 0.5, 0.5),
        }
    );
    assert_eq!(
        collider.material,
        Some(project_physics_material_handle(
            &project,
            "res://physics/default.physics_material.toml"
        ))
    );
    assert_eq!(
        collider.material_override,
        Some(PhysicsMaterialMetadata {
            static_friction: 0.7,
            dynamic_friction: 0.4,
            restitution: 0.2,
            friction_combine: PhysicsCombineRule::Maximum,
            restitution_combine: PhysicsCombineRule::Average,
        })
    );

    let joint = world.joint(mesh_node).unwrap();
    assert_eq!(joint.joint_type, JointKind::Fixed);
    assert_eq!(joint.connected_entity, Some(world.active_camera()));

    let skeleton = world.animation_skeleton(mesh_node).unwrap();
    assert_eq!(
        skeleton.skeleton,
        project_animation_skeleton_handle(&project, "res://animation/hero.skeleton.zranim")
    );

    let animation_player = world.animation_player(mesh_node).unwrap();
    assert_eq!(
        animation_player.clip,
        project_animation_clip_handle(&project, "res://animation/hero.clip.zranim")
    );
    assert_eq!(animation_player.weight, 0.8);

    let sequence_player = world.animation_sequence_player(mesh_node).unwrap();
    assert_eq!(
        sequence_player.sequence,
        project_animation_sequence_handle(&project, "res://animation/hero.sequence.zranim")
    );
    assert!(!sequence_player.looping);

    let graph_player = world.animation_graph_player(mesh_node).unwrap();
    assert_eq!(
        graph_player.graph,
        project_animation_graph_handle(&project, "res://animation/hero.graph.zranim")
    );
    assert_eq!(
        graph_player.parameters.get("speed"),
        Some(&AnimationParameterValue::Scalar(1.5))
    );

    let state_machine_player = world.animation_state_machine_player(mesh_node).unwrap();
    assert_eq!(
        state_machine_player.state_machine,
        project_animation_state_machine_handle(
            &project,
            "res://animation/hero.state_machine.zranim"
        )
    );
    assert_eq!(
        state_machine_player.active_state.as_deref(),
        Some("Locomotion")
    );

    let saved = world.to_scene_asset(&project).unwrap();
    let saved_mesh = saved
        .entities
        .iter()
        .find(|entity| entity.entity == mesh_node)
        .unwrap();
    assert_eq!(
        saved_mesh
            .collider
            .as_ref()
            .and_then(|collider| collider.material.as_ref())
            .unwrap()
            .to_string(),
        "res://physics/default.physics_material.toml"
    );
    assert_eq!(
        saved_mesh
            .animation_player
            .as_ref()
            .unwrap()
            .clip
            .to_string(),
        "res://animation/hero.clip.zranim"
    );
    assert_eq!(
        saved_mesh
            .animation_graph_player
            .as_ref()
            .unwrap()
            .parameters
            .get("grounded"),
        Some(&AnimationParameterValue::Bool(true))
    );

    let _ = fs::remove_dir_all(root);
}
