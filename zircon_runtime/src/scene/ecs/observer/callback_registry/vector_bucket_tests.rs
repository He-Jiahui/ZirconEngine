use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{IndexedObserver, insert_observer_into_bucket, remove_observer_from_indexed_bucket};
use crate::scene::ecs::ObserverId;

const BENCHMARK_OBSERVER_COUNT: usize = 16_384;
const BENCHMARK_ITERATIONS: usize = 256;
const BENCHMARK_SAMPLES: usize = 17;

#[derive(Clone, Debug, PartialEq, Eq)]
struct TestObserver {
    id: ObserverId,
    payload: u64,
}

impl IndexedObserver for TestObserver {
    fn observer_id(&self) -> ObserverId {
        self.id
    }
}

#[test]
fn optimization_batch_20260826cg_observer_vector_bucket_preserves_registration_order() {
    let mut bucket = Arc::<Vec<TestObserver>>::default();
    for index in 0..6_u64 {
        let id = ObserverId::new(index * 3);
        insert_observer_into_bucket(&mut bucket, id, TestObserver { id, payload: index });
    }

    assert!(remove_observer_from_indexed_bucket(
        &mut bucket,
        ObserverId::new(6)
    ));
    assert!(!remove_observer_from_indexed_bucket(
        &mut bucket,
        ObserverId::new(7)
    ));
    assert_eq!(
        bucket
            .iter()
            .map(|observer| (observer.id.index(), observer.payload))
            .collect::<Vec<_>>(),
        vec![(0, 0), (3, 1), (9, 3), (12, 4), (15, 5)]
    );
}

#[test]
fn optimization_batch_20260826cg_observer_vector_bucket_keeps_snapshot_dispatch_contract() {
    let registry_source = include_str!("../callback_registry.rs");
    let entry_source = include_str!("../entry.rs");
    let store_source = include_str!("../store.rs");

    assert!(registry_source.contains("bucket: &mut Arc<Vec<T>>"));
    assert!(registry_source.contains("let observers = Arc::make_mut(bucket)"));
    assert!(registry_source.contains("observers.push(observer)"));
    assert!(registry_source.contains("observers.remove(index)"));
    assert!(entry_source.contains("observers: Arc<Vec<LifecycleObserver>>"));
    assert!(entry_source.contains("observers: Arc<Vec<EventObserver>>"));
    assert!(entry_source.contains("observers: Arc<Vec<EntityEventObserver>>"));
    assert_eq!(
        entry_source
            .matches("for observer in self.observers.iter()")
            .count(),
        3
    );
    assert!(store_source.contains("HashMap<LifecycleObserverKey, Arc<Vec<LifecycleObserver>>>"));
    assert!(store_source.contains("HashMap<TypeId, Arc<Vec<EventObserver>>>"));
    assert!(
        store_source.contains("HashMap<EntityEventObserverKey, Arc<Vec<EntityEventObserver>>>")
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826cg_observer_vector_bucket_p95() {
    let legacy = (0..BENCHMARK_OBSERVER_COUNT as u64)
        .map(|index| (ObserverId::new(index), index.rotate_left(9)))
        .collect::<BTreeMap<_, _>>();
    let optimized = legacy
        .iter()
        .map(|(id, payload)| (*id, *payload))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure_dispatch(|| legacy.values().copied()));
            optimized_samples.push(measure_dispatch(|| {
                optimized.iter().map(|(_, payload)| *payload)
            }));
        } else {
            optimized_samples.push(measure_dispatch(|| {
                optimized.iter().map(|(_, payload)| *payload)
            }));
            legacy_samples.push(measure_dispatch(|| legacy.values().copied()));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples, 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples, 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / legacy_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME60_OBSERVER_VECTOR_BUCKETS_BENCH_V1 samples={BENCHMARK_SAMPLES} \
iterations={BENCHMARK_ITERATIONS} observers={BENCHMARK_OBSERVER_COUNT} \
legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} \
reduction_basis_points={reduction_basis_points}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(70),
        "vector observer traversal must reduce dispatch P95 by at least 30%: \
legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}

fn measure_dispatch<I>(mut values: impl FnMut() -> I) -> Duration
where
    I: Iterator<Item = u64>,
{
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..BENCHMARK_ITERATIONS {
        for value in values() {
            checksum = checksum.wrapping_add(black_box(value));
        }
    }
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let index = (samples.len() - 1).saturating_mul(percentile) / 100;
    samples[index]
}
