use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::scene::NodeKind;

const MOBILITY_BENCH_ENTITY_COUNT: usize = 8_192;
const MOBILITY_BENCH_DIRECT_CHILD_COUNT: usize = 8;
const MOBILITY_BENCH_ITERATIONS: usize = 64;
const MOBILITY_BENCH_SAMPLE_PAIRS: usize = 21;

fn world_with_direct_children(
    entity_count: usize,
    direct_child_count: usize,
) -> (World, EntityId, Vec<EntityId>) {
    assert!(entity_count > direct_child_count);
    let mut world = World::empty();
    let parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let mut direct_children = Vec::with_capacity(direct_child_count);
    for index in 1..entity_count {
        let entity = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        if index <= direct_child_count {
            world.set_parent_checked(entity, Some(parent)).unwrap();
            direct_children.push(entity);
        }
    }
    (world, parent, direct_children)
}

fn legacy_has_static_direct_child(world: &World, parent: EntityId) -> bool {
    world.stable_entity_ids().any(|child| {
        world.parent_of(child) == Some(parent) && world.mobility(child) == Some(Mobility::Static)
    })
}

fn indexed_has_static_direct_child(world: &World, parent: EntityId) -> bool {
    world.has_direct_child_matching(parent, |child| {
        world.mobility(child) == Some(Mobility::Static)
    })
}

fn measure_ns(mut workload: impl FnMut()) -> u128 {
    let started = Instant::now();
    for _ in 0..MOBILITY_BENCH_ITERATIONS {
        workload();
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn dynamic_mobility_rejects_static_direct_child_among_unrelated_entities() {
    let (mut world, parent, direct_children) = world_with_direct_children(4_096, 4);
    world.set_mobility(parent, Mobility::Static).unwrap();
    world
        .set_mobility(direct_children[2], Mobility::Static)
        .unwrap();

    assert_eq!(
        world.set_mobility(parent, Mobility::Dynamic),
        Err(SceneError::DynamicMobilityWithStaticChildren { entity: parent })
    );
}

#[test]
#[ignore = "release performance gate; run through the managed Runtime62 batch"]
fn indexed_mobility_child_validation_release_gate() {
    let (mut world, parent, direct_children) = world_with_direct_children(
        MOBILITY_BENCH_ENTITY_COUNT,
        MOBILITY_BENCH_DIRECT_CHILD_COUNT,
    );
    world.set_mobility(parent, Mobility::Static).unwrap();
    assert_eq!(direct_children.len(), MOBILITY_BENCH_DIRECT_CHILD_COUNT);
    assert!(!legacy_has_static_direct_child(&world, parent));
    assert!(!indexed_has_static_direct_child(&world, parent));

    let mut legacy_samples = Vec::with_capacity(MOBILITY_BENCH_SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(MOBILITY_BENCH_SAMPLE_PAIRS);
    for pair in 0..MOBILITY_BENCH_SAMPLE_PAIRS {
        let measure_legacy = || {
            measure_ns(|| {
                black_box(legacy_has_static_direct_child(
                    black_box(&world),
                    black_box(parent),
                ));
            })
        };
        let measure_indexed = || {
            measure_ns(|| {
                black_box(indexed_has_static_direct_child(
                    black_box(&world),
                    black_box(parent),
                ));
            })
        };
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50_ns = nearest_rank_percentile(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let indexed_p50_ns = nearest_rank_percentile(&indexed_samples, 50);
    let indexed_p95_ns = nearest_rank_percentile(&indexed_samples, 95);
    let legacy_entity_visits = MOBILITY_BENCH_ENTITY_COUNT * MOBILITY_BENCH_ITERATIONS;
    let indexed_entity_visits = MOBILITY_BENCH_DIRECT_CHILD_COUNT * MOBILITY_BENCH_ITERATIONS;
    let legacy_samples_ns = sample_csv(&legacy_samples);
    let indexed_samples_ns = sample_csv(&indexed_samples);

    println!(
        "PERF-MVP-558 task=runtime62_mobility_child_index sample_pairs={} entity_count={} direct_children={} iterations={} legacy_entity_visits={} indexed_entity_visits={} legacy_p50_ns={} legacy_p95_ns={} indexed_p50_ns={} indexed_p95_ns={} legacy_samples_ns={} indexed_samples_ns={}",
        MOBILITY_BENCH_SAMPLE_PAIRS,
        MOBILITY_BENCH_ENTITY_COUNT,
        MOBILITY_BENCH_DIRECT_CHILD_COUNT,
        MOBILITY_BENCH_ITERATIONS,
        legacy_entity_visits,
        indexed_entity_visits,
        legacy_p50_ns,
        legacy_p95_ns,
        indexed_p50_ns,
        indexed_p95_ns,
        legacy_samples_ns,
        indexed_samples_ns,
    );

    assert_eq!(legacy_entity_visits, 524_288);
    assert_eq!(indexed_entity_visits, 512);
    assert!(
        indexed_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "indexed P95 {indexed_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
    );
}
