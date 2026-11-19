use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use zircon_plugin_physics_runtime::{
    build_world_sync_state, DefaultPhysicsManager, PhysicsQueryInterface,
};
use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsColliderShape, PhysicsManager, PhysicsQueryFilter,
    PhysicsQueryMode, PhysicsSettings, PhysicsShapeCastHit, PhysicsShapeCastQuery,
    PhysicsSimulationMode,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};
use zircon_runtime::scene::components::{ColliderComponent, ColliderShape, NodeKind};
use zircon_runtime::scene::world::World;

const WORLD: WorldHandle = WorldHandle::new(1);
const BOX_HALF: [Real; 3] = [2.0, 0.25, 0.25];
const OBLIQUE_HALF: [Real; 3] = [2.0, 0.25, 0.125];
const SPHERE_RADIUS: Real = 0.1;
const SPHERE: PhysicsColliderShape = PhysicsColliderShape::Sphere {
    radius: SPHERE_RADIUS,
};
const EPS: Real = 1.0e-4;

#[test]
fn rotated_sweep_control_identity_target_box() {
    let target = long_box_target(Quat::IDENTITY);
    target.expect_hit(
        "identity target, centered",
        &sphere_along_x(0.0, 0.0, 3.0),
        0.90,
    );
    target.expect_miss("identity target, offset", &sphere_along_x(1.5, 0.0, 3.0));
}

#[test]
fn rotated_sweep_control_identity_query_box() {
    let target = offset_sphere_target();
    target.expect_hit(
        "identity query, centered",
        &long_box_sweep(1.5, Quat::IDENTITY),
        0.90,
    );
    target.expect_miss(
        "identity query, offset",
        &long_box_sweep(0.0, Quat::IDENTITY),
    );
}

#[test]
fn rotated_sweep_control_short_distance() {
    let rotation = Quat::from_rotation_z(FRAC_PI_2);
    let target = long_box_target(rotation);
    // Rz leaves Z unchanged, so even the old sweep must hit this line at 2.65.
    let mut axial = sweep(
        SPHERE,
        Transform::from_translation(-3.0 * Vec3::Z),
        Vec3::Z,
        3.0,
    );
    target.expect_hit("right-angle target, full axial Z", &axial, 2.65);
    axial.max_distance = 2.50;
    target.expect_miss("right-angle target, short axial Z", &axial);
    target.expect_miss(
        "right-angle target, short offset X",
        &sphere_along_x(1.5, 0.0, 2.50),
    );
    let mut query = long_box_sweep(0.0, rotation);
    query.max_distance = 2.50;
    offset_sphere_target().expect_miss("right-angle query, short offset X", &query);
}

#[test]
fn rotated_sweep_control_unpublished_world() {
    let target = long_box_target(Quat::IDENTITY);
    let mut query = sphere_along_x(0.0, 0.0, 3.0);
    target.expect_hit("published world", &query, 0.90);
    query.world = WorldHandle::new(2);
    assert!(target.manager.synchronized_world(query.world).is_none());
    target.expect_miss("unpublished world", &query);
}

#[test]
fn rotated_sweep_control_entity_filter() {
    let target = long_box_target(Quat::IDENTITY);
    let mut query = sphere_along_x(0.0, 0.0, 3.0);
    target.expect_hit("entity admitted", &query, 0.90);
    query.filter.excluded_entities.push(target.entity);
    target.expect_miss("same hittable entity excluded", &query);
}

#[test]
fn rotated_sweep_control_right_angle_outside_bounds() {
    let target = long_box_target(Quat::from_rotation_z(FRAC_PI_2));
    target.expect_miss(
        "right-angle target, beyond Y",
        &sphere_along_x(2.30, 0.0, 3.0),
    );
    target.expect_miss(
        "right-angle target, beyond Z",
        &sphere_along_x(1.5, 0.60, 3.0),
    );
}

#[test]
fn rotated_sweep_control_oblique_short_and_off_axis() {
    let target = oblique_box_target();
    target.expect_miss("45-degree target, short X", &sphere_along_x(1.0, 0.0, 1.25));
    target.expect_miss(
        "45-degree target, beyond Z",
        &sphere_along_x(1.0, 0.55, 5.0),
    );
}

#[test]
fn rotated_sweep_control_zero_distance_initial_overlap() {
    let target = long_box_target(Quat::IDENTITY);
    let mut query = sphere_along_x(0.0, 0.0, 0.0);
    target.expect_miss("zero distance without initial overlap", &query);
    query.origin_transform.translation.x = 0.125;
    let normal = target.expect_hit("zero distance with initial overlap", &query, 0.0);
    assert_eq!(normal, [0.0; 3]);
}

#[test]
fn rotated_sweep_control_finite_nonunit_query_rotation() {
    let target = offset_sphere_target();
    // These finite DTOs reached the old Box fallback. They are not authored
    // World rotations or a contract for arbitrary nonunit orientations.
    for (case, rotation) in [
        (
            "scalar-only nonunit query q",
            Quat::from_array([0.0, 0.0, 0.0, 2.0]),
        ),
        ("zero query q", Quat::from_array([0.0; 4])),
    ] {
        target.expect_hit(case, &long_box_sweep(1.5, rotation), 0.90);
    }
}

#[test]
fn rotated_sweep_target_box_keeps_bound() {
    let target = long_box_target(Quat::from_rotation_z(FRAC_PI_2));
    target.expect_hit(
        "right-angle target Box",
        &sphere_along_x(1.5, 0.0, 3.0),
        2.65,
    );
}

#[test]
fn rotated_sweep_query_box_keeps_bound() {
    let target = offset_sphere_target();
    let rotation = Quat::from_rotation_z(FRAC_PI_2);
    let query = long_box_sweep(0.0, rotation);
    let opposite = long_box_sweep(0.0, -rotation);
    // Log both actual outputs from the same snapshot before either assertion.
    let hits = target.observe("right-angle query Box, q", &query);
    let opposite_hits = target.observe("right-angle query Box, -q", &opposite);
    target.check_hit("right-angle query Box, q", &query, &hits, 2.65);
    target.check_hit("right-angle query Box, -q", &opposite, &opposite_hits, 2.65);
}

#[test]
fn rotated_sweep_oblique_target_box_keeps_bound() {
    let target = oblique_box_target();
    // (1, 1, 0) is inside the rotated Box. The earlier hit is the proxy's
    // X entry: 3 - (2.25 / sqrt(2) + 0.1), not an exact contact time.
    target.expect_hit(
        "45-degree target Box",
        &sphere_along_x(1.0, 0.0, 5.0),
        1.309_01,
    );
}

// HeadlessChaos GJK.GJKRaycastTests -> GJKSphereBoxSweep. These inputs keep
// the moving vs. fixed geometry, direction and length. ZE reports the moving center.
#[test]
fn rotated_sweep_ue_ordinary_sphere_hits_box() {
    let pose = Transform::from_translation(Vec3::new(3.5, 0.0, 2.0));
    let target = PublishedCollider::box_at([0.5, 1.0, 2.0], pose);
    let sphere = PhysicsColliderShape::Sphere { radius: 1.0 };
    let query = sweep(sphere, Transform::identity(), Vec3::X, 4.0);
    let normal = target.expect_hit("UE ordinary sphere against Box", &query, 2.0);
    assert_eq!(normal, [-1.0, 0.0, 0.0]);
}

#[test]
fn rotated_sweep_ue_rotated_box_keeps_contact_distance() {
    expect_ue_rotated_box("UE rotated Box against sphere", [0.5, 1.0, 2.0], 2.0);
}

#[test]
fn rotated_sweep_ue_rotated_needle_keeps_contact_distance() {
    expect_ue_rotated_box("UE rotated needle against sphere", [0.5, 0.0, 0.0], 0.0);
}

fn expect_ue_rotated_box(case: &str, half_extents: [Real; 3], center_z: Real) {
    let target = PublishedCollider::sphere_at(1.0, Vec3::ZERO);
    // UE rotates the off-center Box around the origin. Its local center
    // (3.5, 0, z) becomes the ZE start center (0, 3.5, z).
    let pose = Transform::from_translation(Vec3::new(0.0, 3.5, center_z))
        .with_rotation(Quat::from_rotation_z(FRAC_PI_2));
    let shape = PhysicsColliderShape::Box { half_extents };
    let normal = target.expect_hit(case, &sweep(shape, pose, -Vec3::Y, 4.0), 2.0);
    assert_eq!(normal, [0.0, 1.0, 0.0], "{case}: axis normal");
}

fn long_box_target(rotation: Quat) -> PublishedCollider {
    PublishedCollider::box_at(BOX_HALF, Transform::identity().with_rotation(rotation))
}

fn oblique_box_target() -> PublishedCollider {
    let pose = Transform::identity().with_rotation(Quat::from_rotation_z(FRAC_PI_4));
    PublishedCollider::box_at(OBLIQUE_HALF, pose)
}

fn offset_sphere_target() -> PublishedCollider {
    PublishedCollider::sphere_at(SPHERE_RADIUS, Vec3::new(0.0, 1.5, 0.0))
}

fn long_box_sweep(y: Real, rotation: Quat) -> PhysicsShapeCastQuery {
    let shape = PhysicsColliderShape::Box {
        half_extents: BOX_HALF,
    };
    let pose = Transform::from_translation(Vec3::new(-3.0, y, 0.0)).with_rotation(rotation);
    sweep(shape, pose, Vec3::X, 3.0)
}

fn sphere_along_x(y: Real, z: Real, max_distance: Real) -> PhysicsShapeCastQuery {
    let pose = Transform::from_translation(Vec3::new(-3.0, y, z));
    sweep(SPHERE, pose, Vec3::X, max_distance)
}

fn sweep(
    shape: PhysicsColliderShape,
    origin_transform: Transform,
    direction: Vec3,
    max_distance: Real,
) -> PhysicsShapeCastQuery {
    PhysicsShapeCastQuery {
        world: WORLD,
        shape,
        origin_transform,
        direction: direction.to_array(),
        max_distance,
        mode: PhysicsQueryMode::Closest,
        filter: PhysicsQueryFilter::default(),
    }
}

struct PublishedCollider {
    manager: DefaultPhysicsManager,
    entity: EntityId,
}

impl PublishedCollider {
    fn box_at(half_extents: [Real; 3], pose: Transform) -> Self {
        let shape = ColliderShape::Box {
            half_extents: Vec3::from_array(half_extents),
        };
        Self::publish(shape, PhysicsColliderShape::Box { half_extents }, pose)
    }

    fn sphere_at(radius: Real, center: Vec3) -> Self {
        let shape = ColliderShape::Sphere { radius };
        Self::publish(
            shape,
            PhysicsColliderShape::Sphere { radius },
            Transform::from_translation(center),
        )
    }

    fn publish(shape: ColliderShape, expected: PhysicsColliderShape, pose: Transform) -> Self {
        let mut world = World::empty();
        let entity = world
            .spawn_node(NodeKind::Empty)
            .expect("spawn root collider node");
        world
            .update_transform(entity, pose)
            .expect("author root pose");
        let collider = ColliderComponent {
            shape,
            ..ColliderComponent::default()
        };
        assert!(world
            .set_collider(entity, Some(collider))
            .expect("author root collider"));
        assert_eq!(world.parent_of(entity), None);
        assert!(world.rigid_body(entity).is_none());
        let world_pose = world.world_transform(entity).expect("read root world pose");
        assert_pose("authored World pose", world_pose, pose);

        let manager = DefaultPhysicsManager::new(None);
        let settings = PhysicsSettings {
            backend: "builtin".into(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            ..PhysicsSettings::default()
        };
        manager
            .store_settings(settings)
            .expect("select Builtin QueryOnly");
        PhysicsManager::sync_world(&manager, build_world_sync_state(WORLD, &world));
        let status = manager.backend_status();
        assert_eq!(status.state, PhysicsBackendState::Ready, "{status:?}");
        assert_eq!(status.active_backend.as_deref(), Some("builtin"));
        assert_eq!(status.simulation_mode, PhysicsSimulationMode::QueryOnly);
        let published = manager
            .synchronized_world(WORLD)
            .expect("read published query world");
        assert_eq!(published.world, WORLD);
        assert!(published.bodies.is_empty());
        let [row] = published.colliders.as_slice() else {
            panic!(
                "expected one published collider, got {:?}",
                published.colliders
            );
        };
        println!("published world={WORLD:?}: {row:?}");
        assert_eq!(row.entity, entity);
        assert_eq!(row.shape, expected);
        assert!(!row.sensor);
        assert_pose("published collider pose", row.transform, pose);
        Self { manager, entity }
    }

    fn observe(&self, case: &str, query: &PhysicsShapeCastQuery) -> Vec<PhysicsShapeCastHit> {
        let query_api: &dyn PhysicsQueryInterface = &self.manager;
        let hits = query_api.shape_cast(query);
        let first = hits.first();
        println!(
            "{case}: hit={} count={} entity={:?} distance={:?} position={:?} normal={:?}; \
             request={query:?}",
            first.is_some(),
            hits.len(),
            first.map(|hit| hit.entity),
            first.map(|hit| hit.distance),
            first.map(|hit| hit.position),
            first.map(|hit| hit.normal),
        );
        hits
    }

    fn expect_hit(&self, case: &str, query: &PhysicsShapeCastQuery, distance: Real) -> [Real; 3] {
        let hits = self.observe(case, query);
        self.check_hit(case, query, &hits, distance)
    }

    fn check_hit(
        &self,
        case: &str,
        query: &PhysicsShapeCastQuery,
        hits: &[PhysicsShapeCastHit],
        distance: Real,
    ) -> [Real; 3] {
        let [hit] = hits else {
            panic!(
                "{case}: expected one hit on {} at {distance}, got {hits:?}",
                self.entity
            );
        };
        assert_eq!(hit.entity, self.entity, "{case}: target entity");
        assert!(
            (hit.distance - distance).abs() <= EPS,
            "{case}: expected distance {distance}, got {}",
            hit.distance
        );
        // These requests have unit axis directions. Use expected distance, not the hit's.
        let center =
            query.origin_transform.translation + Vec3::from_array(query.direction) * distance;
        assert_near(case, Vec3::from_array(hit.position), center);
        hit.normal
    }

    fn expect_miss(&self, case: &str, query: &PhysicsShapeCastQuery) {
        let hits = self.observe(case, query);
        assert!(hits.is_empty(), "{case}: expected no hit, got {hits:?}");
    }
}

fn assert_pose(label: &str, actual: Transform, expected: Transform) {
    assert_near(label, actual.translation, expected.translation);
    assert_near(label, actual.scale, Vec3::ONE);
    for axis in [Vec3::X, Vec3::Y, Vec3::Z] {
        assert_near(label, actual.rotation * axis, expected.rotation * axis);
    }
}

fn assert_near(label: &str, actual: Vec3, expected: Vec3) {
    assert!(
        (actual - expected).length() <= EPS,
        "{label}: expected {expected:?}, got {actual:?}"
    );
}
