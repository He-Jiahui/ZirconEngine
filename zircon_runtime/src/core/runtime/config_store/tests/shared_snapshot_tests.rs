use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use serde_json::{json, Value};

use crate::core::CoreRuntime;

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
fn config_persistence_snapshot_source_shares_large_json_and_preserves_sealed_values() {
    const LARGE_PAYLOAD_BYTES: usize = 1_048_576;
    const KEY: &str = "config.\u{5927}\u{503c}";

    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let source = core.config_snapshot_source();
    let original = json!({
        "payload": "x".repeat(LARGE_PAYLOAD_BYTES),
        "nested": {"enabled": true, "labels": ["\u{6807}\u{7b7e}", null, 7]},
    });
    let original_payload = original["payload"].as_str().unwrap().as_ptr();
    core.store_config_value(KEY, original);
    core.store_config_value("other", json!({"revision": 1}));

    let sealed = source();
    let repeated = source();
    let expected = core.snapshot_config_values();
    assert_eq!(sealed.len(), 2);
    for (key, owner) in &sealed {
        assert!(Arc::ptr_eq(owner, repeated.get(key).unwrap()));
    }
    let payload = sealed.get(KEY).unwrap()["payload"].as_str().unwrap();
    assert_eq!(payload.len(), LARGE_PAYLOAD_BYTES);
    assert_eq!(payload.as_ptr(), original_payload);

    let raw_writer = core.clone();
    std::thread::spawn(move || {
        raw_writer.store_config_value(KEY, json!({"payload": "replacement"}));
        raw_writer.store_config_value("later", json!(3));
    })
    .join()
    .unwrap();

    let current = source();
    assert_eq!(current.len(), 3);
    assert!(!Arc::ptr_eq(
        sealed.get(KEY).unwrap(),
        current.get(KEY).unwrap()
    ));
    assert!(!sealed.contains_key("later"));
    let serialized = serde_json::to_vec_pretty(&sealed).unwrap();
    assert_eq!(
        serde_json::from_slice::<HashMap<String, Value>>(&serialized).unwrap(),
        expected
    );

    let weak_core = core.downgrade();
    drop(core);
    drop(runtime);
    assert!(weak_core.upgrade().is_none());
    assert!(Arc::ptr_eq(
        current.get(KEY).unwrap(),
        source().get(KEY).unwrap()
    ));
}

#[test]
fn config_public_owned_snapshots_remain_independent_of_shared_owners() {
    let original = json!({"nested": {"value": "original"}, "items": [1, 2]});
    let store = ConfigStore::default();
    store.store_value("owned", original.clone());
    let shared = store.shared_snapshot_values();
    let mut owned = store.snapshot_values();
    owned.get_mut("owned").unwrap()["nested"]["value"] = json!("edited store snapshot");
    assert_eq!(store.load_value("owned"), Some(original.clone()));
    assert_eq!(shared.get("owned").unwrap().as_ref(), &original);

    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    core.store_config_value("owned", original.clone());
    let source = core.config_snapshot_source();
    let engine_shared = source();
    let mut engine_owned = core.snapshot_config_values();
    engine_owned.get_mut("owned").unwrap()["nested"]["value"] = json!("edited handle snapshot");
    engine_owned.insert("snapshot only".to_owned(), json!(true));
    assert_eq!(core.load_config_value("owned"), Some(original.clone()));
    assert_eq!(engine_shared.get("owned").unwrap().as_ref(), &original);
    assert!(!source().contains_key("snapshot only"));
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
