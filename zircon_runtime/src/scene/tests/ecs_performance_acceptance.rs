use std::time::Instant;

use crate::core::diagnostics::{DiagnosticStore, DiagnosticStoreSnapshot};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::Name;
use crate::scene::ecs::{
    ChangeDetectionScanStats, Changed, Component, ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC,
    ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC, ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC, ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC,
    ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC, EcsFramePerformanceDiagnostics, InternalSceneSystem,
    QueryState, QueryStateCacheStats, SystemState,
};
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

fn diagnostic_current(snapshot: &DiagnosticStoreSnapshot, path: &str) -> Option<f64> {
    snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
}

#[test]
fn ecs_frame_performance_diagnostics_record_query_and_change_counts() {
    let mut frame = EcsFramePerformanceDiagnostics::default();
    frame.add_query_stats(QueryStateCacheStats {
        cache_hits: 3,
        cache_misses: 1,
        cache_rebuilds: 1,
        cached_revision: 7,
        cached_archetype_count: 2,
        cached_entity_count: 64,
        candidate_entity_count: 96,
        matched_entity_count: 48,
    });
    frame.add_query_stats(QueryStateCacheStats {
        cache_hits: 5,
        cache_misses: 2,
        cache_rebuilds: 2,
        cached_revision: 11,
        cached_archetype_count: 3,
        cached_entity_count: 32,
        candidate_entity_count: 48,
        matched_entity_count: 24,
    });
    frame.add_change_detection_stats(ChangeDetectionScanStats {
        scanned_marks: 6,
        added_matches: 2,
        changed_matches: 1,
    });
    frame.add_change_detection_stats(ChangeDetectionScanStats {
        scanned_marks: 4,
        added_matches: 1,
        changed_matches: 3,
    });

    assert_eq!(frame.query.cache_hits, 8);
    assert_eq!(frame.query.cache_misses, 3);
    assert_eq!(frame.query.cache_rebuilds, 3);
    assert_eq!(frame.query.cached_revision, 11);
    assert_eq!(frame.query.cached_archetype_count, 5);
    assert_eq!(frame.query.cached_entity_count, 96);
    assert_eq!(frame.query.candidate_entity_count, 144);
    assert_eq!(frame.query.matched_entity_count, 72);
    assert_eq!(frame.change_detection.scanned_marks, 10);
    assert_eq!(frame.change_detection.added_matches, 3);
    assert_eq!(frame.change_detection.changed_matches, 4);

    let mut diagnostics = DiagnosticStore::default();
    frame.record_diagnostics(&mut diagnostics, 7);
    let snapshot = diagnostics.snapshot();
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC),
        Some(8.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC),
        Some(3.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC),
        Some(3.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC),
        Some(144.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC),
        Some(72.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC),
        Some(10.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC),
        Some(3.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC),
        Some(4.0)
    );
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
fn query_state_cache_stats_record_reuse_and_rebuild_counts() {
    let mut world = World::empty();
    spawn_health_entities(&mut world, ENTITY_COUNT);

    type HealthQuery = QueryState<(EntityId, &'static Health)>;
    let mut system = SystemState::<HealthQuery>::new(&mut world).unwrap();
    let initial = system.state().cache_stats();
    assert_eq!(initial.cache_hits, 0);
    assert_eq!(initial.cache_misses, 1);
    assert_eq!(initial.cache_rebuilds, 1);
    assert_eq!(initial.candidate_entity_count, ENTITY_COUNT);
    assert_eq!(initial.matched_entity_count, ENTITY_COUNT);
    assert_eq!(initial.cached_entity_count, ENTITY_COUNT);

    for _ in 0..REPEATED_QUERY_RUNS {
        let count = system.run(&mut world, |query| query.count());
        assert_eq!(count, ENTITY_COUNT);
    }
    let reused = system.state().cache_stats();
    assert_eq!(reused.cache_hits, REPEATED_QUERY_RUNS as u64);
    assert_eq!(reused.cache_misses, 1);
    assert_eq!(reused.cache_rebuilds, 1);
    assert_eq!(reused.cached_revision, initial.cached_revision);

    world
        .spawn((Name("Perf stats structural insert".to_string()), Health(7)))
        .unwrap();
    let after_spawn_count = system.run(&mut world, |query| query.count());
    assert_eq!(after_spawn_count, ENTITY_COUNT + 1);
    let rebuilt = system.state().cache_stats();
    assert_eq!(rebuilt.cache_hits, REPEATED_QUERY_RUNS as u64);
    assert_eq!(rebuilt.cache_misses, 2);
    assert_eq!(rebuilt.cache_rebuilds, 2);
    assert_eq!(rebuilt.candidate_entity_count, ENTITY_COUNT + 1);
    assert_eq!(rebuilt.matched_entity_count, ENTITY_COUNT + 1);
    assert!(rebuilt.cached_revision > reused.cached_revision);

    let mut diagnostics = DiagnosticStore::default();
    rebuilt.record_diagnostics(&mut diagnostics, 42);
    let snapshot = diagnostics.snapshot();
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC),
        Some(REPEATED_QUERY_RUNS as f64)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC),
        Some(2.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC),
        Some(2.0)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC),
        Some((ENTITY_COUNT + 1) as f64)
    );
    assert_eq!(
        diagnostic_current(&snapshot, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC),
        Some((ENTITY_COUNT + 1) as f64)
    );
}

#[test]
fn query_state_reuses_archetype_matches_across_unchanged_frames() {
    let mut world = World::empty();
    spawn_health_entities(&mut world, ENTITY_COUNT);

    type HealthQuery = QueryState<(EntityId, &'static Health)>;
    let mut system = SystemState::<HealthQuery>::new(&mut world).unwrap();
    let initial = system.state().cache_stats();
    assert_eq!(initial.cache_rebuilds, 1);

    for _ in 0..REPEATED_QUERY_RUNS {
        let count = system.run(&mut world, |query| query.count());
        assert_eq!(count, ENTITY_COUNT);
    }

    let reused = system.state().cache_stats();
    assert_eq!(reused.cache_hits, REPEATED_QUERY_RUNS as u64);
    assert_eq!(reused.cache_misses, 1);
    assert_eq!(reused.cache_rebuilds, initial.cache_rebuilds);
    assert_eq!(reused.cached_revision, initial.cached_revision);
    assert_eq!(reused.candidate_entity_count, ENTITY_COUNT);
    assert_eq!(reused.matched_entity_count, ENTITY_COUNT);
}

#[test]
fn system_state_records_query_cache_stats_into_world_frame_diagnostics() {
    let mut world = World::empty();
    spawn_health_entities(&mut world, ENTITY_COUNT);

    type HealthQuery = QueryState<(EntityId, &'static Health)>;
    let mut first_system = SystemState::<HealthQuery>::new(&mut world).unwrap();
    let mut second_system = SystemState::<HealthQuery>::new(&mut world).unwrap();

    world.reset_ecs_frame_performance_diagnostics();
    assert_eq!(
        world.ecs_frame_performance_diagnostics(),
        EcsFramePerformanceDiagnostics::default()
    );

    assert_eq!(
        first_system.run(&mut world, |query| query.count()),
        ENTITY_COUNT
    );
    assert_eq!(
        second_system.run(&mut world, |query| query.count()),
        ENTITY_COUNT
    );
    let first_frame = world.ecs_frame_performance_diagnostics();
    assert_eq!(first_frame.query.cache_hits, 2);
    assert_eq!(first_frame.query.cache_misses, 0);
    assert_eq!(first_frame.query.cache_rebuilds, 0);
    assert_eq!(first_frame.query.candidate_entity_count, ENTITY_COUNT * 2);
    assert_eq!(first_frame.query.matched_entity_count, ENTITY_COUNT * 2);

    world.reset_ecs_frame_performance_diagnostics();
    assert_eq!(
        first_system.run(&mut world, |query| query.count()),
        ENTITY_COUNT
    );
    let second_frame = world.ecs_frame_performance_diagnostics();
    assert_eq!(second_frame.query.cache_hits, 1);
    assert_eq!(second_frame.query.cache_misses, 0);
    assert_eq!(second_frame.query.cache_rebuilds, 0);
    assert_eq!(second_frame.query.candidate_entity_count, ENTITY_COUNT);
    assert_eq!(second_frame.query.matched_entity_count, ENTITY_COUNT);
}

#[test]
fn system_state_records_change_detection_stats_into_world_frame_diagnostics() {
    let mut world = World::empty();
    let entities = spawn_health_entities(&mut world, ENTITY_COUNT);

    type ChangedHealth = QueryState<(EntityId, &'static Health), Changed<Health>>;
    let mut system = SystemState::<ChangedHealth>::new(&mut world).unwrap();

    world.reset_ecs_frame_performance_diagnostics();
    let initial = system.run(&mut world, |query| query.iter().count());
    assert_eq!(initial, ENTITY_COUNT);
    let first_frame = world.ecs_frame_performance_diagnostics();
    assert_eq!(
        first_frame.change_detection.scanned_marks,
        ENTITY_COUNT as u64
    );
    assert_eq!(
        first_frame.change_detection.changed_matches,
        ENTITY_COUNT as u64
    );
    assert_eq!(first_frame.change_detection.added_matches, 0);

    world.reset_ecs_frame_performance_diagnostics();
    let unchanged = system.run(&mut world, |query| query.count());
    assert_eq!(unchanged, 0);
    let second_frame = world.ecs_frame_performance_diagnostics();
    assert_eq!(
        second_frame.change_detection.scanned_marks,
        ENTITY_COUNT as u64
    );
    assert_eq!(second_frame.change_detection.changed_matches, 0);
    assert_eq!(second_frame.change_detection.added_matches, 0);

    for entity in entities.iter().copied().take(CHANGED_ENTITY_COUNT) {
        world.get_mut::<Health>(entity).unwrap().0 += 1;
    }
    world.reset_ecs_frame_performance_diagnostics();
    let changed = system.run(&mut world, |query| query.iter().count());
    assert_eq!(changed, CHANGED_ENTITY_COUNT);
    let third_frame = world.ecs_frame_performance_diagnostics();
    assert_eq!(
        third_frame.change_detection.scanned_marks,
        ENTITY_COUNT as u64
    );
    assert_eq!(
        third_frame.change_detection.changed_matches,
        CHANGED_ENTITY_COUNT as u64
    );
    assert_eq!(third_frame.change_detection.added_matches, 0);
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
