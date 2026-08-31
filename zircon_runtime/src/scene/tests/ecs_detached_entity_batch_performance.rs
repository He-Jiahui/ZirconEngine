use std::time::{Duration, Instant};

use crate::scene::ecs::{Component, DetachedEntityBatchDiagnostics, StorageType};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct DetachedHealth(u32);

impl Component for DetachedHealth {}

#[derive(Debug, PartialEq, Eq)]
struct DetachedSparseMarker;

impl Component for DetachedSparseMarker {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[test]
fn subtree_component_count_is_scoped_to_descendants() {
    let mut world = World::empty();
    let root = world.spawn((DetachedHealth(0),)).unwrap();
    let descendant = world.spawn((DetachedHealth(1),)).unwrap();
    let unrelated = world.spawn((DetachedHealth(2),)).unwrap();
    world.set_parent_checked(descendant, Some(root)).unwrap();

    assert_eq!(world.subtree_component_count::<DetachedHealth>(root), 2);
    assert_eq!(
        world.subtree_component_count::<DetachedHealth>(unrelated),
        1
    );
    assert_eq!(world.subtree_component_count::<DetachedHealth>(999_999), 0);
}

fn detached_batch_scale_sample(
    affected_entities: usize,
    unrelated_entities: usize,
) -> (
    DetachedEntityBatchDiagnostics,
    DetachedEntityBatchDiagnostics,
    Duration,
    Duration,
) {
    assert!(affected_entities > 0);
    let mut world = World::empty();
    let root = world
        .spawn((DetachedHealth(0), DetachedSparseMarker))
        .expect("spawn detached batch root");
    for index in 1..affected_entities {
        let entity = world
            .spawn((DetachedHealth(index as u32), DetachedSparseMarker))
            .expect("spawn detached batch descendant");
        world
            .set_parent_checked(entity, Some(root))
            .expect("attach detached batch descendant");
    }
    let first_unrelated = affected_entities as EntityId + 1;
    for offset in 0..unrelated_entities {
        world
            .spawn_empty_at(first_unrelated + offset as EntityId)
            .expect("spawn unrelated entity");
    }

    world.reset_ecs_frame_performance_diagnostics();
    let detach_started = Instant::now();
    let batch = world
        .remove_entity_recursive(root)
        .expect("detach profiled subtree");
    let detach_elapsed = detach_started.elapsed();
    let detach = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;

    world.reset_ecs_frame_performance_diagnostics();
    let restore_started = Instant::now();
    world
        .restore_detached_entity_batch(batch)
        .expect("restore profiled subtree");
    let restore_elapsed = restore_started.elapsed();
    let restore = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;

    let entity_query = world.query::<crate::scene::EntityId>();
    assert_eq!(
        entity_query.iter(&world).count(),
        affected_entities + unrelated_entities
    );
    (detach, restore, detach_elapsed, restore_elapsed)
}

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    let index = samples.len().saturating_mul(95).div_ceil(100) - 1;
    samples[index]
}

#[test]
fn detached_batch_cost_tracks_affected_rows_instead_of_world_cardinality() {
    let (small_world, _, _, _) = detached_batch_scale_sample(1, 0);
    let (large_world, _, _, _) = detached_batch_scale_sample(1, 10_000);

    assert_eq!(large_world, small_world);
    assert_eq!(large_world.moved_rows, 1);
    assert_eq!(large_world.ordered_removals, 1);
    assert_eq!(large_world.hierarchy_index_lookups, 1);
    assert_eq!(large_world.full_world_clone_bytes, 0);
    assert_eq!(large_world.node_record_clone_bytes, 0);
    assert_eq!(large_world.rollback_bytes, 0);
}

#[test]
#[ignore = "managed Runtime08 1/1k/100k detached-batch profiling gate"]
fn detached_entity_batch_managed_scale_fixture() {
    const WORLD_ENTITIES: usize = 100_000;
    const SAMPLE_COUNT: usize = 20;

    for affected_entities in [1_usize, 1_000, 100_000] {
        let mut detach_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut restore_samples = Vec::with_capacity(SAMPLE_COUNT);
        let unrelated_entities = WORLD_ENTITIES - affected_entities;
        let mut last_detach = DetachedEntityBatchDiagnostics::default();
        for _ in 0..SAMPLE_COUNT {
            let (detach, restore, detach_elapsed, restore_elapsed) =
                detached_batch_scale_sample(affected_entities, unrelated_entities);
            for diagnostics in [detach, restore] {
                assert_eq!(diagnostics.commit_count, 1);
                assert_eq!(diagnostics.moved_rows, affected_entities as u64);
                assert_eq!(diagnostics.archetype_publications, affected_entities as u64);
                assert_eq!(diagnostics.generation_advances, 1);
                assert_eq!(diagnostics.full_world_clone_bytes, 0);
                assert_eq!(diagnostics.node_record_clone_bytes, 0);
                assert_eq!(diagnostics.rollback_bytes, 0);
            }
            assert_eq!(detach.ordered_removals, affected_entities as u64);
            assert_eq!(detach.hierarchy_index_lookups, affected_entities as u64);
            assert_eq!(restore.ordered_removals, 0);
            last_detach = detach;
            detach_samples.push(detach_elapsed);
            restore_samples.push(restore_elapsed);
        }
        eprintln!(
            "detached_batch_scale world={WORLD_ENTITIES} affected={affected_entities} samples={SAMPLE_COUNT} detach_p95_ms={} restore_p95_ms={} moved_table={} moved_sparse={} swap_repairs={}",
            p95(detach_samples).as_secs_f64() * 1_000.0,
            p95(restore_samples).as_secs_f64() * 1_000.0,
            last_detach.moved_table_components,
            last_detach.moved_sparse_components,
            last_detach.swap_repairs,
        );
    }
}
