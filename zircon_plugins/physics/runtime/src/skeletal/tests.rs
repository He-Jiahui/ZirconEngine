use std::sync::Arc;

use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsWorldStepPlan, SkeletalPoseTarget, SkeletalPoseTargets,
};
use zircon_runtime::core::framework::scene::physics::{
    PhysicsJointConstraintMetadata, PhysicsSkeletonJointBinding,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{
    JointComponent, JointKind, NodeKind, RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::scene::world::World;

use crate::backend::builtin::integrate_builtin_physics_steps;
use crate::manager::build_world_sync_state;

use super::runtime::{
    drive_ragdoll_bodies_from_animation, write_simulated_pose_feed, RagdollMode, RagdollRuntime,
};
use super::{RagdollBoneProfile, RagdollProfile};

#[test]
fn ragdoll_profile_spawns_expected_body_count() {
    let profile = RagdollProfile::from_toml(
        r#"
            id = "humanoid"

            [[bones]]
            bone_path = "Armature/Hips"
            mass = 4.0
            blend_weight = 1.0
            [bones.shape]
            kind = "capsule"
            radius = 0.25
            half_height = 0.4

            [[bones]]
            bone_path = "Armature/Hips/Spine"
            parent_bone_path = "Armature/Hips"
            mass = 2.0
            blend_weight = 0.75
            [bones.shape]
            kind = "box"
            half_extents = [0.2, 0.3, 0.1]
        "#,
    )
    .expect("profile should parse");
    let mut world = World::empty();
    let skeleton = world.spawn_node(NodeKind::Empty);
    world.insert_resource(SkeletalPoseTargets::default());
    world.resource_mut::<SkeletalPoseTargets>().replace(
        skeleton,
        Arc::from([
            SkeletalPoseTarget {
                bone_name: "Hips".to_string(),
                local_transform: translated(1.0),
                normalized_weight: 1.0,
            },
            SkeletalPoseTarget {
                bone_name: "Spine".to_string(),
                local_transform: translated(2.0),
                normalized_weight: 1.0,
            },
        ]),
    );

    let spawn = profile
        .spawn_configured(&mut world, skeleton, RagdollMode::Blended { weight: 0.5 })
        .expect("profile should spawn");

    assert_eq!(spawn.bodies_by_bone.len(), 2);
    let hips = spawn.bodies_by_bone["Armature/Hips"];
    let spine = spawn.bodies_by_bone["Armature/Hips/Spine"];
    assert_eq!(world.world_transform(hips), Some(translated(1.0)));
    assert_eq!(world.world_transform(spine), Some(translated(3.0)));
    assert_eq!(
        world
            .joint(spine)
            .and_then(|joint| joint.skeleton_binding.as_ref())
            .and_then(|binding| binding.parent_bone_path.as_deref()),
        Some("Armature/Hips")
    );
    assert_eq!(
        world.resource::<RagdollRuntime>().mode(skeleton),
        Some(RagdollMode::Blended { weight: 0.5 })
    );
}

#[test]
fn ragdoll_profile_rejects_parent_cycles_before_scene_mutation() {
    let error = RagdollProfile::from_toml(
        r#"
            id = "cycle"

            [[bones]]
            bone_path = "A"
            parent_bone_path = "B"
            [bones.shape]
            kind = "sphere"
            radius = 0.25

            [[bones]]
            bone_path = "B"
            parent_bone_path = "A"
            [bones.shape]
            kind = "sphere"
            radius = 0.25
        "#,
    )
    .expect_err("cycle should be rejected");

    assert!(matches!(
        error,
        super::RagdollProfileError::ParentCycle { .. }
    ));
}

#[test]
fn animated_to_simulated_switch_has_no_pose_pop() {
    let mut world = World::empty();
    let skeleton = world.spawn_node(NodeKind::Empty);
    let hand_body = spawn_bound_body(&mut world, skeleton, "Armature/Hand", None);
    world.insert_resource(SkeletalPoseTargets::default());

    world.resource_mut::<SkeletalPoseTargets>().replace(
        skeleton,
        Arc::from([SkeletalPoseTarget {
            bone_name: "Hand".to_string(),
            local_transform: translated(3.0),
            normalized_weight: 1.0,
        }]),
    );

    let mut ragdolls = RagdollRuntime::default();
    ragdolls.configure(skeleton, RagdollMode::Animated);
    drive_ragdoll_bodies_from_animation(&mut world, &mut ragdolls, 1.0);
    assert_eq!(world.world_transform(hand_body), Some(translated(3.0)));
    assert_eq!(
        world.rigid_body(hand_body).map(|body| body.body_type),
        Some(RigidBodyType::Kinematic)
    );

    world.resource_mut::<SkeletalPoseTargets>().replace(
        skeleton,
        Arc::from([SkeletalPoseTarget {
            bone_name: "Hand".to_string(),
            local_transform: translated(4.0),
            normalized_weight: 1.0,
        }]),
    );
    drive_ragdoll_bodies_from_animation(&mut world, &mut ragdolls, 1.0);

    ragdolls.set_mode(skeleton, RagdollMode::Simulated);
    drive_ragdoll_bodies_from_animation(&mut world, &mut ragdolls, 1.0);
    assert_eq!(world.world_transform(hand_body), Some(translated(4.0)));
    assert_eq!(
        world.rigid_body(hand_body).map(|body| body.body_type),
        Some(RigidBodyType::Dynamic)
    );
    assert_eq!(
        world.rigid_body(hand_body).map(|body| body.linear_velocity),
        Some(Vec3::new(1.0, 0.0, 0.0))
    );
}

#[test]
fn ragdoll_drop_golden_snapshot() {
    let profile = RagdollProfile {
        id: "drop".to_string(),
        bones: vec![RagdollBoneProfile {
            bone_path: "Armature/Hand".to_string(),
            parent_bone_path: None,
            shape: PhysicsColliderShape::Sphere { radius: 0.25 },
            mass: 1.0,
            body_offset: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            constraint: PhysicsJointConstraintMetadata::default(),
            blend_weight: 1.0,
        }],
    };
    let mut world = World::empty();
    let skeleton = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(
            skeleton,
            Transform::from_translation(Vec3::new(0.0, 10.0, 0.0)),
        )
        .unwrap();
    world.insert_resource(SkeletalPoseTargets::default());
    world.resource_mut::<SkeletalPoseTargets>().replace(
        skeleton,
        Arc::from([SkeletalPoseTarget {
            bone_name: "Hand".to_string(),
            local_transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
            normalized_weight: 1.0,
        }]),
    );
    profile
        .spawn_configured(&mut world, skeleton, RagdollMode::Simulated)
        .expect("drop profile should spawn");

    integrate_builtin_physics_steps(
        &mut world,
        PhysicsWorldStepPlan {
            steps: 1,
            step_seconds: 0.1,
            remaining_seconds: 0.0,
            interpolation_alpha: 1.0,
        },
    );
    let sync = build_world_sync_state(WorldHandle::new(8), &world);
    let ragdolls = world.resource::<RagdollRuntime>().clone();
    let mut feed = Default::default();
    write_simulated_pose_feed(&world, &sync, &ragdolls, 1.0, &mut feed);

    let rows = feed.targets(skeleton).expect("drop feed should exist");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].bone_name, "Hand");
    assert!((rows[0].local_transform.translation.y - 1.9019).abs() < 1.0e-4);
    assert_eq!(rows[0].normalized_weight, 1.0);
}

#[test]
fn blended_ragdoll_writes_local_pose_with_composed_weight() {
    let mut world = World::empty();
    let skeleton = world.spawn_node(NodeKind::Empty);
    world.update_transform(skeleton, translated(10.0)).unwrap();
    let hand_body = spawn_bound_body(&mut world, skeleton, "Armature/Hand", None);
    world.update_transform(hand_body, translated(12.0)).unwrap();

    let mut ragdolls = RagdollRuntime::default();
    ragdolls.configure(skeleton, RagdollMode::Blended { weight: 0.5 });
    ragdolls.set_bone_weight(skeleton, "Armature/Hand", 0.25);

    let sync = build_world_sync_state(WorldHandle::new(7), &world);
    let mut feed = Default::default();
    write_simulated_pose_feed(&world, &sync, &ragdolls, 0.8, &mut feed);

    let rows = feed.targets(skeleton).expect("skeleton feed should exist");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].bone_name, "Hand");
    assert!((rows[0].local_transform.translation.x - 2.0).abs() < 1.0e-5);
    assert!((rows[0].normalized_weight - 0.1).abs() < 1.0e-5);
}

fn spawn_bound_body(
    world: &mut World,
    skeleton: u64,
    bone_path: &str,
    parent_bone_path: Option<&str>,
) -> u64 {
    let body = world.spawn_node(NodeKind::Empty);
    world
        .set_rigid_body(body, Some(RigidBodyComponent::default()))
        .unwrap();
    world
        .set_joint(
            body,
            Some(JointComponent {
                joint_type: JointKind::Generic6Dof,
                connected_entity: Some(skeleton),
                skeleton_binding: Some(PhysicsSkeletonJointBinding {
                    skeleton_entity: skeleton,
                    bone_path: bone_path.to_string(),
                    parent_bone_path: parent_bone_path.map(str::to_string),
                }),
                ..JointComponent::default()
            }),
        )
        .unwrap();
    body
}

fn translated(x: f32) -> Transform {
    Transform {
        translation: Vec3::new(x, 0.0, 0.0),
        ..Transform::default()
    }
}
