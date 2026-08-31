use std::hint::black_box;
use std::time::Instant;

use zr_contracts::random::{
    RandomAlgorithmId, RandomEntityKey, RandomPurposeKey, RandomServiceState,
    RandomStreamCheckpoint, RandomStreamKey, RandomSystemKey, RandomWorldKey,
};

use super::{RandomStreamEntry, RandomStreamRegistry};
use crate::core::runtime::random::RandomServiceLimits;
use crate::core::runtime::random::derivation::derive_stream;

const SAMPLE_PAIRS: usize = 31;
const STREAMS_PER_SAMPLE: usize = 4_096;

#[test]
fn optimization_batch_20260829at_runtime320_scope_eviction_preserves_order_and_other_worlds() {
    let removed_world = RandomWorldKey::new(7, 3);
    let retained_world = RandomWorldKey::new(9, 1);
    let checkpoints = (0..8)
        .map(|index| {
            let world = if index % 2 == 0 {
                removed_world
            } else {
                retained_world
            };
            checkpoint(world, index)
        })
        .collect();
    let registry = registry(checkpoints);

    let removed = registry
        .evict_matching(|key| key.world() == removed_world, || 0)
        .expect("idle matching streams should be evicted");

    assert_eq!(removed.len(), 4);
    assert!(removed.windows(2).all(|pair| pair[0].key() < pair[1].key()));
    assert!(
        removed
            .iter()
            .all(|checkpoint| checkpoint.key().world() == removed_world)
    );
    assert_eq!(registry.registered_stream_count(), 4);
    let (_, retained) = registry
        .checkpoint_with_authority_snapshot(authority_snapshot, || {})
        .expect("remaining registry is idle");
    assert!(
        retained
            .iter()
            .all(|checkpoint| checkpoint.key().world() == retained_world)
    );
}

#[test]
fn optimization_batch_20260829at_runtime320_scope_eviction_keeps_registry_when_lease_is_active() {
    let world = RandomWorldKey::new(7, 3);
    let stream_key = key(world, 4);
    let registry = registry(vec![checkpoint(world, 4), checkpoint(world, 8)]);
    let leased = registry
        .acquire(stream_key, || stream(stream_key))
        .expect("fixture stream should be available");

    assert_eq!(
        registry.evict_matching(|candidate| candidate.world() == world, || 0),
        Err(1)
    );
    assert_eq!(registry.registered_stream_count(), 2);

    registry.release(stream_key, leased);
}

#[test]
fn scope_eviction_binds_generation_while_the_registry_lock_is_held() {
    let world = RandomWorldKey::new(7, 3);
    let registry = registry(vec![checkpoint(world, 4)]);

    let removed = registry
        .evict_matching(
            |key| key.world() == world,
            || {
                assert!(registry.lock_is_held_for_test());
                9
            },
        )
        .expect("idle matching stream should be evicted");

    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].master_seed_generation(), 9);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829at_runtime320_linear_random_scope_eviction_bench() {
    let world = RandomWorldKey::new(7, 3);
    let checkpoints = (0..STREAMS_PER_SAMPLE as u64)
        .map(|index| checkpoint(world, index))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let legacy = registry(checkpoints.clone());
        let optimized = registry(checkpoints.clone());
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&legacy, world));
            optimized_samples.push(measure_optimized(&optimized, world));
        } else {
            optimized_samples.push(measure_optimized(&optimized, world));
            legacy_samples.push(measure_legacy(&legacy, world));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME320_LINEAR_RANDOM_SCOPE_EVICTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
streams_per_sample={STREAMS_PER_SAMPLE} legacy_temporary_allocations=2 \
optimized_temporary_allocations=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure_legacy(registry: &RandomStreamRegistry, world: RandomWorldKey) -> u128 {
    let started = Instant::now();
    let removed = legacy_evict_matching(registry, |key| key.world() == world)
        .expect("benchmark registry is idle");
    black_box(removed.len());
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(registry: &RandomStreamRegistry, world: RandomWorldKey) -> u128 {
    let started = Instant::now();
    let removed = registry
        .evict_matching(|key| key.world() == world, || 0)
        .expect("benchmark registry is idle");
    black_box(removed.len());
    started.elapsed().as_nanos().max(1)
}

fn legacy_evict_matching(
    registry: &RandomStreamRegistry,
    matches: impl Fn(RandomStreamKey) -> bool,
) -> Result<Vec<RandomStreamCheckpoint>, usize> {
    let mut state = registry.lock();
    let active_leases = state
        .streams
        .iter()
        .filter(|(key, entry)| matches(**key) && matches!(entry, RandomStreamEntry::Leased))
        .count();
    if active_leases > 0 {
        return Err(active_leases);
    }
    let keys = state
        .streams
        .keys()
        .copied()
        .filter(|key| matches(*key))
        .collect::<Vec<_>>();
    Ok(keys
        .into_iter()
        .filter_map(|key| match state.streams.remove(&key) {
            Some(RandomStreamEntry::Available(stream)) => {
                Some(RandomStreamCheckpoint::new(key, stream.snapshot(), 0))
            }
            _ => None,
        })
        .collect())
}

fn registry(checkpoints: Vec<RandomStreamCheckpoint>) -> RandomStreamRegistry {
    RandomStreamRegistry::from_checkpoints(
        RandomServiceLimits::new(checkpoints.len().saturating_add(1)),
        checkpoints,
    )
    .expect("fixture checkpoints fit the registry")
}

fn checkpoint(world: RandomWorldKey, entity: u64) -> RandomStreamCheckpoint {
    let key = key(world, entity);
    RandomStreamCheckpoint::new(key, stream(key).snapshot(), 0)
}

fn stream(key: RandomStreamKey) -> crate::core::runtime::random::RandomStream {
    derive_stream(RandomAlgorithmId::Pcg32XshRrV1, 0x51a7_2026, 0, key)
}

fn key(world: RandomWorldKey, entity: u64) -> RandomStreamKey {
    RandomStreamKey::for_entity(
        world,
        RandomEntityKey::new(entity, 1),
        RandomSystemKey::new(9),
        RandomPurposeKey::new(3),
        0x5eed,
    )
}

fn authority_snapshot() -> RandomServiceState {
    RandomServiceState::new(RandomAlgorithmId::Pcg32XshRrV1, 0x51a7_2026, 0)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
