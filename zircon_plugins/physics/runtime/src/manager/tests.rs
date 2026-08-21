use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use zircon_runtime::core::framework::physics::{
    PhysicsBodySyncState, PhysicsBodyType, PhysicsColliderShape, PhysicsColliderSyncState,
    PhysicsManager, PhysicsQueryFilter, PhysicsQueryMode, PhysicsRayCastQuery, PhysicsSettings,
    PhysicsSimulationMode, PhysicsWorldSyncState,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::scene::physics::PhysicsSleepPolicy;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{NodeKind, RigidBodyComponent, RigidBodyType};
use zircon_runtime::scene::world::World;

use super::{DefaultPhysicsManager, PhysicsBodyCommand, apply_synchronized_bodies_to_scene};

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
    let entity = world.spawn_node(NodeKind::Cube);
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

    assert!(Arc::ptr_eq(&snapshot, &first));
    assert!(Arc::ptr_eq(&first, &second));
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
