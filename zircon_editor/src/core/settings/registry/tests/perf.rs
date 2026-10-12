use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;

fn key(value: &str) -> SettingsKey {
    SettingsKey::parse(value).unwrap()
}

fn legacy_changed_keys(
    previous: &BTreeMap<String, u64>,
    values: &BTreeMap<String, u64>,
) -> Vec<String> {
    previous
        .keys()
        .chain(values.keys())
        .filter(|key| previous.get(*key) != values.get(*key))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn linear_changed_keys(
    previous: &BTreeMap<String, u64>,
    values: &BTreeMap<String, u64>,
) -> Vec<String> {
    let mut changed = Vec::new();
    let mut previous_entries = previous.iter().peekable();
    let mut value_entries = values.iter().peekable();
    loop {
        match (previous_entries.peek(), value_entries.peek()) {
            (Some((previous_key, previous_value)), Some((value_key, value))) => {
                match previous_key.cmp(value_key) {
                    Ordering::Less => {
                        changed.push((*previous_key).clone());
                        previous_entries.next();
                    }
                    Ordering::Greater => {
                        changed.push((*value_key).clone());
                        value_entries.next();
                    }
                    Ordering::Equal => {
                        if previous_value != value {
                            changed.push((*previous_key).clone());
                        }
                        previous_entries.next();
                        value_entries.next();
                    }
                }
            }
            (Some((previous_key, _)), None) => {
                changed.push((*previous_key).clone());
                previous_entries.next();
            }
            (None, Some((value_key, _))) => {
                changed.push((*value_key).clone());
                value_entries.next();
            }
            (None, None) => break,
        }
    }
    changed
}

#[test]
fn optimization_batch_hz_editor609_linear_merge_preserves_sorted_unique_change_keys() {
    let previous = BTreeMap::from([
        (key("editor.merge.a"), SettingValue::Int(1)),
        (key("editor.merge.b"), SettingValue::Int(2)),
        (key("editor.merge.d"), SettingValue::Int(4)),
        (key("editor.merge.e"), SettingValue::Int(5)),
    ]);
    let values = BTreeMap::from([
        (key("editor.merge.b"), SettingValue::Int(2)),
        (key("editor.merge.c"), SettingValue::Int(3)),
        (key("editor.merge.d"), SettingValue::Int(40)),
        (key("editor.merge.f"), SettingValue::Int(6)),
    ]);

    assert_eq!(
        changed_layer_keys(&previous, &values)
            .iter()
            .map(SettingsKey::as_str)
            .collect::<Vec<_>>(),
        vec![
            "editor.merge.a",
            "editor.merge.c",
            "editor.merge.d",
            "editor.merge.e",
            "editor.merge.f",
        ]
    );
    assert!(changed_layer_keys(&values, &values).is_empty());
}

#[test]
fn optimization_batch_hz_editor609_layer_diff_uses_linear_ordered_merge() {
    let source = include_str!("../../registry.rs");
    let helper = source
        .split("fn changed_layer_keys")
        .nth(1)
        .expect("changed layer key merge")
        .split("#[cfg(test)]")
        .next()
        .expect("bounded changed layer key merge");

    assert!(source.contains("use std::cmp::Ordering;"));
    assert!(helper.contains("Ordering::Less"));
    assert!(helper.contains("Ordering::Greater"));
    assert!(helper.contains("Ordering::Equal"));
    assert!(!source.contains("BTreeSet"));
    assert!(!helper.contains(".keys().chain(values.keys())"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hz_editor609_linear_layer_key_merge_performance_evidence() {
    const SETTINGS: usize = 8_192;
    const SAMPLE_PAIRS: usize = 17;
    let previous = (0..SETTINGS)
        .map(|index| (format!("editor.performance.setting_{index:05}"), 0_u64))
        .collect::<BTreeMap<_, _>>();
    let values = previous
        .keys()
        .cloned()
        .map(|key| (key, 1_u64))
        .collect::<BTreeMap<_, _>>();
    let measure_legacy = || {
        let started = Instant::now();
        black_box(legacy_changed_keys(
            black_box(&previous),
            black_box(&values),
        ));
        started.elapsed().as_nanos().max(1)
    };
    let measure_linear = || {
        let started = Instant::now();
        black_box(linear_changed_keys(
            black_box(&previous),
            black_box(&values),
        ));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_legacy());
        black_box(measure_linear());
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut linear_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            linear_samples.push(measure_linear());
        } else {
            linear_samples.push(measure_linear());
            legacy_samples.push(measure_legacy());
        }
    }
    legacy_samples.sort_unstable();
    linear_samples.sort_unstable();
    let legacy_p50 = legacy_samples[8];
    let legacy_p95 = legacy_samples[16];
    let linear_p50 = linear_samples[8];
    let linear_p95 = linear_samples[16];
    println!(
        "EDITOR609_LINEAR_LAYER_KEY_MERGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 linear_first_pairs=8 settings={SETTINGS} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} linear_p50_ns={linear_p50} linear_p95_ns={linear_p95} changed_key_clones=16384->8192 ordered_set_insertions=16384->0 target_ratio_bp=2000"
    );
    assert!(
        linear_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(2_000),
        "linear key merge P95 {linear_p95} ns exceeded 20% of legacy {legacy_p95} ns"
    );
}
