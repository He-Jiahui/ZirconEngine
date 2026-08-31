use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use crate::scene::{EntityId, NodeKind, World};

use super::{
    DynamicScene, DynamicSceneError, EntityIdReservationProbe, EntityRemap, build_entity_remap,
};

const SOURCE_ENTITIES: usize = 1_024;
const TARGET_ENTITIES: usize = 2_048;
const BENCHMARK_ITERATIONS: usize = 8;
const SMALL_SOURCE_ENTITIES: usize = 1;
const SMALL_BENCHMARK_ITERATIONS: usize = 256;
const SMALL_MAX_P50_REGRESSION_PERCENT: u128 = 10;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn entity_remap_successor_probe_matches_legacy_dense_and_reordered_collisions() {
    let (mut scene, target) = dense_collision_fixture(257, 513);

    assert_remap_matches_legacy(&scene, &target);

    scene.entities.reverse();
    assert_remap_matches_legacy(&scene, &target);

    scene.entities.rotate_left(83);
    assert_remap_matches_legacy(&scene, &target);

    let (small_scene, small_target) = dense_collision_fixture(8, 513);
    assert_remap_matches_legacy(&small_scene, &small_target);
}

#[test]
fn entity_remap_successor_probe_preserves_terminal_id_exhaustion() {
    let target = World::empty();
    let mut probe = EntityIdReservationProbe::new(&target);

    assert_eq!(probe.reserve(EntityId::MAX).unwrap(), EntityId::MAX);
    assert!(matches!(
        probe.reserve(EntityId::MAX),
        Err(DynamicSceneError::EntityIdSpaceExhausted {
            source_entity: EntityId::MAX
        })
    ));
}

#[test]
#[ignore = "release performance gate; run through the Runtime99u managed validator"]
fn runtime99u_entity_remap_successor_probe_release_benchmark_evidence() {
    let (scene, target) = dense_collision_fixture(SOURCE_ENTITIES, TARGET_ENTITIES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let measure_legacy = || {
            measure_ns(BENCHMARK_ITERATIONS, || {
                legacy_build_entity_remap(&scene, &target)
                    .expect("legacy remap fixture must retain free entity IDs")
            })
        };
        let measure_optimized = || {
            measure_ns(BENCHMARK_ITERATIONS, || {
                build_entity_remap(&scene, &target)
                    .expect("optimized remap fixture must retain free entity IDs")
            })
        };
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);

    let (small_scene, small_target) =
        dense_collision_fixture(SMALL_SOURCE_ENTITIES, TARGET_ENTITIES);
    let mut small_legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut small_optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let measure_legacy = || {
            measure_ns(SMALL_BENCHMARK_ITERATIONS, || {
                legacy_build_entity_remap(&small_scene, &small_target)
                    .expect("small legacy remap fixture must retain free entity IDs")
            })
        };
        let measure_optimized = || {
            measure_ns(SMALL_BENCHMARK_ITERATIONS, || {
                build_entity_remap(&small_scene, &small_target)
                    .expect("small optimized remap fixture must retain free entity IDs")
            })
        };
        if pair % 2 == 0 {
            small_legacy_samples.push(measure_legacy());
            small_optimized_samples.push(measure_optimized());
        } else {
            small_optimized_samples.push(measure_optimized());
            small_legacy_samples.push(measure_legacy());
        }
    }
    let small_legacy_p50 = nearest_rank(&small_legacy_samples, 50);
    let small_legacy_p95 = nearest_rank(&small_legacy_samples, 95);
    let small_optimized_p50 = nearest_rank(&small_optimized_samples, 50);
    let small_optimized_p95 = nearest_rank(&small_optimized_samples, 95);

    assert_remap_matches_legacy(&scene, &target);
    assert_remap_matches_legacy(&small_scene, &small_target);
    println!(
        "RUNTIME99U_ENTITY_REMAP_PERF source_entities={} target_entities={} sample_pairs={} iterations={} sample_order=alternating_legacy_first_even percentile_method=nearest_rank threshold_percent=80 legacy_ns={} optimized_ns={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} small_source_entities={} small_iterations={} small_max_p50_regression_percent={} small_legacy_ns={} small_optimized_ns={} small_legacy_p50_ns={} small_legacy_p95_ns={} small_optimized_p50_ns={} small_optimized_p95_ns={}",
        SOURCE_ENTITIES,
        TARGET_ENTITIES,
        SAMPLE_PAIRS,
        BENCHMARK_ITERATIONS,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95,
        SMALL_SOURCE_ENTITIES,
        SMALL_BENCHMARK_ITERATIONS,
        SMALL_MAX_P50_REGRESSION_PERCENT,
        sample_csv(&small_legacy_samples),
        sample_csv(&small_optimized_samples),
        small_legacy_p50,
        small_legacy_p95,
        small_optimized_p50,
        small_optimized_p95,
    );
    assert!(
        optimized_p50.saturating_mul(100) <= legacy_p50.saturating_mul(20),
        "optimized P50 {optimized_p50}ns must be at most 20% of legacy P50 {legacy_p50}ns"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(20),
        "optimized P95 {optimized_p95}ns must be at most 20% of legacy P95 {legacy_p95}ns"
    );
    assert!(
        small_optimized_p50.saturating_mul(100)
            <= small_legacy_p50.saturating_mul(100 + SMALL_MAX_P50_REGRESSION_PERCENT),
        "small optimized P50 {small_optimized_p50}ns must remain within {SMALL_MAX_P50_REGRESSION_PERCENT}% of legacy P50 {small_legacy_p50}ns"
    );
}

fn dense_collision_fixture(source_count: usize, target_count: usize) -> (DynamicScene, World) {
    let mut source = World::empty();
    for _ in 0..source_count {
        source
            .spawn_node(NodeKind::Empty)
            .expect("source fixture must allocate dense entity IDs");
    }
    let scene = DynamicScene::from_world(&source).expect("source fixture must capture");

    let mut target = World::empty();
    for _ in 0..target_count {
        target
            .spawn_node(NodeKind::Empty)
            .expect("target fixture must allocate dense entity IDs");
    }
    (scene, target)
}

fn assert_remap_matches_legacy(scene: &DynamicScene, target: &World) {
    let optimized = build_entity_remap(scene, target).expect("optimized remap must succeed");
    let legacy = legacy_build_entity_remap(scene, target).expect("legacy remap must succeed");
    assert_eq!(optimized, legacy);
}

fn legacy_build_entity_remap(
    scene: &DynamicScene,
    world: &World,
) -> Result<EntityRemap, DynamicSceneError> {
    let mut remap = EntityRemap::new();
    let mut reserved = BTreeSet::new();
    for entity in &scene.entities {
        let mut candidate = entity.source_entity;
        loop {
            if !world.contains_entity(candidate) && !reserved.contains(&candidate) {
                reserved.insert(candidate);
                remap.insert(entity.source_entity, candidate);
                break;
            }
            candidate =
                candidate
                    .checked_add(1)
                    .ok_or(DynamicSceneError::EntityIdSpaceExhausted {
                        source_entity: entity.source_entity,
                    })?;
        }
    }
    Ok(remap)
}

fn measure_ns(iterations: usize, mut remap: impl FnMut() -> EntityRemap) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum ^= black_box(remap()).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
