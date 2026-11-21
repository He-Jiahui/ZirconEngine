use super::*;
use zircon_runtime::core::framework::physics::{
    PhysicsBodyType, PhysicsColliderShape, PhysicsJointType,
};
use zircon_runtime::core::math::Transform;

use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsManager, PhysicsQueryFilter, PhysicsQueryMode, PhysicsRayCastHit,
    PhysicsRayCastQuery, PhysicsSimulationMode,
};
use zircon_runtime::core::framework::scene::physics::PhysicsSleepPolicy;
use zircon_runtime::core::math::{Quat, Vec3};
use zircon_runtime::scene::components::{
    ColliderComponent, ColliderShape, NodeKind, RigidBodyComponent, RigidBodyType,
};
use zircon_runtime::scene::world::World;

#[test]
fn joint_resolves_entity_pair_to_handles() {
    let mut world = JoltManagedWorld::new(PhysicsSettings::default()).expect("Jolt world");
    let world_handle = WorldHandle::new(41);
    let material = PhysicsMaterialMetadata::default();
    for entity in [411, 412] {
        let transform = Transform::default();
        let body = PhysicsBodySyncState {
            entity,
            body_type: PhysicsBodyType::Dynamic,
            transform,
            mass: 1.0,
            mass_properties: Default::default(),
            linear_velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 0.0,
            ccd_mode: Default::default(),
            sleep_policy: Default::default(),
            lock_translation: [false; 3],
            lock_rotation: [false; 3],
        };
        let collider = PhysicsColliderSyncState {
            entity,
            shape: PhysicsColliderShape::Sphere { radius: 0.25 },
            sensor: false,
            layer: 0,
            collision_group: 0,
            collision_mask: u32::MAX,
            material: None,
            material_override: None,
            transform,
        };
        world
            .synchronize_entity(world_handle, entity, &body, &collider, &material)
            .expect("synchronize body");
    }
    let joint = PhysicsJointSyncState {
        entity: 411,
        kind: PhysicsJointType::Fixed,
        connected_entity: Some(412),
        anchor: [0.0; 3],
        axis: [0.0, 1.0, 0.0],
        limits: None,
        collide_connected: false,
        constraint: Default::default(),
        skeleton_binding: None,
    };

    let (body_a, body_b) = world
        .resolved_constraint_handles(&joint)
        .expect("resolve joint body pair");
    assert_eq!(body_a, world.entities[&411].body);
    assert_eq!(body_b, Some(world.entities[&412].body));
}

#[test]
fn query_only_active_collider_keeps_separate_snapshot_pose() {
    let row = snapshot_probe("query_only", SnapshotProbe::ActiveOffset);
    assert!(
        row.correct(false, false),
        "published query pose must not collapse to the active body: {row:?}"
    );
}

#[test]
fn stepped_parented_collider_follows_native_rigid_motion() {
    let row = snapshot_probe("moving", SnapshotProbe::MovingOffset);
    assert!(
        row.correct(true, false),
        "published collider point and orientation must follow native rigid motion: {row:?}"
    );
}

#[test]
fn inactive_static_collider_keeps_existing_query_pose() {
    let row = snapshot_probe("inactive", SnapshotProbe::StaticOffset);
    assert!(
        row.correct(false, false),
        "inactive query placement must stay unchanged: {row:?}"
    );
}

#[test]
fn coincident_active_collider_keeps_existing_query_pose() {
    let row = snapshot_probe("coincident", SnapshotProbe::ActiveCoincident);
    assert!(
        row.correct(false, true),
        "coincident query placement must stay unchanged: {row:?}"
    );
}

#[derive(Clone, Copy)]
enum SnapshotProbe {
    ActiveOffset,
    MovingOffset,
    StaticOffset,
    ActiveCoincident,
}

#[derive(Debug)]
struct SnapshotRow {
    center_error: Real,
    rotation_error: Real,
    scale_error: Real,
    radius: Real,
    exact_hits: usize,
    origin_hits: usize,
    stale_hits: usize,
}

impl SnapshotRow {
    fn correct(&self, moving: bool, coincident: bool) -> bool {
        self.center_error < 1e-4
            && self.rotation_error < 1e-4
            && self.scale_error < 1e-4
            && (self.radius - if coincident { 0.1 } else { 0.15 }).abs() < 1e-4
            && self.exact_hits == 1
            && self.origin_hits == usize::from(coincident)
            && self.stale_hits == usize::from(!moving)
    }
}

fn snapshot_probe(label: &str, probe: SnapshotProbe) -> SnapshotRow {
    const EPS: Real = 1e-4;
    let moving = matches!(probe, SnapshotProbe::MovingOffset);
    let inactive = matches!(probe, SnapshotProbe::StaticOffset);
    let coincident = matches!(probe, SnapshotProbe::ActiveCoincident);
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let (authored, initial_origin, initial_center) = if moving {
        let parent = world.spawn_node(NodeKind::Empty).unwrap();
        world
            .update_transform(
                parent,
                Transform {
                    translation: Vec3::new(2.0, 3.0, 4.0),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ..Transform::identity()
                },
            )
            .unwrap();
        world.set_parent_checked(entity, Some(parent)).unwrap();
        (
            Transform::from_translation(Vec3::X),
            Vec3::new(2.0, 3.0, 3.0),
            Vec3::new(2.0, 3.0, -5.0),
        )
    } else {
        let initial_origin = Vec3::new(3.0, 2.0, 4.0);
        let initial_center = if coincident {
            initial_origin
        } else {
            Vec3::new(11.0, 2.0, 4.0)
        };
        (
            Transform::from_translation(initial_origin),
            initial_origin,
            initial_center,
        )
    };
    world.update_transform(entity, authored).unwrap();
    world
        .set_rigid_body(
            entity,
            Some(RigidBodyComponent {
                body_type: if inactive {
                    RigidBodyType::Static
                } else {
                    RigidBodyType::Dynamic
                },
                gravity_scale: 0.0,
                linear_damping: 0.0,
                angular_damping: 0.0,
                linear_velocity: if moving { Vec3::X * 8.0 } else { Vec3::ZERO },
                angular_velocity: if moving { Vec3::Y * 4.0 } else { Vec3::ZERO },
                sleep_policy: PhysicsSleepPolicy::Never,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    let local = if coincident {
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
    let initial_matrix = world.world_matrix(entity).unwrap();
    assert!(
        initial_matrix
            .transform_point3(local.translation)
            .distance(initial_center)
            < EPS
    );
    let handle = WorldHandle::new(961);
    let pre = crate::build_world_sync_state(handle, &world);
    assert_eq!((pre.bodies.len(), pre.colliders.len()), (1, 1));
    let before = pre.bodies[0].transform;
    assert!(before.translation.distance(initial_origin) < EPS);
    assert!(
        pre.colliders[0]
            .transform
            .translation
            .distance(initial_center)
            < EPS
    );
    assert!(pre.colliders[0].transform.scale.distance(local.scale) < EPS);

    let manager = DefaultPhysicsManager::default();
    let mode = if moving {
        PhysicsSimulationMode::Simulate
    } else {
        PhysicsSimulationMode::QueryOnly
    };
    manager
        .store_settings(PhysicsSettings {
            backend: "jolt".into(),
            simulation_mode: mode,
            ..PhysicsSettings::default()
        })
        .unwrap();
    assert!(manager.synchronized_world(handle).is_none());
    let result = manager.tick_scene_world(handle, &mut world, if moving { 0.25 } else { 0.0 });
    assert_eq!(result.step_plan.steps, u32::from(moving));
    if moving {
        assert_eq!(result.step_plan.step_seconds, 0.25);
    }
    let status = PhysicsManager::backend_status(&manager);
    assert_eq!(
        (
            status.active_backend.as_deref(),
            status.state,
            status.simulation_mode
        ),
        (Some("jolt"), PhysicsBackendState::Ready, mode)
    );
    let snapshot = manager
        .synchronized_world(handle)
        .expect("linked Jolt must publish the real same-world snapshot");
    assert_eq!(snapshot.world, handle);
    assert_eq!((snapshot.bodies.len(), snapshot.colliders.len()), (1, 1));
    let body = &snapshot.bodies[0];
    let collider = &snapshot.colliders[0];
    assert_eq!((body.entity, collider.entity), (entity, entity));
    assert_eq!(collider.shape, PhysicsColliderShape::Sphere { radius: 0.1 });
    assert_eq!(
        (collider.layer, collider.collision_group, collider.sensor),
        (0, 0, false)
    );
    assert_eq!(world.local_transform(entity), Some(authored));
    assert_eq!(world.world_matrix(entity), Some(initial_matrix));
    assert_eq!(world.collider(entity).unwrap().local_transform, local);
    let pose = body.transform;
    let active = {
        // Check actual native activity and pose, not the settings bit or an empty result.
        let mut worlds = recover_lock(&manager.jolt_worlds);
        let managed = worlds
            .get_mut(&handle)
            .expect("native managed world exists");
        let record = &managed.entities[&entity];
        assert_eq!(record.last_collider.transform, collider.transform);
        let native_handle = record.body;
        let mut native = Vec::new();
        managed.backend.read_active_states(&mut native);
        if inactive {
            assert!(native.is_empty());
            assert_eq!(body.body_type, PhysicsBodyType::Static);
            false
        } else {
            assert_eq!(native.len(), 1);
            assert_eq!(native[0].0, native_handle);
            assert_eq!(&native[0].1, body);
            true
        }
    };
    let body_delta = pose.translation.distance(before.translation);
    let rotation_delta = (1.0 - pose.rotation.dot(before.rotation).abs()).abs();
    let expected = pose.translation + pose.rotation * local.translation;
    let expected_rotation = pose.rotation * local.rotation;
    if moving {
        assert!(pose.translation.distance(initial_origin + Vec3::X * 2.0) < EPS);
        assert!(body_delta > 1.0 && rotation_delta > 0.02);
        assert!(expected.distance(initial_center) > 1.0);
    } else {
        assert!(body_delta < EPS && rotation_delta < EPS);
        assert!(expected.distance(initial_center) < EPS);
    }
    if !coincident {
        assert!(expected.distance(pose.translation) > 7.99);
    }
    let actual = collider.transform.translation;
    let radius = 0.1 * collider.transform.scale.abs().max_element();
    let exact = PhysicsManager::ray_cast(&manager, &snapshot_ray(handle, expected));
    let origin = PhysicsManager::ray_cast(&manager, &snapshot_ray(handle, pose.translation));
    let stale = PhysicsManager::ray_cast(&manager, &snapshot_ray(handle, initial_center));
    let at_actual = PhysicsManager::ray_cast(&manager, &snapshot_ray(handle, actual));
    assert_eq!(
        at_actual.len(),
        1,
        "same-world synchronized sphere must really be queryable"
    );
    snapshot_hit(&at_actual[0], entity, actual, radius);
    for hits in [&exact, &origin, &stale] {
        assert!(hits.len() <= 1);
        if let Some(hit) = hits.first() {
            snapshot_hit(hit, entity, actual, radius);
        }
    }
    let mut excluded = snapshot_ray(handle, actual);
    excluded.filter.excluded_entities.push(entity);
    assert!(PhysicsManager::ray_cast(&manager, &excluded).is_empty());
    assert!(PhysicsManager::ray_cast(&manager, &snapshot_ray(handle, actual + Vec3::X)).is_empty());
    let row = SnapshotRow {
        center_error: actual.distance(expected),
        rotation_error: (1.0 - collider.transform.rotation.dot(expected_rotation).abs()).abs(),
        scale_error: collider.transform.scale.distance(local.scale),
        radius,
        exact_hits: exact.len(),
        origin_hits: origin.len(),
        stale_hits: stale.len(),
    };
    println!("JOLT_SNAPSHOT label={label} active={active} steps={} body_delta={body_delta} rotation_delta={rotation_delta} center_error={} rotation_error={} scale_error={} radius={} exact={} origin={} stale={} native_body={:?} published={:?} expected={:?} cached=true filters=pass", result.step_plan.steps, row.center_error, row.rotation_error, row.scale_error, row.radius, row.exact_hits, row.origin_hits, row.stale_hits, pose.translation.to_array(), actual.to_array(), expected.to_array());
    row
}

fn snapshot_ray(world: WorldHandle, center: Vec3) -> PhysicsRayCastQuery {
    PhysicsRayCastQuery {
        world,
        origin: (center + Vec3::Y * 3.0).to_array(),
        direction: [0.0, -2.0, 0.0],
        max_distance: 6.0,
        mode: PhysicsQueryMode::Closest,
        filter: PhysicsQueryFilter::default(),
    }
}

fn snapshot_hit(hit: &PhysicsRayCastHit, entity: EntityId, center: Vec3, radius: Real) {
    assert_eq!(hit.entity, entity);
    assert!((hit.distance - (3.0 - radius)).abs() < 1e-4);
    assert!(Vec3::from_array(hit.position).distance(center + Vec3::Y * radius) < 1e-4);
    assert!(Vec3::from_array(hit.normal).distance(Vec3::Y) < 1e-4);
}
