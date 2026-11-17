//! Ragdoll world origins remain affine even when the output body is rigid.

use std::sync::Arc;

use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, SkeletalPoseTarget, SkeletalPoseTargets,
};
use zircon_runtime::core::math::{Quat, Transform, Vec3};
use zircon_runtime::scene::components::{NodeKind, RigidBodyType};
use zircon_runtime::scene::{EntityId, World};

use super::{
    drive_ragdoll_bodies_from_animation, RagdollBoneProfile, RagdollMode, RagdollProfile,
    RagdollRuntime,
};

const HIPS: &str = "Armature/Hips";
const HAND: &str = "Armature/Hips/Hand";
const ORIGIN: Vec3 = Vec3::new(10.0, 0.0, -3.0);
const C: f32 = std::f32::consts::FRAC_1_SQRT_2;

fn yaw_45() -> Quat {
    Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)
}

fn bone(path: &str, parent: Option<&str>, offset: Vec3) -> RagdollBoneProfile {
    RagdollBoneProfile {
        bone_path: path.to_owned(),
        parent_bone_path: parent.map(str::to_owned),
        shape: PhysicsColliderShape::Sphere { radius: 0.25 },
        body_offset: Transform::from_translation(offset),
        mass: 1.0,
        blend_weight: 1.0,
        constraint: Default::default(),
    }
}

fn profile(bones: Vec<RagdollBoneProfile>) -> RagdollProfile {
    RagdollProfile {
        id: "affine-origins".to_owned(),
        bones,
    }
}

fn target(path: &str, local_transform: Transform) -> SkeletalPoseTarget {
    SkeletalPoseTarget {
        bone_name: path.to_owned(),
        local_transform,
        normalized_weight: 1.0,
    }
}

fn skeleton_fixture(sheared_scene: bool) -> (World, EntityId) {
    let mut world = World::empty();
    let skeleton = world.spawn_node(NodeKind::Empty).unwrap();
    if sheared_scene {
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .update_transform(
                parent,
                Transform {
                    translation: ORIGIN,
                    scale: Vec3::new(2.0, 1.0, 1.0),
                    ..Transform::default()
                },
            )
            .unwrap();
        world
            .update_transform(
                skeleton,
                Transform {
                    rotation: yaw_45(),
                    ..Transform::default()
                },
            )
            .unwrap();
        assert!(world.set_parent_checked(skeleton, Some(parent)).unwrap());
        let scene = world.world_matrix(skeleton).unwrap();
        assert!((scene.x_axis.dot(scene.z_axis) - 1.5).abs() < 1.0e-5);
    } else {
        world
            .update_transform(skeleton, Transform::from_translation(ORIGIN))
            .unwrap();
    }
    world.insert_resource(SkeletalPoseTargets::default());
    (world, skeleton)
}

fn set_pose(world: &mut World, skeleton: EntityId, rows: Vec<SkeletalPoseTarget>) {
    world
        .resource_mut::<SkeletalPoseTargets>()
        .replace(skeleton, Arc::from(rows));
}

fn chain_pose(hand_rotation: Quat) -> Vec<SkeletalPoseTarget> {
    vec![
        target(
            HIPS,
            Transform {
                translation: Vec3::X,
                scale: Vec3::new(2.0, 1.0, 1.0),
                ..Transform::default()
            },
        ),
        target(
            HAND,
            Transform {
                translation: Vec3::Z,
                rotation: hand_rotation,
                ..Transform::default()
            },
        ),
    ]
}

fn offset_chain() -> RagdollProfile {
    profile(vec![
        bone(HIPS, None, 2.0 * Vec3::Y),
        bone(HAND, Some(HIPS), Vec3::X),
    ])
}

fn root_body_position(world: &World, body: EntityId) -> Vec3 {
    assert_eq!(
        world.parent_of(body),
        None,
        "profile bodies use joints, not scene parenting"
    );
    world.world_matrix(body).unwrap().w_axis.truncate()
}

fn assert_point(label: &str, observed: Vec3, expected: Vec3) {
    assert!(
        observed.abs_diff_eq(expected, 1.0e-5),
        "{label}: {observed:?}; expected {expected:?}"
    );
}

#[test]
fn spawn_resolves_world_point_under_a_sheared_skeleton() {
    let (mut world, skeleton) = skeleton_fixture(true);
    let before = world.world_matrix(skeleton).unwrap();
    set_pose(
        &mut world,
        skeleton,
        vec![target(HIPS, Transform::from_translation(Vec3::X))],
    );
    let spawn = profile(vec![bone(HIPS, None, Vec3::ZERO)])
        .spawn(&mut world, skeleton)
        .unwrap();
    assert_eq!(spawn.bodies_by_bone.len(), 1);
    let world_point = root_body_position(&world, spawn.bodies_by_bone[HIPS]);
    assert_eq!(world.world_matrix(skeleton).unwrap(), before);
    eprintln!("SKELETAL_AFFINE source=scene_spawn point={world_point:?}");
    // diag(2,1,1) * Ry(45°) * X = (2C,0,-C).
    assert_point(
        "scene root world origin",
        world_point,
        ORIGIN + Vec3::new(2.0 * C, 0.0, -C),
    );
}

#[test]
fn spawn_applies_the_offset_after_the_bone_chain() {
    let (mut world, skeleton) = skeleton_fixture(false);
    set_pose(&mut world, skeleton, chain_pose(yaw_45()));
    let spawn = offset_chain().spawn(&mut world, skeleton).unwrap();
    let hip = spawn.bodies_by_bone[HIPS];
    let hand = spawn.bodies_by_bone[HAND];
    assert_eq!(world.joint(hand).unwrap().connected_entity, Some(hip));
    assert_point(
        "parent body offset",
        root_body_position(&world, hip),
        ORIGIN + Vec3::new(1.0, 2.0, 0.0),
    );
    let world_point = root_body_position(&world, hand);
    eprintln!("SKELETAL_AFFINE source=chain_spawn point={world_point:?}");
    // The parent bone transforms the hand pose, not the parent's offset body.
    let expected = ORIGIN + Vec3::new(1.0 + 2.0 * C, 0.0, 1.0 - C);
    assert_point("spawned child offset world origin", world_point, expected);
}

#[test]
fn animated_drive_resolves_the_child_offset_from_the_parent_bone() {
    let (mut world, skeleton) = skeleton_fixture(false);
    set_pose(&mut world, skeleton, chain_pose(Quat::IDENTITY));
    let spawn = offset_chain()
        .spawn_configured(&mut world, skeleton, RagdollMode::Animated)
        .unwrap();
    let hip = spawn.bodies_by_bone[HIPS];
    let hand = spawn.bodies_by_bone[HAND];
    let mut ragdolls = world.resource::<RagdollRuntime>().clone();
    let collider = world.collider(hand).unwrap().clone();
    let initial = root_body_position(&world, hand);
    assert_point(
        "unsheared initial child offset",
        initial,
        ORIGIN + Vec3::new(3.0, 0.0, 1.0),
    );
    set_pose(&mut world, skeleton, chain_pose(yaw_45()));
    drive_ragdoll_bodies_from_animation(&mut world, &mut ragdolls, 1.0);
    assert_eq!(
        world.rigid_body(hand).unwrap().body_type,
        RigidBodyType::Kinematic
    );
    assert_eq!(world.collider(hand), Some(&collider));
    assert_point(
        "parent body does not displace the child bone",
        root_body_position(&world, hip),
        ORIGIN + Vec3::new(1.0, 2.0, 0.0),
    );
    let local = world.local_transform(hand).unwrap();
    // These are the preexisting rigid orientation/scale approximation; only
    // the position should change in this fix.
    assert_eq!(local.scale, Vec3::new(2.0, 1.0, 1.0));
    assert!(local.rotation.abs_diff_eq(yaw_45(), 1.0e-5));
    let world_point = root_body_position(&world, hand);
    eprintln!("SKELETAL_AFFINE source=chain_drive point={world_point:?}");
    assert_point(
        "animated child offset world origin",
        world_point,
        ORIGIN + Vec3::new(1.0 + 2.0 * C, 0.0, 1.0 - C),
    );
}

#[test]
fn release_uses_two_affine_world_targets_and_preserves_the_last_point() {
    let (mut world, skeleton) = skeleton_fixture(true);
    set_pose(
        &mut world,
        skeleton,
        vec![target(HIPS, Transform::default())],
    );
    let spawn = profile(vec![bone(HIPS, None, Vec3::ZERO)])
        .spawn_configured(&mut world, skeleton, RagdollMode::Animated)
        .unwrap();
    let body = spawn.bodies_by_bone[HIPS];
    assert_point(
        "initial body origin",
        root_body_position(&world, body),
        ORIGIN,
    );
    let mut ragdolls = world.resource::<RagdollRuntime>().clone();
    let mut poses = [Vec3::ZERO; 2];
    for (i, point) in poses.iter_mut().enumerate() {
        set_pose(
            &mut world,
            skeleton,
            vec![target(
                HIPS,
                Transform::from_translation((i as f32 + 1.0) * Vec3::X),
            )],
        );
        drive_ragdoll_bodies_from_animation(&mut world, &mut ragdolls, 0.5);
        *point = root_body_position(&world, body);
        let rigid = world.rigid_body(body).unwrap();
        assert_eq!(rigid.body_type, RigidBodyType::Kinematic);
        assert_eq!(rigid.linear_velocity, Vec3::ZERO);
    }
    ragdolls.set_mode(skeleton, RagdollMode::Simulated);
    drive_ragdoll_bodies_from_animation(&mut world, &mut ragdolls, 0.5);
    let rigid = world.rigid_body(body).unwrap();
    assert_eq!(rigid.body_type, RigidBodyType::Dynamic);
    let release = root_body_position(&world, body);
    assert_eq!(
        release, poses[1],
        "release must not move or reparent the body"
    );
    let velocity = rigid.linear_velocity;
    eprintln!(
        "SKELETAL_AFFINE source=release first={:?} second={:?} velocity={velocity:?}",
        poses[0], poses[1]
    );
    let step = Vec3::new(2.0 * C, 0.0, -C);
    assert_point("release world velocity", velocity, 2.0 * step);
    assert_point("first animated world point", poses[0], ORIGIN + step);
    assert_point("second animated world point", poses[1], ORIGIN + 2.0 * step);
}
