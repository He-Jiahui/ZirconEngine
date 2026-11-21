use crate::{BodyHandle, DefaultPhysicsManager, PhysicsBackend, PhysicsBodyCommand};
use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsBodyType, PhysicsColliderShape, PhysicsManager, PhysicsQueryFilter,
    PhysicsQueryMode, PhysicsRayCastHit, PhysicsRayCastQuery, PhysicsSettings,
    PhysicsSimulationMode, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::scene::world::World;
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata};

const EPS: Real = 1e-4;

#[test]
fn translated_static_teleport_carries_the_collider_point() {
    run_probe(Probe::Shift);
}

#[test]
fn rotated_static_teleport_carries_the_collider_point_and_basis() {
    run_probe(Probe::Yaw);
}

#[test]
fn static_tick_without_teleport_keeps_collider_placement() {
    run_probe(Probe::NoCommand);
}

#[test]
fn coincident_static_collider_keeps_teleport_behavior() {
    run_probe(Probe::Coincident);
}

#[test]
fn changed_body_scale_keeps_existing_snapshot_policy() {
    run_probe(Probe::ChangedScale);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Probe {
    Shift,
    Yaw,
    NoCommand,
    Coincident,
    ChangedScale,
}

fn run_probe(probe: Probe) {
    let manager = DefaultPhysicsManager::default();
    manager
        .store_settings(PhysicsSettings {
            backend: "jolt".into(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let body_before = Transform::from_translation(Vec3::new(3.0, 2.0, 4.0));
    world.update_transform(entity, body_before).unwrap();
    world
        .set_rigid_body(
            entity,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Static,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    let local = if probe == Probe::Coincident {
        Transform::identity()
    } else {
        Transform {
            translation: Vec3::X * 8.0,
            rotation: Quat::from_rotation_x(0.25),
            scale: Vec3::new(1.5, 0.75, 1.25),
        }
    };
    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: 0.1 },
                local_transform: local,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();
    let levels = DefaultLevelManager::default();
    let level = levels
        .try_create_level(world, LevelMetadata::default())
        .unwrap();
    let handle = level.world_handle();
    assert!(levels.level(handle).is_some());
    assert!(manager.synchronized_world(handle).is_none());
    let queued = probe != Probe::NoCommand;
    let old_center = if probe == Probe::Coincident {
        Vec3::new(3.0, 2.0, 4.0)
    } else {
        Vec3::new(11.0, 2.0, 4.0)
    };
    let target = if queued {
        Transform {
            translation: Vec3::new(8.0, 2.0, 5.0),
            rotation: if probe == Probe::Shift {
                Quat::IDENTITY
            } else {
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            },
            scale: if probe == Probe::ChangedScale {
                Vec3::splat(2.0)
            } else {
                Vec3::ONE
            },
        }
    } else {
        body_before
    };
    let initial = level.with_world_mut(|world| manager.tick_scene_world(handle, world, 0.0));
    assert_eq!(initial.step_plan.steps, 0);
    let before = manager.synchronized_world(handle).unwrap();
    assert_eq!(before.world, handle);
    let native_body = static_native_handle(&manager, &before, entity);
    assert_eq!(before.bodies[0].transform, body_before);
    let initial_collider = before.colliders[0].transform;
    assert!(initial_collider.translation.distance(old_center) < EPS);
    assert!(rotation_error(initial_collider.rotation, local.rotation) < EPS);
    assert!(initial_collider.scale.distance(local.scale) < EPS);
    check_hit(
        &rays(&manager, handle, old_center)[..],
        entity,
        old_center,
        0.1 * local.scale.max_element(),
    );
    if queued {
        assert!(rays(&manager, handle, target.translation).is_empty());
    }
    let command = PhysicsBodyCommand::Teleport {
        world: handle,
        entity,
        transform: target,
    };
    if queued {
        manager.queue_body_command(command).unwrap();
    }
    let zero = level.with_world_mut(|world| manager.tick_scene_world(handle, world, 0.0));
    assert_eq!(zero.step_plan.steps, 0);
    let pending = manager.synchronized_world(handle).unwrap();
    assert_eq!(pending.bodies, before.bodies);
    assert_eq!(pending.colliders, before.colliders);
    assert_eq!(
        static_native_handle(&manager, &pending, entity),
        native_body
    );
    assert_eq!(
        manager
            .body_commands
            .lock()
            .unwrap()
            .get(&handle)
            .map(Vec::as_slice),
        queued.then_some(std::slice::from_ref(&command))
    );
    let dt = 1.0 / 60.0;
    let result = level.with_world_mut(|world| manager.tick_scene_world(handle, world, dt));
    assert_eq!(result.step_plan.steps, 1);
    assert_eq!(result.step_plan.step_seconds, dt);
    assert!(!manager.body_commands.lock().unwrap().contains_key(&handle));
    let snapshot = manager.synchronized_world(handle).unwrap();
    assert_eq!(snapshot.world, handle);
    assert_eq!(
        static_native_handle(&manager, &snapshot, entity),
        native_body
    );
    // The inactive row carries the admitted target, not a native static pose readout.
    assert_eq!(snapshot.bodies[0].transform, target);
    let published = snapshot.colliders[0].transform;
    level.with_world(|world| {
        assert_eq!(world.local_transform(entity), Some(body_before));
        assert_eq!(world.collider(entity).unwrap().local_transform, local);
    });

    let expected = match probe {
        Probe::Shift => Vec3::new(16.0, 2.0, 5.0),
        Probe::Yaw => Vec3::new(8.0, 2.0, -3.0),
        Probe::NoCommand => old_center,
        Probe::Coincident | Probe::ChangedScale => target.translation,
    };
    // This legacy control intentionally does not define a new native scaling contract.
    let (expected_rotation, expected_scale) = if probe == Probe::ChangedScale {
        (target.rotation, target.scale)
    } else {
        (target.rotation * local.rotation, local.scale)
    };
    let center_error = published.translation.distance(expected);
    let orientation_error = rotation_error(published.rotation, expected_rotation);
    let scale_error = published.scale.distance(expected_scale);
    let radius = 0.1 * published.scale.abs().max_element();
    let expected_radius = 0.1 * expected_scale.abs().max_element();
    let actual = rays(&manager, handle, published.translation);
    check_hit(&actual, entity, published.translation, radius);
    assert!(rays(&manager, handle, published.translation + Vec3::X).is_empty());
    let mut excluded = ray(handle, published.translation);
    excluded.filter.excluded_entities.push(entity);
    assert!(PhysicsManager::ray_cast(&manager, &excluded).is_empty());
    let counts = [expected, target.translation, old_center].map(|point| {
        let hits = rays(&manager, handle, point);
        if !hits.is_empty() {
            check_hit(&hits, entity, published.translation, radius);
        }
        hits.len()
    });
    let expected_counts = match probe {
        Probe::Shift | Probe::Yaw => [1, 0, 0],
        Probe::NoCommand => [1, 0, 1],
        Probe::Coincident | Probe::ChangedScale => [1, 1, 0],
    };
    println!(
        "JOLT_TELEPORT probe={probe:?} queued={queued} zero_steps=0 positive_steps=1 active=false center_error={center_error} rotation_error={orientation_error} scale_error={scale_error} radius={radius} hits={counts:?} body={:?} before={:?} published={:?} expected={:?} drained=true controls=pass",
        target.translation.to_array(), old_center.to_array(), published.translation.to_array(), expected.to_array(),
    );
    assert!(
        center_error < EPS && orientation_error < EPS && scale_error < EPS
            && (radius - expected_radius).abs() < EPS && counts == expected_counts,
        "queued same-scale transform must retain the distinct collider frame (legacy controls must stay unchanged): {probe:?} [{center_error}, {orientation_error}, {scale_error}], {counts:?}",
    );
}

fn static_native_handle(
    manager: &DefaultPhysicsManager,
    state: &PhysicsWorldSyncState,
    entity: EntityId,
) -> BodyHandle {
    let status = PhysicsManager::backend_status(manager);
    assert_eq!(
        (
            status.active_backend.as_deref(),
            status.state,
            status.simulation_mode
        ),
        (
            Some("jolt"),
            PhysicsBackendState::Ready,
            PhysicsSimulationMode::Simulate
        ),
    );
    assert_eq!((state.bodies.len(), state.colliders.len()), (1, 1));
    let body = &state.bodies[0];
    let collider = &state.colliders[0];
    assert_eq!((body.entity, collider.entity), (entity, entity));
    assert_eq!(body.body_type, PhysicsBodyType::Static);
    assert_eq!(collider.shape, PhysicsColliderShape::Sphere { radius: 0.1 });
    assert_eq!(
        (collider.layer, collider.collision_group, collider.sensor),
        (0, 0, false)
    );
    let mut worlds = manager.jolt_worlds.lock().unwrap();
    let managed = worlds
        .get_mut(&state.world)
        .expect("actual native Jolt world");
    assert_eq!(managed.entities.len(), 1);
    let record = &managed.entities[&entity];
    assert_eq!(&record.last_body, body);
    assert_eq!(&record.last_collider, collider);
    let body_handle = record.body;
    let mut active = Vec::new();
    managed.backend.read_active_states(&mut active);
    assert!(
        active.is_empty(),
        "static case must not reach active collider readback"
    );
    body_handle
}

fn rotation_error(actual: Quat, expected: Quat) -> Real {
    (1.0 - actual.dot(expected).abs()).abs()
}

fn ray(world: WorldHandle, center: Vec3) -> PhysicsRayCastQuery {
    PhysicsRayCastQuery {
        world,
        origin: (center + Vec3::Y * 3.0).to_array(),
        direction: [0.0, -2.0, 0.0],
        max_distance: 6.0,
        mode: PhysicsQueryMode::Closest,
        filter: PhysicsQueryFilter::default(),
    }
}

fn rays(
    manager: &DefaultPhysicsManager,
    world: WorldHandle,
    center: Vec3,
) -> Vec<PhysicsRayCastHit> {
    PhysicsManager::ray_cast(manager, &ray(world, center))
}

fn check_hit(hits: &[PhysicsRayCastHit], entity: EntityId, center: Vec3, radius: Real) {
    assert_eq!(
        hits.len(),
        1,
        "same-world snapshot sphere must be queryable"
    );
    let hit = &hits[0];
    assert_eq!(hit.entity, entity);
    assert!((hit.distance - (3.0 - radius)).abs() < EPS);
    assert!(Vec3::from_array(hit.position).distance(center + Vec3::Y * radius) < EPS);
    assert!(Vec3::from_array(hit.normal).distance(Vec3::Y) < EPS);
}
