use std::hint::black_box;
use std::time::Instant;

use crate::scene::{EntityId, NodeKind, World};

use super::super::SceneBindingGenerations;
use super::scene_binding_removal_roots;

const BENCHMARK_ROOT_COUNT: usize = 65_536;
const BENCHMARK_WARMUP_PAIRS: usize = 4;
const BENCHMARK_SAMPLE_PAIRS: usize = 21;

#[test]
fn streamed_scene_binding_root_invalidation_preserves_one_generation() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let parent = world.spawn_node(NodeKind::Empty).unwrap();
    let removed = world.spawn_node(NodeKind::Empty).unwrap();
    world.set_parent_checked(parent, Some(root)).unwrap();
    world.set_parent_checked(removed, Some(parent)).unwrap();
    let previous_generation = world.scene_binding_generation(root);

    world.remove_entity(removed).unwrap();

    let generation = world.scene_binding_generation(root);
    assert!(generation > previous_generation);
    assert_eq!(world.scene_binding_generation(parent), generation);
    assert_eq!(world.scene_binding_generation(removed), generation);
}

#[test]
#[ignore = "performance acceptance benchmark"]
fn streamed_scene_binding_root_invalidation_performance_acceptance() {
    let roots = benchmark_roots(BENCHMARK_ROOT_COUNT);
    let entity = roots[0];
    let ancestors = &roots[1..];
    let mut legacy = SceneBindingGenerations::default();
    let mut optimized = SceneBindingGenerations::default();
    legacy.advance_roots(roots.iter().copied());
    optimized.advance_roots(roots.iter().copied());

    for _ in 0..BENCHMARK_WARMUP_PAIRS {
        black_box(time_legacy(&mut legacy, entity, ancestors));
        black_box(time_optimized(&mut optimized, entity, ancestors));
    }

    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut legacy_checksum = 0_u64;
    let mut optimized_checksum = 0_u64;
    for pair in 0..BENCHMARK_SAMPLE_PAIRS {
        let ((legacy_ns, legacy_result), (optimized_ns, optimized_result)) = if pair % 2 == 0 {
            (
                time_legacy(&mut legacy, entity, ancestors),
                time_optimized(&mut optimized, entity, ancestors),
            )
        } else {
            let optimized_result = time_optimized(&mut optimized, entity, ancestors);
            let legacy_result = time_legacy(&mut legacy, entity, ancestors);
            (legacy_result, optimized_result)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
        legacy_checksum = legacy_checksum.wrapping_add(legacy_result);
        optimized_checksum = optimized_checksum.wrapping_add(optimized_result);
    }

    let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&optimized_samples, 50);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);

    println!(
        "RUNTIME05_STREAMED_SCENE_BINDING_ROOT_INVALIDATION_PERF roots={} warmup_pairs={} sample_pairs={} order=alternating percentile=nearest-rank legacy_sort_calls=1 legacy_dedup_calls=1 optimized_sort_calls=0 optimized_dedup_calls=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_checksum={} optimized_checksum={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        BENCHMARK_ROOT_COUNT,
        BENCHMARK_WARMUP_PAIRS,
        BENCHMARK_SAMPLE_PAIRS,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_checksum,
        optimized_checksum,
        legacy_samples,
        optimized_samples,
    );

    assert_eq!(legacy_checksum, optimized_checksum);
    assert_ne!(optimized_checksum, 0);
    assert!(
        optimized_p50_ns.saturating_mul(100) <= legacy_p50_ns.saturating_mul(90),
        "streamed roots must reduce P50 by at least 10%: legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns",
    );
    assert!(
        optimized_p95_ns <= legacy_p95_ns,
        "streamed roots must not regress P95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns",
    );
}

fn benchmark_roots(count: usize) -> Vec<EntityId> {
    (0..count)
        .map(|index| ((index * 32_771) % count) as EntityId + 1)
        .collect()
}

fn time_legacy(
    generations: &mut SceneBindingGenerations,
    entity: EntityId,
    ancestors: &[EntityId],
) -> (u128, u64) {
    let started = Instant::now();
    let mut roots = Vec::with_capacity(ancestors.len() + 1);
    roots.push(entity);
    roots.extend_from_slice(black_box(ancestors));
    roots.sort_unstable();
    roots.dedup();
    generations.advance_roots(roots);
    let elapsed = started.elapsed().as_nanos();
    (
        elapsed,
        generations
            .for_root(entity)
            .wrapping_add(generations.catalog_generation()),
    )
}

fn time_optimized(
    generations: &mut SceneBindingGenerations,
    entity: EntityId,
    ancestors: &[EntityId],
) -> (u128, u64) {
    let started = Instant::now();
    generations.advance_roots(scene_binding_removal_roots(
        entity,
        black_box(ancestors).iter().copied(),
    ));
    let elapsed = started.elapsed().as_nanos();
    (
        elapsed,
        generations
            .for_root(entity)
            .wrapping_add(generations.catalog_generation()),
    )
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}
