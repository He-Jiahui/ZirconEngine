use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use zircon_runtime::core::framework::physics::{
    PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsColliderSyncState,
    PhysicsManager, PhysicsQueryFilter, PhysicsQueryMode, PhysicsRayCastQuery, PhysicsSettings,
    PhysicsSimulationMode, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::physics::PhysicsSleepPolicy;
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};
use zircon_runtime::core::math::{transform_to_mat4, Quat, Transform, Vec3};
use zircon_runtime::scene::components::{NodeKind, RigidBodyComponent, RigidBodyType};
use zircon_runtime::scene::world::World;

use super::{
    apply_synchronized_bodies_to_scene, build_world_sync_state, DefaultPhysicsManager,
    PhysicsBodyCommand,
};

#[test]
fn physics_manager_settings_and_clock_recover_poisoned_state_locks() {
    let manager = DefaultPhysicsManager::default();
    poison(manager.settings.clone());

    manager
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 60,
            max_substeps: 4,
            ..PhysicsSettings::default()
        })
        .unwrap();

    poison(manager.accumulators.clone());
    poison(manager.body_commands.clone());
    manager
        .queue_body_command(PhysicsBodyCommand::SetLinearVelocity {
            world: WorldHandle::new(7),
            entity: 1,
            velocity: [1.0, 0.0, 0.0],
        })
        .unwrap();
    let plan = manager.advance_clock(WorldHandle::new(7), 1.0 / 60.0);

    assert_eq!(PhysicsManager::settings(&manager).backend, "builtin");
    assert_eq!(plan.steps, 1);
    assert_eq!(manager.drain_body_commands(WorldHandle::new(7)).len(), 1);
}

#[test]
fn physics_manager_world_state_recovers_poisoned_state_locks() {
    let manager = DefaultPhysicsManager::default();
    manager
        .store_settings(PhysicsSettings {
            backend: "builtin".to_string(),
            simulation_mode: PhysicsSimulationMode::QueryOnly,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let world = WorldHandle::new(11);

    poison(manager.synced_worlds.clone());
    poison(manager.contacts.clone());
    poison(manager.trigger_pairs.clone());
    poison(manager.triggers.clone());

    manager.sync_world(PhysicsWorldSyncState {
        world,
        ..PhysicsWorldSyncState::default()
    });

    assert_eq!(manager.synchronized_world(world).unwrap().world, world);
    assert!(manager.drain_contacts(world).is_empty());
    assert!(manager.drain_triggers(world).is_empty());
}

#[test]
fn physics_sync_to_scene_applies_synchronized_body_state() {
    let mut world = World::new();
    let entity = world
        .spawn_node(NodeKind::Cube)
        .expect("spawn physics fixture");
    world
        .set_rigid_body(entity, Some(RigidBodyComponent::default()))
        .unwrap();
    let transform = Transform::from_translation(Vec3::new(3.0, 4.0, 5.0));
    let sync = PhysicsWorldSyncState {
        world: WorldHandle::new(19),
        bodies: vec![PhysicsBodySyncState {
            entity,
            body_type: PhysicsBodyType::Kinematic,
            transform,
            mass: 4.0,
            mass_properties: Default::default(),
            linear_velocity: [1.0, 2.0, 3.0],
            angular_velocity: [0.1, 0.2, 0.3],
            linear_damping: 0.25,
            angular_damping: 0.5,
            gravity_scale: 0.0,
            ccd_mode: Default::default(),
            sleep_policy: PhysicsSleepPolicy::Never,
            lock_translation: [true, false, true],
            lock_rotation: [false, true, false],
        }],
        ..PhysicsWorldSyncState::default()
    };

    apply_synchronized_bodies_to_scene(&mut world, &sync);

    assert_eq!(world.find_node(entity).unwrap().transform, transform);
    let body = world.rigid_body(entity).unwrap();
    assert_eq!(body.body_type, RigidBodyType::Kinematic);
    assert_eq!(body.mass, 4.0);
    assert_eq!(body.linear_velocity, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(body.angular_velocity, Vec3::new(0.1, 0.2, 0.3));
    assert_eq!(body.sleep_policy, PhysicsSleepPolicy::Never);
    assert_eq!(body.lock_translation, [true, false, true]);
    assert_eq!(body.lock_rotation, [false, true, false]);
}

#[test]
fn physics_sync_to_scene_keeps_parented_world_pose_after_a_real_builtin_step() {
    let manager = DefaultPhysicsManager::default();
    manager
        .store_settings(PhysicsSettings {
            backend: "builtin".to_owned(),
            simulation_mode: PhysicsSimulationMode::Simulate,
            fixed_hz: 64,
            ..PhysicsSettings::default()
        })
        .unwrap();
    let handle = WorldHandle::new(145);
    let mut scene = World::new();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    let child = scene.spawn_node(NodeKind::Empty).unwrap();
    let parent_local = Transform::from_translation(Vec3::new(10.0, 0.0, 0.0));
    scene.update_transform(parent, parent_local).unwrap();
    scene
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    assert!(scene.set_parent_checked(child, Some(parent)).unwrap());
    scene
        .set_rigid_body(
            child,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Dynamic,
                linear_velocity: Vec3::new(64.0, 0.0, 0.0),
                linear_damping: 0.0,
                angular_damping: 0.0,
                gravity_scale: 0.0,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    assert_eq!(
        scene
            .world_matrix(child)
            .unwrap()
            .transform_point3(Vec3::ZERO)
            .x,
        12.0
    );

    let result = manager.tick_scene_world(handle, &mut scene, 1.0 / 64.0);
    assert_eq!(result.step_plan.steps, 1);
    assert_eq!(result.step_plan.step_seconds, 1.0 / 64.0);
    let sync = manager
        .synchronized_world_snapshot(handle)
        .expect("snapshot from the actual fixed step");
    assert_eq!(sync.world, handle);
    assert_eq!(sync.bodies.len(), 1);
    assert_eq!(sync.bodies[0].entity, child);
    assert_eq!(sync.bodies[0].transform.translation.x, 13.0);
    assert_eq!(scene.local_transform(child).unwrap().translation.x, 3.0);

    apply_synchronized_bodies_to_scene(&mut scene, &sync);
    let local_x = scene.local_transform(child).unwrap().translation.x;
    let world_x = scene
        .world_matrix(child)
        .unwrap()
        .transform_point3(Vec3::ZERO)
        .x;
    eprintln!("PHYSICS_PARENTED_WRITEBACK before_step=12 snapshot=13 local_after={local_x} world_after={world_x}");
    assert_eq!(scene.parent_of(child), Some(parent));
    assert_eq!(scene.local_transform(parent), Some(parent_local));
    assert_eq!(
        (local_x, world_x),
        (3.0, 13.0),
        "do not apply the parent translation twice"
    );
}

#[test]
fn physics_sync_to_scene_does_not_write_unrepresentable_relative_pose() {
    let mut scene = World::new();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    let child = scene.spawn_node(NodeKind::Empty).unwrap();
    scene
        .update_transform(
            parent,
            Transform {
                scale: Vec3::new(2.0, 1.0, 1.0),
                ..Transform::default()
            },
        )
        .unwrap();
    assert!(scene.set_parent_checked(child, Some(parent)).unwrap());
    scene
        .set_rigid_body(
            child,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Dynamic,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    let before_local = scene.local_transform(child).unwrap();
    let before_world = scene.world_matrix(child).unwrap();
    let mut sync = build_world_sync_state(WorldHandle::new(146), &scene);
    assert_eq!(sync.bodies.len(), 1);
    assert_eq!(sync.bodies[0].entity, child);
    // P^-1 * Rz(pi/4) has nonorthogonal columns (dot=3/8). A local TRS cannot
    // represent this requested world pose exactly; dropping shear is not a fix.
    sync.bodies[0].transform = Transform {
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
        ..Transform::default()
    };
    apply_synchronized_bodies_to_scene(&mut scene, &sync);
    assert_eq!(scene.parent_of(child), Some(parent));
    assert_eq!(scene.local_transform(child), Some(before_local));
    assert_eq!(scene.world_matrix(child), Some(before_world));
}

#[test]
fn physics_sync_to_scene_applies_a_representable_child_target_after_its_parent() {
    let mut scene = World::new();
    // Stable IDs are not parent order. The child is intentionally first in the snapshot.
    let child = scene.spawn_node(NodeKind::Empty).unwrap();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    assert!(child < parent);
    let parent_target = Transform {
        translation: Vec3::new(20.0, 3.0, 0.0),
        rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        scale: Vec3::new(-2.0, 3.0, 1.0),
    };
    scene
        .update_transform(
            parent,
            Transform {
                translation: Vec3::new(10.0, 3.0, 0.0),
                ..parent_target
            },
        )
        .unwrap();
    assert!(scene.set_parent_checked(child, Some(parent)).unwrap());
    for entity in [parent, child] {
        scene
            .set_rigid_body(
                entity,
                Some(RigidBodyComponent {
                    body_type: RigidBodyType::Dynamic,
                    ..RigidBodyComponent::default()
                }),
            )
            .unwrap();
    }
    let mut sync = build_world_sync_state(WorldHandle::new(147), &scene);
    assert_eq!(sync.bodies.len(), 2);
    assert_eq!(sync.bodies[0].entity, child);
    assert_eq!(sync.bodies[1].entity, parent);
    let child_target = Transform {
        // T(20,3,0) * Rz(90deg) * S(-2,3,1) * (3,-1,0,1) = (23,-3,0,1).
        translation: Vec3::new(23.0, -3.0, 0.0),
        ..parent_target
    };
    sync.bodies[0].transform = child_target;
    sync.bodies[1].transform = parent_target;
    assert_eq!(scene.local_transform(child), Some(Transform::default()));

    apply_synchronized_bodies_to_scene(&mut scene, &sync);

    assert_eq!(scene.parent_of(child), Some(parent));
    assert_eq!(scene.local_transform(parent), Some(parent_target));
    let world = scene.world_matrix(child).unwrap();
    let local = scene.local_transform(child).unwrap();
    eprintln!(
        "PHYSICS_PARENTED_TARGET local={:?} world={:?}",
        local.translation,
        world.transform_point3(Vec3::ZERO)
    );
    assert!(
        transform_to_mat4(local).abs_diff_eq(
            transform_to_mat4(Transform::from_translation(Vec3::new(3.0, -1.0, 0.0))),
            1.0e-5,
        ),
        "a changed, representable child pose must be converted against the updated parent; actual {local:?}"
    );
    assert!(world.abs_diff_eq(transform_to_mat4(child_target), 1.0e-5));
}

#[test]
fn physics_sync_to_scene_uses_the_full_sheared_parent_matrix() {
    let (mut scene, parent, child) = sheared_parent_fixture(-std::f32::consts::FRAC_PI_4);
    let parent_world = scene.world_matrix(parent).unwrap();
    assert!(parent_world.x_axis.dot(parent_world.y_axis).abs() > 1.0);
    let mut sync = build_world_sync_state(WorldHandle::new(148), &scene);
    assert_eq!(sync.bodies.len(), 1);
    // P = diag(2,1,1) * Rz(45deg); local Rz(-45deg) cancels the
    // rotation, so the requested world basis is exactly diag(2,1,1).
    let child_local = Transform {
        translation: Vec3::new(1.0, 2.0, 0.0),
        rotation: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_4),
        ..Transform::default()
    };
    let child_world = Transform {
        translation: Vec3::new(
            -std::f32::consts::SQRT_2,
            3.0 * std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        ),
        scale: Vec3::new(2.0, 1.0, 1.0),
        ..Transform::default()
    };
    assert!(
        scene
            .world_matrix(child)
            .unwrap()
            .w_axis
            .truncate()
            .length()
            < 1.0e-5
    );
    sync.bodies[0].transform = child_world;
    apply_synchronized_bodies_to_scene(&mut scene, &sync);

    assert_eq!(scene.parent_of(child), Some(parent));
    assert_eq!(scene.world_matrix(parent), Some(parent_world));
    assert!(
        transform_to_mat4(scene.local_transform(child).unwrap())
            .abs_diff_eq(transform_to_mat4(child_local), 1.0e-5,),
        "a sheared parent must not be approximated by world TRS or rejected wholesale"
    );
    assert!(scene
        .world_matrix(child)
        .unwrap()
        .abs_diff_eq(transform_to_mat4(child_world), 1.0e-5));
}

#[test]
fn physics_sync_to_scene_does_not_flatten_an_unchanged_sheared_pose() {
    let (mut scene, parent, child) = sheared_parent_fixture(0.0);
    let before = scene.world_matrix(child).unwrap();
    assert!(before.x_axis.dot(before.y_axis).abs() > 1.0);
    let local = scene.local_transform(child).unwrap();
    let sync = build_world_sync_state(WorldHandle::new(149), &scene);
    assert_eq!(sync.bodies.len(), 1);
    assert_eq!(Some(sync.bodies[0].transform), scene.world_transform(child));
    apply_synchronized_bodies_to_scene(&mut scene, &sync);
    assert_eq!(scene.parent_of(child), Some(parent));
    assert_eq!(
        scene.local_transform(child),
        Some(local),
        "a snapshot echo is not a pose change"
    );
    assert_eq!(scene.world_matrix(child), Some(before));
}

fn sheared_parent_fixture(child_z: f32) -> (World, EntityId, EntityId) {
    let mut scene = World::new();
    let grandparent = scene.spawn_node(NodeKind::Empty).unwrap();
    let parent = scene.spawn_node(NodeKind::Empty).unwrap();
    let child = scene.spawn_node(NodeKind::Empty).unwrap();
    scene
        .update_transform(
            grandparent,
            Transform {
                scale: Vec3::new(2.0, 1.0, 1.0),
                ..Transform::default()
            },
        )
        .unwrap();
    scene
        .update_transform(
            parent,
            Transform {
                rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                ..Transform::default()
            },
        )
        .unwrap();
    scene
        .update_transform(
            child,
            Transform {
                rotation: Quat::from_rotation_z(child_z),
                ..Transform::default()
            },
        )
        .unwrap();
    scene.set_parent_checked(parent, Some(grandparent)).unwrap();
    scene.set_parent_checked(child, Some(parent)).unwrap();
    scene
        .set_rigid_body(
            child,
            Some(RigidBodyComponent {
                body_type: RigidBodyType::Dynamic,
                ..RigidBodyComponent::default()
            }),
        )
        .unwrap();
    (scene, parent, child)
}

#[test]
fn unchanged_bodies_skip_sync() {
    let body = PhysicsBodySyncState {
        entity: 27,
        body_type: PhysicsBodyType::Dynamic,
        transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        mass: 2.0,
        mass_properties: Default::default(),
        linear_velocity: [4.0, 5.0, 6.0],
        angular_velocity: [0.1, 0.2, 0.3],
        linear_damping: 0.25,
        angular_damping: 0.5,
        gravity_scale: 1.0,
        ccd_mode: Default::default(),
        sleep_policy: Default::default(),
        lock_translation: [false; 3],
        lock_rotation: [false; 3],
    };

    let change = super::change_detection::detect_body_change(&body, &body);

    assert!(!change.requires_commands());
    assert!(!change.requires_recreation());
}

#[test]
fn physics_query_snapshot_clones_the_arc_instead_of_the_world() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(29);
    let snapshot = Arc::new(query_world(world, 32));
    manager
        .synced_worlds
        .lock()
        .unwrap()
        .insert(world, Arc::clone(&snapshot));

    let first = super::query::synchronized_world(&manager, world).unwrap();
    let second = super::query::synchronized_world(&manager, world).unwrap();
    let runtime_snapshot = manager.synchronized_world_snapshot(world).unwrap();
    let service_snapshot = PhysicsManager::synchronized_world(&manager, world).unwrap();

    assert!(Arc::ptr_eq(&snapshot, &first));
    assert!(Arc::ptr_eq(&first, &second));
    assert!(Arc::ptr_eq(&second, &runtime_snapshot));
    assert!(Arc::ptr_eq(&runtime_snapshot, &service_snapshot));
}

#[test]
fn ray_query_modes_and_large_exclusions_preserve_contracts() {
    let manager = DefaultPhysicsManager::default();
    let world = WorldHandle::new(31);
    let sync = PhysicsWorldSyncState {
        world,
        colliders: vec![
            query_collider(30, 5.0),
            query_collider(10, 1.0),
            query_collider(20, 3.0),
        ],
        ..PhysicsWorldSyncState::default()
    };
    manager
        .synced_worlds
        .lock()
        .unwrap()
        .insert(world, Arc::new(sync));

    let mut query = PhysicsRayCastQuery {
        world,
        origin: [0.0, 0.0, 0.0],
        direction: [1.0, 0.0, 0.0],
        max_distance: 10.0,
        mode: PhysicsQueryMode::First,
        filter: PhysicsQueryFilter::default(),
    };
    assert_eq!(super::query::ray_cast(&manager, &query)[0].entity, 30);

    query.mode = PhysicsQueryMode::Closest;
    assert_eq!(super::query::ray_cast(&manager, &query)[0].entity, 10);

    query.mode = PhysicsQueryMode::All;
    assert_eq!(
        super::query::ray_cast(&manager, &query)
            .into_iter()
            .map(|hit| hit.entity)
            .collect::<Vec<_>>(),
        vec![10, 20, 30]
    );

    query.mode = PhysicsQueryMode::Closest;
    query.filter.excluded_entities = (1..=16).collect();
    assert_eq!(super::query::ray_cast(&manager, &query)[0].entity, 20);
}

#[test]
#[ignore = "managed physics query release performance gate"]
fn physics_query_snapshot_filter_and_mode_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const COLLIDER_COUNT: usize = 4_096;
    const EXCLUDED_COUNT: usize = 2_048;
    const MODE_CANDIDATES: usize = 32_768;

    let snapshot = Arc::new(query_world(WorldHandle::new(37), COLLIDER_COUNT));
    let filter = PhysicsQueryFilter {
        excluded_entities: (1..=EXCLUDED_COUNT as u64).collect(),
        ..PhysicsQueryFilter::default()
    };
    let mode_candidates = (0..MODE_CANDIDATES)
        .map(|index| index.wrapping_mul(40_503) % MODE_CANDIDATES)
        .collect::<Vec<_>>();

    let legacy_filter_count = snapshot
        .colliders
        .iter()
        .filter(|collider| legacy_filter_matches(&filter, collider))
        .count();
    let prepared_filter = crate::backend::builtin::PreparedPhysicsQueryFilter::new(&filter);
    let prepared_filter_count = snapshot
        .colliders
        .iter()
        .filter(|collider| prepared_filter.matches(collider))
        .count();
    assert_eq!(legacy_filter_count, COLLIDER_COUNT - EXCLUDED_COUNT);
    assert_eq!(prepared_filter_count, legacy_filter_count);
    assert_eq!(
        crate::backend::builtin::collect_query_mode(
            mode_candidates.iter().copied(),
            PhysicsQueryMode::Closest,
            usize::cmp,
        ),
        vec![0]
    );

    let (snapshot_legacy_ns, snapshot_arc_ns) = measure_alternating_pairs(
        SAMPLE_PAIRS,
        || black_box(snapshot.as_ref().clone()),
        || black_box(Arc::clone(&snapshot)),
    );
    let (filter_legacy_ns, filter_prepared_ns) = measure_alternating_pairs(
        SAMPLE_PAIRS,
        || {
            black_box(
                snapshot
                    .colliders
                    .iter()
                    .filter(|collider| legacy_filter_matches(&filter, collider))
                    .count(),
            )
        },
        || {
            let prepared = crate::backend::builtin::PreparedPhysicsQueryFilter::new(&filter);
            black_box(
                snapshot
                    .colliders
                    .iter()
                    .filter(|collider| prepared.matches(collider))
                    .count(),
            )
        },
    );
    let (mode_sort_ns, mode_linear_ns) = measure_alternating_pairs(
        SAMPLE_PAIRS,
        || {
            let mut hits = mode_candidates.clone();
            hits.sort_unstable();
            hits.truncate(1);
            black_box(hits)
        },
        || {
            black_box(crate::backend::builtin::collect_query_mode(
                mode_candidates.iter().copied(),
                PhysicsQueryMode::Closest,
                usize::cmp,
            ))
        },
    );

    let snapshot_legacy_p95_ns = nearest_rank_percentile(&snapshot_legacy_ns, 95);
    let snapshot_arc_p95_ns = nearest_rank_percentile(&snapshot_arc_ns, 95);
    let filter_legacy_p95_ns = nearest_rank_percentile(&filter_legacy_ns, 95);
    let filter_prepared_p95_ns = nearest_rank_percentile(&filter_prepared_ns, 95);
    let mode_sort_p95_ns = nearest_rank_percentile(&mode_sort_ns, 95);
    let mode_linear_p95_ns = nearest_rank_percentile(&mode_linear_ns, 95);

    println!(
        "PERF_RESULT physics_query_snapshot colliders={COLLIDER_COUNT} sample_pairs={SAMPLE_PAIRS} legacy_p95_ns={snapshot_legacy_p95_ns} arc_p95_ns={snapshot_arc_p95_ns} legacy_ns={} arc_ns={}",
        join_samples(&snapshot_legacy_ns),
        join_samples(&snapshot_arc_ns),
    );
    println!(
        "PERF_RESULT physics_query_filter colliders={COLLIDER_COUNT} excluded={EXCLUDED_COUNT} sample_pairs={SAMPLE_PAIRS} legacy_p95_ns={filter_legacy_p95_ns} prepared_p95_ns={filter_prepared_p95_ns} legacy_ns={} prepared_ns={}",
        join_samples(&filter_legacy_ns),
        join_samples(&filter_prepared_ns),
    );
    println!(
        "PERF_RESULT physics_query_mode candidates={MODE_CANDIDATES} sample_pairs={SAMPLE_PAIRS} sort_p95_ns={mode_sort_p95_ns} linear_p95_ns={mode_linear_p95_ns} sort_ns={} linear_ns={}",
        join_samples(&mode_sort_ns),
        join_samples(&mode_linear_ns),
    );

    assert!(
        snapshot_arc_p95_ns.saturating_mul(4) <= snapshot_legacy_p95_ns,
        "Arc snapshot P95 {snapshot_arc_p95_ns}ns must be at most 25% of deep clone P95 {snapshot_legacy_p95_ns}ns"
    );
    assert!(
        filter_prepared_p95_ns.saturating_mul(2) <= filter_legacy_p95_ns,
        "prepared filter P95 {filter_prepared_p95_ns}ns must be at most 50% of linear exclusion P95 {filter_legacy_p95_ns}ns"
    );
    assert!(
        mode_linear_p95_ns.saturating_mul(4) <= mode_sort_p95_ns.saturating_mul(3),
        "linear closest P95 {mode_linear_p95_ns}ns must be at most 75% of sort P95 {mode_sort_p95_ns}ns"
    );
}

fn query_world(world: WorldHandle, collider_count: usize) -> PhysicsWorldSyncState {
    PhysicsWorldSyncState {
        world,
        colliders: (1..=collider_count)
            .map(|entity| query_collider(entity as u64, entity as f32 + 1.0))
            .collect(),
        ..PhysicsWorldSyncState::default()
    }
}

fn query_collider(entity: u64, x: f32) -> PhysicsColliderSyncState {
    PhysicsColliderSyncState {
        entity,
        shape: PhysicsColliderShape::Sphere { radius: 0.25 },
        sensor: false,
        layer: 0,
        collision_group: 0,
        collision_mask: u32::MAX,
        material: None,
        material_override: None,
        transform: Transform::from_translation(Vec3::new(x, 0.0, 0.0)),
    }
}

fn legacy_filter_matches(filter: &PhysicsQueryFilter, collider: &PhysicsColliderSyncState) -> bool {
    (filter.include_sensors || !collider.sensor)
        && filter.collision_mask.is_none_or(|mask| {
            1_u32
                .checked_shl(collider.layer)
                .is_some_and(|layer_bit| mask & layer_bit != 0)
        })
        && !filter.excluded_entities.contains(&collider.entity)
        && filter
            .required_collision_group
            .is_none_or(|group| collider.collision_group == group)
}

fn measure_alternating_pairs<T, U>(
    sample_pairs: usize,
    mut legacy: impl FnMut() -> T,
    mut optimized: impl FnMut() -> U,
) -> (Vec<u128>, Vec<u128>) {
    let mut legacy_ns = Vec::with_capacity(sample_pairs);
    let mut optimized_ns = Vec::with_capacity(sample_pairs);
    for sample_index in 0..sample_pairs {
        let mut measure_legacy = || {
            let started = Instant::now();
            black_box(legacy());
            legacy_ns.push(started.elapsed().as_nanos());
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            black_box(optimized());
            optimized_ns.push(started.elapsed().as_nanos());
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    (legacy_ns, optimized_ns)
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn poison<T>(state: Arc<Mutex<T>>)
where
    T: Send + 'static,
{
    let result = thread::spawn(move || {
        let _guard = state.lock().unwrap();
        panic!("intentional poison for recovery coverage");
    })
    .join();

    assert!(result.is_err());
}
