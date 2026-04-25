use crate::core::framework::scene::{ComponentPropertyPath, EntityPath, ScenePropertyValue};
use crate::core::math::{Quat, Transform, Vec3};
use crate::core::resource::{AnimationClipMarker, ResourceHandle, ResourceId};
use crate::scene::components::{
    AnimationPlayerComponent, NodeKind, RigidBodyComponent, RigidBodyType,
};
use crate::scene::world::World;

#[test]
fn world_resolves_entity_paths_and_mutates_component_properties() {
    let mut world = World::new();
    let root = world.spawn_node(NodeKind::Cube);
    world.rename_node(root, "Root").unwrap();

    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    world
        .update_transform(hero, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();
    world
        .set_rigid_body(
            hero,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Dynamic,
                mass: 2.5,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    world
        .set_animation_player(
            hero,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "res://animation/hero.clip.zranim",
                )),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 0.25,
                looping: true,
                playing: true,
            }),
        )
        .unwrap();

    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let translation_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let mass_path = ComponentPropertyPath::parse("RigidBody.mass").unwrap();
    let weight_path = ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap();

    assert_eq!(world.entity_path(hero), Some(entity_path.clone()));
    assert_eq!(world.resolve_entity_path(&entity_path), Some(hero));
    assert_eq!(
        world.property(hero, &translation_path).unwrap(),
        ScenePropertyValue::Vec3([1.0, 2.0, 3.0])
    );
    assert_eq!(
        world.property(hero, &mass_path).unwrap(),
        ScenePropertyValue::Scalar(2.5)
    );
    assert_eq!(
        world.property(hero, &weight_path).unwrap(),
        ScenePropertyValue::Scalar(0.25)
    );

    assert!(world
        .set_property(
            hero,
            &translation_path,
            ScenePropertyValue::Vec3([4.0, 5.0, 6.0]),
        )
        .unwrap());
    assert!(world
        .set_property(hero, &mass_path, ScenePropertyValue::Scalar(5.5))
        .unwrap());
    assert!(world
        .set_property(hero, &weight_path, ScenePropertyValue::Scalar(0.75))
        .unwrap());
    assert!(!world
        .set_property(hero, &weight_path, ScenePropertyValue::Scalar(0.75))
        .unwrap());

    let node = world.find_node(hero).unwrap();
    assert_eq!(node.transform.translation, Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(world.rigid_body(hero).unwrap().mass, 5.5);
    assert_eq!(world.animation_player(hero).unwrap().weight, 0.75);
    assert_eq!(
        world.property(hero, &translation_path).unwrap(),
        ScenePropertyValue::Vec3([4.0, 5.0, 6.0])
    );
    assert_eq!(
        world.property(hero, &mass_path).unwrap(),
        ScenePropertyValue::Scalar(5.5)
    );
    assert_eq!(
        world.property(hero, &weight_path).unwrap(),
        ScenePropertyValue::Scalar(0.75)
    );
}

#[test]
fn world_rejects_zero_length_transform_rotation_property_writes() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    let rotation_path = ComponentPropertyPath::parse("Transform.rotation").unwrap();
    let rotation_w_path = ComponentPropertyPath::parse("Transform.rotation.w").unwrap();

    let error = world
        .set_property(
            hero,
            &rotation_path,
            ScenePropertyValue::Quaternion([0.0, 0.0, 0.0, 0.0]),
        )
        .unwrap_err();
    assert!(error.contains("zero-length"), "{error}");
    assert_eq!(
        world.find_node(hero).unwrap().transform.rotation,
        Quat::IDENTITY
    );

    let error = world
        .set_property(hero, &rotation_w_path, ScenePropertyValue::Scalar(0.0))
        .unwrap_err();
    assert!(error.contains("zero-length"), "{error}");
    assert_eq!(
        world.find_node(hero).unwrap().transform.rotation,
        Quat::IDENTITY
    );
}

#[test]
fn world_rejects_non_finite_transform_property_writes() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    let translation_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let translation_x_path = ComponentPropertyPath::parse("Transform.translation.x").unwrap();
    let scale_path = ComponentPropertyPath::parse("Transform.scale").unwrap();

    let error = world
        .set_property(
            hero,
            &translation_path,
            ScenePropertyValue::Vec3([f32::NAN, 1.0, 2.0]),
        )
        .unwrap_err();
    assert!(error.contains("finite"), "{error}");
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::ZERO
    );

    let error = world
        .set_property(
            hero,
            &translation_x_path,
            ScenePropertyValue::Scalar(f32::INFINITY),
        )
        .unwrap_err();
    assert!(error.contains("finite"), "{error}");
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::ZERO
    );

    let error = world
        .set_property(
            hero,
            &scale_path,
            ScenePropertyValue::Vec3([1.0, f32::NEG_INFINITY, 1.0]),
        )
        .unwrap_err();
    assert!(error.contains("finite"), "{error}");
    assert_eq!(world.find_node(hero).unwrap().transform.scale, Vec3::ONE);
}
