use std::time::Instant;

use crate::core::math::{Transform, Vec3};
use crate::scene::components::Name;
use crate::scene::ecs::{Changed, Component, InternalSceneSystem, QueryState, SystemState};
use crate::scene::{EntityId, NodeKind, World};

const ENTITY_COUNT: usize = 128;
const REPEATED_QUERY_RUNS: usize = 8;
const CHANGED_ENTITY_COUNT: usize = 16;
const TRANSFORM_READS: usize = 128;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

fn spawn_health_entities(world: &mut World, count: usize) -> Vec<EntityId> {
    (0..count)
        .map(|index| {
            world
                .spawn((Name(format!("Perf {index}")), Health(index as u32)))
                .unwrap()
        })
        .collect()
}

fn expected_health_sum(count: usize, offset: u32) -> u64 {
    (0..count)
        .map(|index| u64::from(offset + index as u32))
        .sum()
}

#[test]
fn spawn_query_hot_path_reuses_cache_until_structural_change() {
    let mut world = World::empty();
    let entities = spawn_health_entities(&mut world, ENTITY_COUNT);

    type HealthQuery = QueryState<(EntityId, &'static Health)>;
    let mut system = SystemState::<HealthQuery>::new(&mut world).unwrap();
    assert_eq!(system.state().cache_rebuilds(), 1);
    assert_eq!(system.state().cached_entity_count(), ENTITY_COUNT);

    let start = Instant::now();
    let baseline_sum = system.run(&mut world, |query| {
        let iter = query.iter();
        assert!(iter.uses_cached_component_locations());
        iter.map(|(_, health)| u64::from(health.0)).sum::<u64>()
    });
    assert_eq!(baseline_sum, expected_health_sum(ENTITY_COUNT, 0));

    for _ in 0..REPEATED_QUERY_RUNS {
        let (count, sum) = system.run(&mut world, |query| {
            query.iter().fold((0, 0_u64), |(count, sum), (_, health)| {
                (count + 1, sum + u64::from(health.0))
            })
        });
        assert_eq!(count, ENTITY_COUNT);
        assert_eq!(sum, expected_health_sum(ENTITY_COUNT, 0));
        assert_eq!(system.state().cache_rebuilds(), 1);
    }

    for (index, entity) in entities.iter().copied().enumerate() {
        world.insert(entity, Health(1_000 + index as u32)).unwrap();
    }
    let replaced_sum = system.run(&mut world, |query| {
        query
            .iter()
            .map(|(_, health)| u64::from(health.0))
            .sum::<u64>()
    });
    assert_eq!(replaced_sum, expected_health_sum(ENTITY_COUNT, 1_000));
    assert_eq!(system.state().cache_rebuilds(), 1);

    let extra = world
        .spawn((Name("Perf structural insert".to_string()), Health(9_999)))
        .unwrap();
    let after_spawn = system.run(&mut world, |query| {
        query
            .iter()
            .map(|(entity, health)| (entity, health.0))
            .collect::<Vec<_>>()
    });
    assert_eq!(after_spawn.len(), ENTITY_COUNT + 1);
    assert_eq!(after_spawn.last(), Some(&(extra, 9_999)));
    assert_eq!(system.state().cache_rebuilds(), 2);
    assert_eq!(system.state().cached_entity_count(), ENTITY_COUNT + 1);

    eprintln!(
        "ecs perf acceptance spawn/query: entities={} repeated_runs={} cache_rebuilds={} elapsed_us={}",
        ENTITY_COUNT,
        REPEATED_QUERY_RUNS,
        system.state().cache_rebuilds(),
        start.elapsed().as_micros()
    );
}

#[test]
fn changed_filter_hot_path_matches_only_mutated_entities_without_cache_rebuild() {
    let mut world = World::empty();
    let entities = spawn_health_entities(&mut world, ENTITY_COUNT);

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();
    assert_eq!(system.state().cache_rebuilds(), 1);

    let start = Instant::now();
    let initial = system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(initial, entities);
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged = system.run(&mut world, |query| query.iter().count());
    assert_eq!(unchanged, 0);
    assert_eq!(system.state().cache_rebuilds(), 1);

    for entity in entities.iter().copied().take(CHANGED_ENTITY_COUNT) {
        world.get_mut::<Health>(entity).unwrap().0 += 10_000;
    }

    let changed = system.run(&mut world, |query| {
        query.iter().map(|(entity, _)| entity).collect::<Vec<_>>()
    });
    assert_eq!(changed, entities[..CHANGED_ENTITY_COUNT]);
    assert_eq!(system.state().cache_rebuilds(), 1);

    let unchanged_after_read = system.run(&mut world, |query| query.iter().count());
    assert_eq!(unchanged_after_read, 0);
    assert_eq!(system.state().cache_rebuilds(), 1);

    eprintln!(
        "ecs perf acceptance changed: entities={} changed={} cache_rebuilds={} elapsed_us={}",
        ENTITY_COUNT,
        CHANGED_ENTITY_COUNT,
        system.state().cache_rebuilds(),
        start.elapsed().as_micros()
    );
}

#[test]
fn transform_hot_path_projects_stable_world_transform_and_flushes_once() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();

    assert!(world.has_pending_scene_systems());

    let start = Instant::now();
    let mut checksum = 0.0_f32;
    for _ in 0..TRANSFORM_READS {
        let transform = world.world_transform(child).unwrap();
        assert_eq!(transform.translation, Vec3::new(7.0, 0.0, 0.0));
        checksum += transform.translation.x;
    }
    assert_eq!(checksum, 7.0 * TRANSFORM_READS as f32);
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_system(InternalSceneSystem::WorldTransform);
    assert!(world.has_pending_scene_systems());
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );

    eprintln!(
        "ecs perf acceptance transform: reads={} pending_after_projection=true elapsed_us={}",
        TRANSFORM_READS,
        start.elapsed().as_micros()
    );
}
