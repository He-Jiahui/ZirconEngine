use std::hint::black_box;
use std::time::Instant;

use serde_json::{Map, Value};

use super::insert_unique_object_value;

const PERF_ENTRY_COUNT: usize = 10_000;
const PERF_SAMPLE_PAIRS: usize = 21;

#[test]
#[ignore = "release performance evidence"]
fn binary_object_decode_uses_one_index_lookup_per_entry() {
    let keys = (0..PERF_ENTRY_COUNT)
        .map(|index| format!("field-{index:05}"))
        .collect::<Vec<_>>();
    let mut legacy_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);

    for sample in 0..PERF_SAMPLE_PAIRS {
        let legacy_keys = keys.clone();
        let optimized_keys = keys.clone();
        let (legacy, optimized) = if sample % 2 == 0 {
            (
                measure_once(|| build_legacy_object(legacy_keys)),
                measure_once(|| build_optimized_object(optimized_keys)),
            )
        } else {
            let optimized = measure_once(|| build_optimized_object(optimized_keys));
            let legacy = measure_once(|| build_legacy_object(legacy_keys));
            (legacy, optimized)
        };
        legacy_ns.push(legacy);
        optimized_ns.push(optimized);
    }

    let legacy_p50 = percentile(&legacy_ns, 50);
    let legacy_p95 = percentile(&legacy_ns, 95);
    let optimized_p50 = percentile(&optimized_ns, 50);
    let optimized_p95 = percentile(&optimized_ns, 95);
    println!(
        "PERF_RESULT runtime_interface02_binary_object_entry legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} object_entries={PERF_ENTRY_COUNT} samples={PERF_SAMPLE_PAIRS} legacy_index_lookups_per_entry=2 optimized_index_lookups_per_entry=1"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
        "optimized P95 {optimized_p95}ns must be at most 80% of legacy P95 {legacy_p95}ns"
    );
}

fn build_legacy_object(keys: Vec<String>) -> Map<String, Value> {
    let mut values = Map::new();
    for key in keys {
        assert!(!values.contains_key(&key));
        values.insert(key, Value::Null);
    }
    values
}

fn build_optimized_object(keys: Vec<String>) -> Map<String, Value> {
    let mut values = Map::new();
    for key in keys {
        insert_unique_object_value(&mut values, key, Value::Null)
            .expect("performance fixture keys are unique");
    }
    values
}

fn measure_once<T>(build: impl FnOnce() -> T) -> u64 {
    let started = Instant::now();
    black_box(build());
    started.elapsed().as_nanos() as u64
}

fn percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    ordered[rank]
}
