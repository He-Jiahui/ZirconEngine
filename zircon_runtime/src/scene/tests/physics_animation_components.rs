use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};
use crate::core::math::{Transform, Vec3};
use crate::core::resource::{
    AnimationClipMarker, AnimationGraphMarker, AnimationSequenceMarker, AnimationSkeletonMarker,
    AnimationStateMachineMarker, PhysicsMaterialMarker, ResourceHandle, ResourceId,
};
use crate::scene::components::{
    AnimationGraphPlayerComponent, AnimationPlayerComponent, AnimationSequencePlayerComponent,
    AnimationSkeletonComponent, AnimationStateMachinePlayerComponent, ColliderComponent,
    ColliderShape, JointComponent, JointKind, NodeKind, RigidBodyComponent, RigidBodyType,
};
use crate::scene::world::World;

#[test]
fn world_project_roundtrip_preserves_physics_and_animation_components() {
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Mesh);

    let rigid_body = RigidBodyComponent {
        body_type: RigidBodyType::Dynamic,
        mass: 3.5,
        linear_velocity: Vec3::new(0.25, 0.0, 0.0),
        angular_velocity: Vec3::new(0.0, 0.5, 0.0),
        linear_damping: 0.2,
        angular_damping: 0.15,
        gravity_scale: 0.8,
        can_sleep: true,
        lock_translation: [false, false, false],
        lock_rotation: [false, true, false],
    };
    let collider = ColliderComponent {
        shape: ColliderShape::Capsule {
            radius: 0.45,
            half_height: 0.9,
        },
        sensor: false,
        layer: 3,
        collision_group: 6,
        collision_mask: 0x0000_00ff,
        material: Some(ResourceHandle::<PhysicsMaterialMarker>::new(
            ResourceId::from_stable_label("res://physics/default.physics_material.toml"),
        )),
        material_override: Some(PhysicsMaterialMetadata {
            static_friction: 0.6,
            dynamic_friction: 0.4,
            restitution: 0.1,
            friction_combine: PhysicsCombineRule::Maximum,
            restitution_combine: PhysicsCombineRule::Average,
        }),
        local_transform: Transform::default(),
    };
    let joint = JointComponent {
        joint_type: JointKind::Hinge,
        connected_entity: Some(world.active_camera()),
        anchor: crate::core::math::Vec3::new(0.0, 1.0, 0.0),
        axis: crate::core::math::Vec3::Y,
        limits: Some([-0.5, 0.5]),
        collide_connected: false,
    };
    let animation_skeleton = AnimationSkeletonComponent {
        skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(ResourceId::from_stable_label(
            "res://animation/hero.skeleton.zranim",
        )),
    };
    let animation_player = AnimationPlayerComponent {
        clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
            "res://animation/hero.clip.zranim",
        )),
        playback_speed: 1.2,
        time_seconds: 0.75,
        weight: 0.65,
        looping: true,
        playing: true,
    };
    let animation_sequence_player = AnimationSequencePlayerComponent {
        sequence: ResourceHandle::<AnimationSequenceMarker>::new(ResourceId::from_stable_label(
            "res://animation/hero.sequence.zranim",
        )),
        playback_speed: 0.9,
        time_seconds: 0.2,
        looping: false,
        playing: true,
    };
    let animation_graph_player = AnimationGraphPlayerComponent {
        graph: ResourceHandle::<AnimationGraphMarker>::new(ResourceId::from_stable_label(
            "res://animation/hero.graph.zranim",
        )),
        parameters: BTreeMap::from([
            ("grounded".to_string(), AnimationParameterValue::Bool(true)),
            ("speed".to_string(), AnimationParameterValue::Scalar(2.0)),
        ]),
        playing: true,
    };
    let animation_state_machine_player = AnimationStateMachinePlayerComponent {
        state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(
            ResourceId::from_stable_label("res://animation/hero.state_machine.zranim"),
        ),
        parameters: BTreeMap::from([
            ("grounded".to_string(), AnimationParameterValue::Bool(true)),
            ("speed".to_string(), AnimationParameterValue::Scalar(2.0)),
        ]),
        active_state: Some("Locomotion".to_string()),
        playing: true,
    };

    world
        .set_rigid_body(entity, Some(rigid_body.clone()))
        .unwrap();
    world.set_collider(entity, Some(collider.clone())).unwrap();
    world.set_joint(entity, Some(joint.clone())).unwrap();
    world
        .set_animation_skeleton(entity, Some(animation_skeleton.clone()))
        .unwrap();
    world
        .set_animation_player(entity, Some(animation_player.clone()))
        .unwrap();
    world
        .set_animation_sequence_player(entity, Some(animation_sequence_player.clone()))
        .unwrap();
    world
        .set_animation_graph_player(entity, Some(animation_graph_player.clone()))
        .unwrap();
    world
        .set_animation_state_machine_player(entity, Some(animation_state_machine_player.clone()))
        .unwrap();

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "zircon_scene_physics_animation_roundtrip_{unique}.json"
    ));
    world.save_project_to_path(&path).unwrap();
    let loaded = World::load_project_from_path(&path).unwrap();
    let _ = fs::remove_file(&path);

    assert_eq!(loaded.rigid_body(entity), Some(&rigid_body));
    assert_eq!(loaded.collider(entity), Some(&collider));
    assert_eq!(loaded.joint(entity), Some(&joint));
    assert_eq!(loaded.animation_skeleton(entity), Some(&animation_skeleton));
    assert_eq!(loaded.animation_player(entity), Some(&animation_player));
    assert_eq!(
        loaded.animation_sequence_player(entity),
        Some(&animation_sequence_player)
    );
    assert_eq!(
        loaded.animation_graph_player(entity),
        Some(&animation_graph_player)
    );
    assert_eq!(
        loaded.animation_state_machine_player(entity),
        Some(&animation_state_machine_player)
    );

    let node = loaded.find_node(entity).unwrap();
    assert_eq!(node.rigid_body.as_ref(), Some(&rigid_body));
    assert_eq!(node.animation_player.as_ref(), Some(&animation_player));
}
