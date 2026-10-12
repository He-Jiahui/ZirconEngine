use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::physics::{
    PhysicsColliderShape, PhysicsWorldStepPlan, SkeletalPoseTarget, SkeletalPoseTargets,
};
use zircon_runtime::core::framework::scene::physics::{
    PhysicsJointConstraintMetadata, PhysicsSkeletonJointBinding,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{
    ColliderComponent, JointComponent, JointKind, LocalTransform, Name, NodeKind,
    RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::scene::ecs::{Component, LifecycleEventKind};
use zircon_runtime::scene::world::{SceneError, World};
use zircon_runtime::scene::{EntityId, LifecycleTargetChange};

use crate::backend::builtin::integrate_builtin_physics_steps;
use crate::manager::build_world_sync_state;

use super::runtime::{
    drive_ragdoll_bodies_from_animation, write_simulated_pose_feed, RagdollMode, RagdollRuntime,
};
use super::{RagdollBoneProfile, RagdollProfile, RagdollProfileError, RagdollSpawnError};

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
    let skeleton = world
        .spawn_node(NodeKind::Empty)
        .expect("spawn skeleton fixture");
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
    let skeleton = world
        .spawn_node(NodeKind::Empty)
        .expect("spawn skeleton fixture");
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
    let skeleton = world
        .spawn_node(NodeKind::Empty)
        .expect("spawn skeleton fixture");
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
    let skeleton = world
        .spawn_node(NodeKind::Empty)
        .expect("spawn skeleton fixture");
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
    let body = world
        .spawn_node(NodeKind::Empty)
        .expect("spawn body fixture");
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

#[derive(Debug)]
struct RagdollPayloadDropProbe(Arc<AtomicUsize>);

impl Component for RagdollPayloadDropProbe {}

impl Drop for RagdollPayloadDropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn ragdoll_cancellation_fixture(paths: &[&str]) -> (World, EntityId, RagdollProfile) {
    let profile = RagdollProfile {
        id: "lifecycle-cancellation".to_owned(),
        bones: paths
            .iter()
            .map(|path| RagdollBoneProfile {
                bone_path: (*path).to_owned(),
                parent_bone_path: None,
                shape: PhysicsColliderShape::Sphere { radius: 0.1 },
                mass: 1.0,
                body_offset: Transform::default(),
                constraint: PhysicsJointConstraintMetadata::default(),
                blend_weight: 1.0,
            })
            .collect(),
    };
    let mut world = World::empty();
    let skeleton = world.spawn_node(NodeKind::Empty).unwrap();
    let rows = paths
        .iter()
        .map(|path| SkeletalPoseTarget {
            bone_name: (*path).to_owned(),
            local_transform: translated(1.0),
            normalized_weight: 1.0,
        })
        .collect::<Vec<_>>();
    let mut targets = SkeletalPoseTargets::default();
    targets.replace(skeleton, Arc::from(rows));
    world.insert_resource(targets);
    (world, skeleton, profile)
}

fn observe_ragdoll_body_events(
    world: &mut World,
    body_adds: Arc<AtomicUsize>,
    removals: Arc<Mutex<Vec<EntityId>>>,
) {
    world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |_, _| {
        body_adds.fetch_add(1, Ordering::SeqCst);
    });
    world.observe_component_lifecycle::<Name>(LifecycleEventKind::Remove, move |_, event| {
        removals.lock().unwrap().push(event.entity());
    });
}

fn ragdoll_replacement_world(
    entity_count: usize,
    body_adds: Arc<AtomicUsize>,
    removals: Arc<Mutex<Vec<EntityId>>>,
) -> (World, Vec<EntityId>) {
    let mut replacement = World::empty();
    let entities = (0..entity_count)
        .map(|_| {
            let entity = replacement.spawn_node(NodeKind::Empty).unwrap();
            replacement
                .update_transform(entity, translated(73.0))
                .unwrap();
            entity
        })
        .collect();
    observe_ragdoll_body_events(&mut replacement, body_adds, removals);
    (replacement, entities)
}

fn install_ragdoll_target_change<T: Component>(
    world: &mut World,
    kind: LifecycleEventKind,
    body: EntityId,
    cause: LifecycleTargetChange,
    replacement: World,
    calls: Arc<AtomicUsize>,
) {
    let replacement = Mutex::new(Some(replacement));
    world.observe_component_lifecycle::<T>(kind, move |world, event| {
        if event.entity() != body {
            return;
        }
        calls.fetch_add(1, Ordering::SeqCst);
        match cause {
            LifecycleTargetChange::World => *world = replacement.lock().unwrap().take().unwrap(),
            LifecycleTargetChange::Schedule => *world.schedule_mut() = Default::default(),
        }
    });
}

fn assert_ragdoll_primary_cancellation(error: &RagdollSpawnError, cause: LifecycleTargetChange) {
    assert_eq!(error.lifecycle_target_change(), Some(cause));
    let RagdollSpawnError::SceneMutation {
        source,
        cleanup_cancellation,
    } = error
    else {
        panic!("a real scene cancellation must keep its producer source");
    };
    assert_eq!(source.lifecycle_target_change(), Some(cause));
    assert!(cleanup_cancellation.is_none());
    assert_eq!(
        std::error::Error::source(error)
            .unwrap()
            .downcast_ref::<SceneError>(),
        Some(source)
    );
}

#[test]
fn ragdoll_spawn_validation_keeps_the_profile_error_contract() {
    let (mut world, skeleton, mut profile) = ragdoll_cancellation_fixture(&["A"]);
    let publications = Arc::new(AtomicUsize::new(0));
    observe_ragdoll_body_events(
        &mut world,
        publications.clone(),
        Arc::new(Mutex::new(Vec::new())),
    );
    profile.id.clear();
    let validation = profile.validate().unwrap_err();
    assert_eq!(validation.clone(), RagdollProfileError::EmptyId);
    let error = profile.spawn(&mut world, skeleton).unwrap_err();
    assert_eq!(error.lifecycle_target_change(), None);
    assert_eq!(error.to_string(), validation.to_string());
    assert!(matches!(&error, RagdollSpawnError::Profile(error) if *error == validation));
    assert_eq!(
        std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<RagdollProfileError>(),
        Some(&validation)
    );
    assert_eq!(publications.load(Ordering::SeqCst), 0);
    assert!(world.get::<RigidBodyComponent>(skeleton).is_none());
    assert!(world.get_resource::<RagdollRuntime>().is_none());
}

#[test]
fn ragdoll_spawn_cancellation_stops_before_body_configuration() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let (mut world, skeleton, profile) = ragdoll_cancellation_fixture(&["A", "B"]);
        let body_adds = Arc::new(AtomicUsize::new(0));
        let removals = Arc::new(Mutex::new(Vec::new()));
        let calls = Arc::new(AtomicUsize::new(0));
        let (replacement, entities) =
            ragdoll_replacement_world(3, body_adds.clone(), removals.clone());
        assert_eq!(entities[0], skeleton);
        let first_body = entities[1];
        observe_ragdoll_body_events(&mut world, body_adds.clone(), removals.clone());
        install_ragdoll_target_change::<Name>(
            &mut world,
            LifecycleEventKind::Add,
            first_body,
            cause,
            replacement,
            calls.clone(),
        );
        let error = profile.spawn(&mut world, skeleton).unwrap_err();
        assert_ragdoll_primary_cancellation(&error, cause);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(body_adds.load(Ordering::SeqCst), 1);
        assert!(removals.lock().unwrap().is_empty());
        assert!(world.get::<RigidBodyComponent>(first_body).is_none());
        assert!(world.get::<ColliderComponent>(first_body).is_none());
        assert!(world.get::<JointComponent>(first_body).is_none());
        assert!(world.get_resource::<RagdollRuntime>().is_none());
        match cause {
            LifecycleTargetChange::World => {
                for entity in entities {
                    assert_eq!(world.world_transform(entity), Some(translated(73.0)));
                }
            }
            LifecycleTargetChange::Schedule => assert!(!world.contains_entity(entities[2])),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum RagdollConfigurationPhase {
    Transform,
    RigidBody,
    Collider,
    Joint,
}

#[test]
fn ragdoll_configure_cancellation_stops_the_resource_and_body_tail() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        for phase in [
            RagdollConfigurationPhase::Transform,
            RagdollConfigurationPhase::RigidBody,
            RagdollConfigurationPhase::Collider,
            RagdollConfigurationPhase::Joint,
        ] {
            let (mut world, skeleton, profile) = ragdoll_cancellation_fixture(&["A", "B"]);
            let body_adds = Arc::new(AtomicUsize::new(0));
            let removals = Arc::new(Mutex::new(Vec::new()));
            let calls = Arc::new(AtomicUsize::new(0));
            let (replacement, entities) =
                ragdoll_replacement_world(3, body_adds.clone(), removals.clone());
            assert_eq!(entities[0], skeleton);
            let first_body = entities[1];
            observe_ragdoll_body_events(&mut world, body_adds.clone(), removals.clone());
            match phase {
                RagdollConfigurationPhase::Transform => {
                    install_ragdoll_target_change::<LocalTransform>(
                        &mut world,
                        LifecycleEventKind::Replace,
                        first_body,
                        cause,
                        replacement,
                        calls.clone(),
                    )
                }
                RagdollConfigurationPhase::RigidBody => {
                    install_ragdoll_target_change::<RigidBodyComponent>(
                        &mut world,
                        LifecycleEventKind::Add,
                        first_body,
                        cause,
                        replacement,
                        calls.clone(),
                    )
                }
                RagdollConfigurationPhase::Collider => {
                    install_ragdoll_target_change::<ColliderComponent>(
                        &mut world,
                        LifecycleEventKind::Add,
                        first_body,
                        cause,
                        replacement,
                        calls.clone(),
                    )
                }
                RagdollConfigurationPhase::Joint => {
                    install_ragdoll_target_change::<JointComponent>(
                        &mut world,
                        LifecycleEventKind::Add,
                        first_body,
                        cause,
                        replacement,
                        calls.clone(),
                    )
                }
            }
            let error = profile
                .spawn_configured(&mut world, skeleton, RagdollMode::Simulated)
                .unwrap_err();
            assert_ragdoll_primary_cancellation(&error, cause);
            assert_eq!(calls.load(Ordering::SeqCst), 1, "phase: {phase:?}");
            assert_eq!(body_adds.load(Ordering::SeqCst), 1);
            assert!(removals.lock().unwrap().is_empty());
            assert!(world.get_resource::<RagdollRuntime>().is_none());
            match cause {
                LifecycleTargetChange::World => {
                    for entity in entities {
                        assert_eq!(world.world_transform(entity), Some(translated(73.0)));
                    }
                    assert!(world.get::<RigidBodyComponent>(first_body).is_none());
                    assert!(world.get::<ColliderComponent>(first_body).is_none());
                    assert!(world.get::<JointComponent>(first_body).is_none());
                }
                LifecycleTargetChange::Schedule => {
                    assert_eq!(world.world_transform(first_body), Some(translated(1.0)));
                    assert_eq!(
                        world.get::<RigidBodyComponent>(first_body).is_some(),
                        !matches!(phase, RagdollConfigurationPhase::Transform)
                    );
                    assert_eq!(
                        world.get::<ColliderComponent>(first_body).is_some(),
                        matches!(
                            phase,
                            RagdollConfigurationPhase::Collider | RagdollConfigurationPhase::Joint
                        )
                    );
                    assert_eq!(
                        world.get::<JointComponent>(first_body).is_some(),
                        matches!(phase, RagdollConfigurationPhase::Joint)
                    );
                    assert!(!world.contains_entity(entities[2]));
                }
            }
        }
    }
}

fn fail_ragdoll_configuration_by_removing_body(world: &mut World, body: EntityId) {
    // This permitted opaque same-World callback is a fixture for a real later
    // set_collider MissingEntity error; it does not test same-World reentry repair.
    world.observe_component_lifecycle::<RigidBodyComponent>(
        LifecycleEventKind::Add,
        move |world, event| {
            if event.entity() == body {
                drop(world.remove_entity_recursive(body).unwrap());
            }
        },
    );
}

#[test]
fn ragdoll_ordinary_error_keeps_the_cancelled_cleanup_batch_and_stops_tail() {
    for cause in [
        LifecycleTargetChange::World,
        LifecycleTargetChange::Schedule,
    ] {
        let (mut world, skeleton, profile) = ragdoll_cancellation_fixture(&["A", "B", "C", "D"]);
        let body_adds = Arc::new(AtomicUsize::new(0));
        let removals = Arc::new(Mutex::new(Vec::new()));
        let calls = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let (replacement, entities) =
            ragdoll_replacement_world(5, body_adds.clone(), removals.clone());
        assert_eq!(entities[0], skeleton);
        let [a, b, c, d] = [entities[1], entities[2], entities[3], entities[4]];
        observe_ragdoll_body_events(&mut world, body_adds.clone(), removals.clone());
        let probe_drops = drops.clone();
        world.observe_component_lifecycle::<RigidBodyComponent>(
            LifecycleEventKind::Add,
            move |world, event| {
                if event.entity() == c {
                    world
                        .insert(c, RagdollPayloadDropProbe(probe_drops.clone()))
                        .unwrap();
                }
            },
        );
        fail_ragdoll_configuration_by_removing_body(&mut world, d);
        install_ragdoll_target_change::<Name>(
            &mut world,
            LifecycleEventKind::Remove,
            c,
            cause,
            replacement,
            calls.clone(),
        );
        let error = profile
            .spawn_configured(&mut world, skeleton, RagdollMode::Simulated)
            .unwrap_err();
        assert_eq!(error.lifecycle_target_change(), Some(cause));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(body_adds.load(Ordering::SeqCst), 4);
        assert_eq!(*removals.lock().unwrap(), [d, c]);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(world.get_resource::<RagdollRuntime>().is_none());
        assert!(world.contains_entity(a));
        assert!(world.contains_entity(b));
        if cause == LifecycleTargetChange::World {
            for entity in entities {
                assert_eq!(world.world_transform(entity), Some(translated(73.0)));
            }
        } else {
            assert!(!world.contains_entity(c));
            assert!(!world.contains_entity(d));
        }
        let RagdollSpawnError::SceneMutation {
            source,
            cleanup_cancellation: Some(cleanup),
        } = error
        else {
            panic!("the actual cancelled removal must remain owned by the spawn error");
        };
        assert!(matches!(&source, SceneError::MissingEntity {
            operation: "update collider for", entity,
        } if *entity == d));
        assert_eq!(source.lifecycle_target_change(), None);
        assert_eq!(cleanup.error().lifecycle_target_change(), Some(cause));
        let (cancellation, batch) = cleanup.into_parts();
        let batch = batch.expect("a cancelled cleanup owns its exact detached batch");
        assert_eq!(batch.entity_ids().collect::<Vec<_>>(), [c]);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(cancellation);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(batch);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn ragdoll_ordinary_configuration_error_keeps_best_effort_cleanup_order() {
    let (mut world, skeleton, profile) = ragdoll_cancellation_fixture(&["A", "B", "C", "D"]);
    let body_adds = Arc::new(AtomicUsize::new(0));
    let removals = Arc::new(Mutex::new(Vec::new()));
    let (_, entities) = ragdoll_replacement_world(5, body_adds.clone(), removals.clone());
    let [a, b, c, d] = [entities[1], entities[2], entities[3], entities[4]];
    observe_ragdoll_body_events(&mut world, body_adds, removals.clone());
    fail_ragdoll_configuration_by_removing_body(&mut world, d);
    let error = profile.spawn(&mut world, skeleton).unwrap_err();
    assert_eq!(error.lifecycle_target_change(), None);
    let RagdollSpawnError::SceneMutation {
        source,
        cleanup_cancellation,
    } = &error
    else {
        panic!("ordinary configure failure must preserve the real scene source");
    };
    assert!(cleanup_cancellation.is_none());
    assert!(matches!(source, SceneError::MissingEntity {
        operation: "update collider for", entity,
    } if *entity == d));
    assert_eq!(error.to_string(), format!("ragdoll spawn failed: {source}"));
    assert_eq!(*removals.lock().unwrap(), [d, c, b, a]);
    assert!(world.contains_entity(skeleton));
    for body in [a, b, c, d] {
        assert!(!world.contains_entity(body));
    }
    assert!(world.get_resource::<RagdollRuntime>().is_none());
}

#[test]
fn ragdoll_ordinary_rejected_cleanup_continues_the_remaining_prefix() {
    let (mut world, skeleton, profile) = ragdoll_cancellation_fixture(&["A", "B", "C", "D"]);
    let body_adds = Arc::new(AtomicUsize::new(0));
    let removals = Arc::new(Mutex::new(Vec::new()));
    let (_, entities) = ragdoll_replacement_world(5, body_adds.clone(), removals.clone());
    let [a, b, c, d] = [entities[1], entities[2], entities[3], entities[4]];
    observe_ragdoll_body_events(&mut world, body_adds, removals.clone());
    world.observe_component_lifecycle::<RigidBodyComponent>(
        LifecycleEventKind::Add,
        move |world, event| {
            if event.entity() == d {
                // B is already absent when reverse-prefix cleanup reaches it.
                drop(world.remove_entity_recursive(b).unwrap());
                drop(world.remove_entity_recursive(d).unwrap());
            }
        },
    );
    let error = profile.spawn(&mut world, skeleton).unwrap_err();
    assert_eq!(error.lifecycle_target_change(), None);
    assert!(matches!(&error, RagdollSpawnError::SceneMutation {
        source: SceneError::MissingEntity { entity, .. },
        cleanup_cancellation: None,
    } if *entity == d));
    assert_eq!(*removals.lock().unwrap(), [b, d, c, a]);
    for body in [a, b, c, d] {
        assert!(!world.contains_entity(body));
    }
    assert!(world.contains_entity(skeleton));
}
