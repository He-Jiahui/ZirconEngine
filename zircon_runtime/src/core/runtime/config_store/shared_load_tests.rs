use std::hint::black_box;
use std::sync::{Arc, Barrier};
use std::time::Instant;

use serde_json::Value;

use super::ConfigStore;

const LOADS_PER_THREAD: usize = 512;
const SAMPLE_PAIRS: usize = 17;
const THREAD_COUNT: usize = 4;
const VALUE_STRING_COUNT: usize = 128;

#[test]
fn optimization_batch_fn_runtime470_shared_load_preserves_owned_value_semantics() {
    let store = fixture_store();

    let first = store.load_value("render.profile").expect("stored value");
    let second = store.load_value("render.profile").expect("stored value");

    assert_eq!(first, second);
    assert_eq!(first.as_array().map(Vec::len), Some(VALUE_STRING_COUNT));
    assert!(!std::ptr::eq(&first, &second));
}

#[test]
fn optimization_batch_fn_runtime470_lock_step_clones_only_shared_owner() {
    let store = fixture_store();
    let shared = store.shared_value("render.profile").expect("shared value");
    let values = store.lock_values();
    let stored = values.get("render.profile").expect("stored value");

    assert!(Arc::ptr_eq(&shared, stored));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fn_runtime470_short_config_load_lock_benchmark() {
    let store = fixture_store();
    for _ in 0..3 {
        black_box(measure(&store, legacy_load_value));
        black_box(measure(&store, ConfigStore::load_value));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure(&store, legacy_load_value));
            optimized_samples.push(measure(&store, ConfigStore::load_value));
        } else {
            optimized_samples.push(measure(&store, ConfigStore::load_value));
            legacy_samples.push(measure(&store, legacy_load_value));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME470_SHORT_CONFIG_LOAD_LOCK_BENCH_V1 sample_pairs={SAMPLE_PAIRS} threads={THREAD_COUNT} loads_per_thread={LOADS_PER_THREAD} value_strings={VALUE_STRING_COUNT} legacy_locked_deep_clones={} optimized_locked_deep_clones=0 optimized_locked_arc_clones={} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        THREAD_COUNT * LOADS_PER_THREAD,
        THREAD_COUNT * LOADS_PER_THREAD,
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
        "short config load lock P95 must improve by at least 25%"
    );
}

fn measure(store: &ConfigStore, load: fn(&ConfigStore, &str) -> Option<Value>) -> u128 {
    let barrier = Barrier::new(THREAD_COUNT + 1);
    std::thread::scope(|scope| {
        let mut workers = Vec::with_capacity(THREAD_COUNT);
        for _ in 0..THREAD_COUNT {
            let barrier = &barrier;
            workers.push(scope.spawn(move || {
                barrier.wait();
                let mut checksum = 0_usize;
                for _ in 0..LOADS_PER_THREAD {
                    let value = black_box(load(black_box(store), "render.profile"))
                        .expect("benchmark config value");
                    checksum = checksum.wrapping_add(value.as_array().map(Vec::len).unwrap_or(0));
                    black_box(value);
                }
                checksum
            }));
        }
        let started = Instant::now();
        barrier.wait();
        let checksum = workers
            .into_iter()
            .map(|worker| worker.join().expect("benchmark worker"))
            .fold(0_usize, usize::wrapping_add);
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    })
}

fn legacy_load_value(store: &ConfigStore, key: &str) -> Option<Value> {
    store
        .lock_values()
        .get(key)
        .map(|value| value.as_ref().clone())
}

fn fixture_store() -> ConfigStore {
    let store = ConfigStore::default();
    let value = Value::Array(
        (0..VALUE_STRING_COUNT)
            .map(|index| Value::String(format!("config-value-{index:04}-{}", "x".repeat(48))))
            .collect(),
    );
    store.store_value("render.profile", value);
    store
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
