use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::scene::EntityId;

use super::{commit_revision_entries, revision_stage_contains, sort_revision_entries};

const ENTITY_COUNT: usize = 8_192;
const SAMPLE_PAIRS: usize = 21;
const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

#[test]
fn sorted_revision_stage_preserves_membership() {
    let mut staged = vec![(17, ()), (3, ()), (9, ()), (5, ())];
    sort_revision_entries(&mut staged);

    assert_eq!(
        staged.iter().map(|(entity, _)| *entity).collect::<Vec<_>>(),
        vec![3, 5, 9, 17]
    );
    assert!(revision_stage_contains(&staged, &3));
    assert!(revision_stage_contains(&staged, &17));
    assert!(!revision_stage_contains(&staged, &4));
}

#[test]
fn sorted_revision_commit_preserves_deferred_values_and_removes_stale_entities() {
    let mut current = [(2, 20), (3, 30), (9, 90)].into_iter().collect();
    let deferred = BTreeSet::from([3]);

    commit_revision_entries(&mut current, vec![(3, 33), (2, 22)], &deferred);

    assert_eq!(
        current.into_iter().collect::<Vec<_>>(),
        vec![(2, 22), (3, 30)]
    );
}

#[test]
#[ignore = "release-only performance gate"]
fn sorted_seen_entity_vector_release_benchmark_evidence() {
    let entities = shuffled_entities();
    let (legacy_samples, optimized_samples) = paired_samples(
        || legacy_seen_entities(&entities),
        || optimized_revision_stage(&entities),
    );
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);

    println!(
        "PERF_RESULT task=runtime170_sorted_projection_membership entities={ENTITY_COUNT} sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_tree_nodes={ENTITY_COUNT} optimized_tree_nodes=0 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT} legacy_raw_ns={} optimized_raw_ns={}",
        samples_csv(&legacy_samples),
        samples_csv(&optimized_samples),
    );
    assert_eq!(legacy_seen_entities(&entities).0.len(), ENTITY_COUNT);
    assert_eq!(optimized_revision_stage(&entities).len(), ENTITY_COUNT);
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "sorted projection membership must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn shuffled_entities() -> Vec<EntityId> {
    let mut state = 0x4d59_5df4_d0f3_3173_u64;
    let mut entities = (0..ENTITY_COUNT as u64).collect::<Vec<_>>();
    for index in (1..entities.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        entities.swap(index, state as usize % (index + 1));
    }
    entities
}

fn legacy_seen_entities(entities: &[EntityId]) -> (Vec<(EntityId, ())>, BTreeSet<EntityId>) {
    (
        entities
            .iter()
            .copied()
            .map(|entity| (entity, ()))
            .collect(),
        entities.iter().copied().collect(),
    )
}

fn optimized_revision_stage(entities: &[EntityId]) -> Vec<(EntityId, ())> {
    let mut staged = entities
        .iter()
        .copied()
        .map(|entity| (entity, ()))
        .collect::<Vec<_>>();
    sort_revision_entries(&mut staged);
    staged
}

fn paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = ordered.len().saturating_mul(percentile).div_ceil(100) - 1;
    ordered[index]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
