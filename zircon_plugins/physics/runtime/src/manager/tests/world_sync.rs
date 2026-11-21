use super::*;

use zircon_runtime::core::framework::physics::{
    PhysicsManager, PhysicsQueryFilter, PhysicsQueryMode, PhysicsRayCastHit, PhysicsRayCastQuery,
    PhysicsSettings, PhysicsSimulationMode,
};
use zircon_runtime::core::math::{Mat4, Quat};
use zircon_runtime::scene::components::{ColliderComponent, NodeKind, RigidBodyComponent};

use crate::DefaultPhysicsManager;

#[test]
fn owned_projection_preserves_nested_payloads() {
    let shape = ColliderShape::Compound {
        children: vec![(
            Transform::default(),
            Box::new(ColliderShape::ConvexHull {
                points: vec![Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)],
            }),
        )],
    };

    let PhysicsColliderShape::Compound { children } = collider_shape_into_physics(shape) else {
        panic!("owned compound projection must preserve its variant");
    };
    let PhysicsColliderShape::ConvexHull { points } = children[0].1.as_ref() else {
        panic!("owned compound projection must preserve its child variant");
    };
    assert_eq!(points, &[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
}

#[test]
fn typed_world_sync_ignores_ten_thousand_inert_entities() {
    use zircon_runtime::scene::{components::RigidBodyComponent, NodeKind};
    let mut world = World::empty();
    for _ in 0..10_000 {
        world.spawn_node(NodeKind::Empty).unwrap();
    }
    let body = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_rigid_body(body, Some(RigidBodyComponent::default()))
        .unwrap();
    let sync = build_world_sync_state(WorldHandle::new(71), &world);
    assert_eq!(sync.bodies.len(), 1);
    assert_eq!(sync.bodies[0].entity, body);
    assert!(sync.colliders.is_empty());
    assert!(sync.joints.is_empty());
    assert!(sync.materials.is_empty());
}

#[test]
fn sheared_collider_point_drives_actual_synchronized_ray_hits() {
    let row = collider_point_probe("shear", PointPlacement::Shear(2.0), Vec3::X * 8.0);
    assert!(
        row.correct(),
        "affine collider origin must determine real query hits: {row:?}"
    );
}

#[test]
fn mirrored_shear_collider_point_drives_actual_synchronized_ray_hits() {
    let row = collider_point_probe("mirrored", PointPlacement::Shear(-2.0), Vec3::X * 8.0);
    assert!(
        row.correct(),
        "mirrored affine collider origin must determine real query hits: {row:?}"
    );
}

#[test]
fn rigid_collider_offset_keeps_existing_query_geometry() {
    let row = collider_point_probe("rigid", PointPlacement::Rigid, Vec3::X * 8.0);
    assert!(row.correct(), "rigid offset must stay correct: {row:?}");
}

#[test]
fn zero_offset_under_shear_keeps_existing_query_geometry() {
    let row = collider_point_probe("zero", PointPlacement::Shear(2.0), Vec3::ZERO);
    assert!(row.correct(), "zero offset must stay correct: {row:?}");
}

#[derive(Clone, Copy)]
enum PointPlacement {
    Shear(f32),
    Rigid,
}

#[derive(Debug)]
struct ColliderPointRow {
    center_error: f32,
    exact_hits: usize,
    projected_hits: usize,
    separate: bool,
}

impl ColliderPointRow {
    fn correct(&self) -> bool {
        self.center_error < 1e-4
            && self.exact_hits == 1
            && self.projected_hits == usize::from(!self.separate)
    }
}

fn collider_point_probe(
    label: &str,
    placement: PointPlacement,
    local_center: Vec3,
) -> ColliderPointRow {
    const EPS: f32 = 1e-4;
    const RADIUS: f32 = 0.1;
    let mut world = World::empty();
    let mut parents = Vec::new();
    let (entity, expected) = match placement {
        PointPlacement::Shear(sx) => {
            let parent = point_node(
                &mut world,
                Transform {
                    scale: Vec3::new(sx, 1.0, 1.0),
                    ..Transform::identity()
                },
            );
            let entity = point_node(
                &mut world,
                Transform {
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
                    ..Transform::identity()
                },
            );
            world.set_parent_checked(entity, Some(parent)).unwrap();
            let matrix = world.world_matrix(entity).unwrap();
            assert!((matrix.determinant() - sx).abs() < EPS);
            assert!((matrix.x_axis.dot(matrix.z_axis) - 1.5).abs() < EPS);
            parents.push(parent);
            let r = std::f32::consts::FRAC_1_SQRT_2;
            let expected = Vec3::new(
                sx * r * (local_center.x + local_center.z),
                local_center.y,
                r * (-local_center.x + local_center.z),
            );
            (entity, expected)
        }
        PointPlacement::Rigid => {
            let parent = point_node(
                &mut world,
                Transform {
                    translation: Vec3::new(3.0, 2.0, 1.0),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ..Transform::identity()
                },
            );
            let entity = point_node(&mut world, Transform::identity());
            world.set_parent_checked(entity, Some(parent)).unwrap();
            parents.push(parent);
            (
                entity,
                Vec3::new(
                    3.0 + local_center.z,
                    2.0 + local_center.y,
                    1.0 - local_center.x,
                ),
            )
        }
    };
    world
        .set_rigid_body(
            entity,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Static,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    let local = Transform {
        translation: local_center,
        rotation: Quat::from_rotation_z(0.2),
        scale: Vec3::new(1.0, 1.25, 0.75),
    };
    world
        .set_collider(
            entity,
            Some(ColliderComponent {
                shape: ColliderShape::Sphere { radius: RADIUS },
                local_transform: local,
                ..ColliderComponent::default()
            }),
        )
        .unwrap();
    let before = world.world_matrix(entity).unwrap();
    assert!(before.transform_point3(local_center).distance(expected) < EPS);
    let initial_local = world.local_transform(entity).unwrap();
    let ancestors: Vec<(u64, Mat4)> = parents
        .into_iter()
        .map(|p| (p, world.world_matrix(p).unwrap()))
        .collect();
    let compact = combine_transforms(world.world_transform(entity).unwrap(), local);
    let separate = compact.translation.distance(expected) > 1.0;
    if matches!(placement, PointPlacement::Shear(_)) && local_center != Vec3::ZERO {
        assert!(separate, "fixture must expose actual matrix/TRS gap");
    } else {
        assert!(compact.translation.distance(expected) < EPS);
    }
    let manager = DefaultPhysicsManager::default();
    manager
        .store_settings(PhysicsSettings {
            backend: "builtin".into(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            ..PhysicsSettings::default()
        })
        .unwrap();
    assert_eq!(
        PhysicsManager::backend_status(&manager)
            .active_backend
            .as_deref(),
        Some("builtin")
    );
    let handle = WorldHandle::new(801);
    assert!(manager.synchronized_world(handle).is_none());
    let step = manager.tick_scene_world(handle, &mut world, 0.0);
    assert_eq!(step.step_plan.steps, 0);
    assert!(step.contacts.is_empty() && step.triggers.is_empty());
    assert_eq!(world.local_transform(entity), Some(initial_local));
    assert_eq!(world.world_matrix(entity), Some(before));
    for (parent, matrix) in ancestors {
        assert_eq!(world.world_matrix(parent), Some(matrix));
    }
    let snapshot = manager
        .synchronized_world(handle)
        .expect("real World scene tick must publish the query snapshot");
    assert_eq!(snapshot.world, handle);
    assert_eq!((snapshot.bodies.len(), snapshot.colliders.len()), (1, 1));
    let body = &snapshot.bodies[0];
    assert_eq!(body.entity, entity);
    assert_eq!(body.transform, world.world_transform(entity).unwrap());
    let collider = &snapshot.colliders[0];
    assert_eq!(collider.entity, entity);
    assert_eq!(
        collider.shape,
        PhysicsColliderShape::Sphere { radius: RADIUS }
    );
    assert_eq!(
        (collider.transform.rotation, collider.transform.scale),
        (compact.rotation, compact.scale)
    );
    assert_eq!(
        (collider.layer, collider.collision_group, collider.sensor),
        (0, 0, false)
    );
    let radius = RADIUS * collider.transform.scale.abs().max_element();
    assert!(radius.is_finite() && radius > 0.09 && radius < 0.25);
    let exact = PhysicsManager::ray_cast(&manager, &point_query(handle, expected));
    let projected = PhysicsManager::ray_cast(&manager, &point_query(handle, compact.translation));
    // Validate whichever compact sphere is genuinely synchronized, not an assumed backend.
    let at_actual = PhysicsManager::ray_cast(
        &manager,
        &point_query(handle, collider.transform.translation),
    );
    assert_eq!(at_actual.len(), 1);
    point_hit(
        &at_actual[0],
        entity,
        collider.transform.translation,
        radius,
    );
    for hits in [&exact, &projected] {
        assert!(hits.len() <= 1);
        if let Some(hit) = hits.first() {
            point_hit(hit, entity, collider.transform.translation, radius);
        }
    }
    let mut excluded = point_query(handle, collider.transform.translation);
    excluded.filter.excluded_entities.push(entity);
    assert!(PhysicsManager::ray_cast(&manager, &excluded).is_empty());
    let mut masked = point_query(handle, collider.transform.translation);
    masked.filter.collision_mask = Some(0);
    assert!(PhysicsManager::ray_cast(&manager, &masked).is_empty());
    assert!(PhysicsManager::ray_cast(
        &manager,
        &point_query(WorldHandle::new(802), collider.transform.translation)
    )
    .is_empty());
    assert!(PhysicsManager::ray_cast(
        &manager,
        &point_query(handle, collider.transform.translation + Vec3::X * 1.0)
    )
    .is_empty());
    // Repeat the production projection without moving the scene; no new manual sync path.
    let again = manager.tick_scene_world(handle, &mut world, 0.0);
    assert_eq!(again.step_plan.steps, 0);
    assert_eq!(
        manager.synchronized_world(handle).unwrap().as_ref(),
        snapshot.as_ref()
    );
    let row = ColliderPointRow {
        center_error: collider.transform.translation.distance(expected),
        exact_hits: exact.len(),
        projected_hits: projected.len(),
        separate,
    };
    println!("PHYSICS_POINT label={label} expected={:?} projected={:?} synced={:?} error={} radius={} exact={} projected_hits={} actual_distance={} actual_normal={:?} body_same=true shape_same=true filters=pass repeat=true",
        expected.to_array(), compact.translation.to_array(), collider.transform.translation.to_array(), row.center_error, radius,
        row.exact_hits, row.projected_hits, at_actual[0].distance, at_actual[0].normal);
    row
}

fn point_node(world: &mut World, transform: Transform) -> u64 {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world.update_transform(entity, transform).unwrap();
    entity
}

fn point_query(world: WorldHandle, center: Vec3) -> PhysicsRayCastQuery {
    PhysicsRayCastQuery {
        world,
        origin: (center + Vec3::Y * 3.0).to_array(),
        direction: [0.0, -2.0, 0.0],
        max_distance: 6.0,
        mode: PhysicsQueryMode::Closest,
        filter: PhysicsQueryFilter::default(),
    }
}

fn point_hit(hit: &PhysicsRayCastHit, entity: u64, center: Vec3, radius: f32) {
    assert_eq!(hit.entity, entity);
    assert!((hit.distance - (3.0 - radius)).abs() < 1e-4);
    assert!(Vec3::from_array(hit.position).distance(center + Vec3::Y * radius) < 1e-4);
    assert!(Vec3::from_array(hit.normal).distance(Vec3::Y) < 1e-4);
}
