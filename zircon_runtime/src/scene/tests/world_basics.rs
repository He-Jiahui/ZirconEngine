use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Transform, Vec3};

use crate::scene::{world::World, NodeKind, SystemStage};

use super::support::{material_handle, model_handle};

#[test]
fn world_bootstraps_with_renderable_defaults() {
    let world = World::new();
    let snapshot = world.to_render_snapshot();

    assert!(!snapshot.scene.meshes.is_empty());
    assert!(snapshot.overlays.grid.is_none());
    assert!(snapshot.overlays.selection.is_empty());
    assert!(snapshot.overlays.selection_anchors.is_empty());
    assert!(snapshot.overlays.handles.is_empty());
    assert!(snapshot.overlays.scene_gizmos.is_empty());
    assert_eq!(
        world.schedule().stages,
        vec![
            SystemStage::PreUpdate,
            SystemStage::FixedUpdate,
            SystemStage::Update,
            SystemStage::LateUpdate,
            SystemStage::RenderExtract,
        ]
    );
}

#[test]
fn spawned_entities_have_unique_ids() {
    let mut world = World::new();
    let first = world.spawn_node(NodeKind::Cube);
    let second = world.spawn_node(NodeKind::Cube);
    assert_ne!(first, second);
}

#[test]
fn hierarchy_updates_world_transform() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();

    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
}

#[test]
fn updated_transform_is_reflected_in_render_extract() {
    let mut world = World::new();
    let cube = world
        .nodes()
        .iter()
        .find(|node| matches!(node.kind, NodeKind::Cube))
        .unwrap()
        .id;
    world
        .update_transform(cube, Transform::from_translation(Vec3::new(2.0, 3.0, 4.0)))
        .unwrap();

    let snapshot = world.to_render_extract();
    let mesh_snapshot = snapshot
        .scene
        .meshes
        .iter()
        .find(|mesh_snapshot| mesh_snapshot.node_id == cube)
        .unwrap();
    assert_eq!(
        mesh_snapshot.transform.translation,
        Vec3::new(2.0, 3.0, 4.0)
    );
}

#[test]
fn project_roundtrip_preserves_imported_meshes() {
    let mut world = World::new();
    let imported = world.spawn_mesh_node(
        model_handle("res://models/robot.obj"),
        material_handle("res://materials/robot.material.toml"),
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_scene_roundtrip_{unique}.json"));
    world.save_project_to_path(&path).unwrap();
    let saved = fs::read_to_string(&path).unwrap();
    let loaded = World::load_project_from_path(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert!(!saved.contains("selected"));
    let imported_node = loaded.find_node(imported).unwrap();
    assert!(matches!(imported_node.kind, NodeKind::Mesh));
    assert_eq!(
        imported_node.mesh.as_ref().unwrap().model,
        model_handle("res://models/robot.obj")
    );
}

#[test]
fn node_record_roundtrip_restores_same_entity() {
    let mut world = World::new();
    let cube = world.spawn_node(NodeKind::Cube);
    let record = world.node_record(cube).unwrap();

    assert!(world.remove_entity(cube));
    assert!(!world.contains_entity(cube));

    world.insert_node_record(record.clone()).unwrap();

    let restored = world.node_record(cube).unwrap();
    assert_eq!(restored, record);
}

#[test]
fn recursive_remove_returns_parent_and_children_records() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let removed = world.remove_entity_recursive(parent);
    assert_eq!(removed.len(), 2);
    assert!(!world.contains_entity(parent));
    assert!(!world.contains_entity(child));
}

#[test]
fn set_parent_checked_rejects_hierarchy_cycles() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();

    let error = world.set_parent_checked(parent, Some(child)).unwrap_err();

    assert!(error.contains("cycle"));
    assert_eq!(world.find_node(parent).unwrap().parent, None);
    assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
}

#[test]
fn render_extract_separates_directional_point_and_spot_lights() {
    let mut world = World::new();
    let point = world.spawn_node(NodeKind::PointLight);
    let spot = world.spawn_node(NodeKind::SpotLight);

    world
        .update_transform(point, Transform::from_translation(Vec3::new(3.0, 4.0, 5.0)))
        .unwrap();
    world
        .update_transform(spot, Transform::from_translation(Vec3::new(-2.0, 6.0, 1.5)))
        .unwrap();

    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.color").unwrap(),
            ScenePropertyValue::Vec3([0.2, 0.4, 0.8]),
        )
        .unwrap();
    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(6.5),
        )
        .unwrap();
    world
        .set_property(
            point,
            &ComponentPropertyPath::parse("PointLight.range").unwrap(),
            ScenePropertyValue::Scalar(9.0),
        )
        .unwrap();

    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.direction").unwrap(),
            ScenePropertyValue::Vec3([0.0, -1.0, 0.25]),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.color").unwrap(),
            ScenePropertyValue::Vec3([1.0, 0.8, 0.3]),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.intensity").unwrap(),
            ScenePropertyValue::Scalar(12.0),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.range").unwrap(),
            ScenePropertyValue::Scalar(15.0),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.inner_angle_radians").unwrap(),
            ScenePropertyValue::Scalar(0.35),
        )
        .unwrap();
    world
        .set_property(
            spot,
            &ComponentPropertyPath::parse("SpotLight.outer_angle_radians").unwrap(),
            ScenePropertyValue::Scalar(0.65),
        )
        .unwrap();

    let snapshot = world.to_render_extract();

    assert_eq!(snapshot.scene.directional_lights.len(), 1);

    let point_light = snapshot
        .scene
        .point_lights
        .iter()
        .find(|light| light.node_id == point)
        .unwrap();
    assert_eq!(point_light.position, Vec3::new(3.0, 4.0, 5.0));
    assert_eq!(point_light.color, Vec3::new(0.2, 0.4, 0.8));
    assert_eq!(point_light.intensity, 6.5);
    assert_eq!(point_light.range, 9.0);

    let spot_light = snapshot
        .scene
        .spot_lights
        .iter()
        .find(|light| light.node_id == spot)
        .unwrap();
    assert_eq!(spot_light.position, Vec3::new(-2.0, 6.0, 1.5));
    assert_eq!(spot_light.direction, Vec3::new(0.0, -1.0, 0.25));
    assert_eq!(spot_light.color, Vec3::new(1.0, 0.8, 0.3));
    assert_eq!(spot_light.intensity, 12.0);
    assert_eq!(spot_light.range, 15.0);
    assert_eq!(spot_light.inner_angle_radians, 0.35);
    assert_eq!(spot_light.outer_angle_radians, 0.65);
}
