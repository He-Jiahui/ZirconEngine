use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use serde_json::Value;

use super::{shared_snapshot_entries, ConfigStore};

const ENTRY_COUNT: usize = 1_024;
const VALUE_BYTES: usize = 8_192;
const SAMPLE_PAIRS: usize = 31;

#[test]
fn optimization_batch_20260829ax_runtime324_shared_config_snapshot_preserves_values() {
    let store = ConfigStore::default();
    store.store_value("render.profile", Value::String("high".to_string()));
    store.store_value("worker.count", Value::from(8));

    let snapshot = store.snapshot_values();

    assert_eq!(
        snapshot.get("render.profile"),
        Some(&Value::String("high".to_string()))
    );
    assert_eq!(snapshot.get("worker.count"), Some(&Value::from(8)));
}

#[test]
fn optimization_batch_20260829ax_runtime324_locked_snapshot_step_shares_json_storage() {
    let values = fixture_values(4);

    let shared = shared_snapshot_entries(&values);

    assert_eq!(shared.len(), values.len());
    for (key, value) in shared {
        assert!(Arc::ptr_eq(
            &value,
            values.get(&key).expect("shared snapshot key")
        ));
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ax_runtime324_short_config_snapshot_lock_bench() {
    let values = fixture_values(ENTRY_COUNT);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&values, false));
            optimized_samples.push(measure(&values, true));
        } else {
            optimized_samples.push(measure(&values, true));
            legacy_samples.push(measure(&values, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME324_SHORT_CONFIG_SNAPSHOT_LOCK_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
entries={ENTRY_COUNT} value_bytes={VALUE_BYTES} \
legacy_locked_deep_value_clones={ENTRY_COUNT} optimized_locked_deep_value_clones=0 \
legacy_locked_arc_clones=0 optimized_locked_arc_clones={ENTRY_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(values: &HashMap<String, Arc<Value>>, optimized: bool) -> u128 {
    let started = Instant::now();
    if optimized {
        black_box(shared_snapshot_entries(black_box(values)));
    } else {
        black_box(legacy_snapshot_values(black_box(values)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_snapshot_values(values: &HashMap<String, Arc<Value>>) -> HashMap<String, Value> {
    values
        .iter()
        .map(|(key, value)| (key.clone(), value.as_ref().clone()))
        .collect()
}

fn fixture_values(count: usize) -> HashMap<String, Arc<Value>> {
    (0..count)
        .map(|index| {
            let prefix = format!("value.{index:08}.");
            let value = format!("{prefix}{}", "x".repeat(VALUE_BYTES - prefix.len()));
            (format!("config.{index:08}"), Arc::new(Value::String(value)))
        })
        .collect()
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
