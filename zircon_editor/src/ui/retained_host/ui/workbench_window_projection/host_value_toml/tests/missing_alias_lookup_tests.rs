use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::alias_toml_value_key;

const ENTRY_COUNT: usize = 4_096;
const LOOKUP_PASSES: usize = 16_384;
const SAMPLE_PAIRS: usize = 31;
const ALIASES: [(&str, &str); 3] = [
    ("focus_border_color", "border_color"),
    ("thumb_outline_color", "border_color"),
    ("disabled_opacity", "opacity"),
];

#[test]
fn optimization_batch_20260829av_editor267_alias_inserts_source_only_when_target_is_absent() {
    let mut values = BTreeMap::from([(
        "focus_border_color".to_string(),
        toml::Value::String("#123456".to_string()),
    )]);
    alias_toml_value_key(&mut values, "focus_border_color", "border_color");
    assert_eq!(
        values.get("border_color").and_then(toml::Value::as_str),
        Some("#123456")
    );

    values.insert(
        "border_color".to_string(),
        toml::Value::String("#abcdef".to_string()),
    );
    alias_toml_value_key(&mut values, "focus_border_color", "border_color");
    assert_eq!(
        values.get("border_color").and_then(toml::Value::as_str),
        Some("#abcdef")
    );
}

#[test]
fn optimization_batch_20260829av_editor267_missing_alias_source_leaves_values_unchanged() {
    let mut values = benchmark_values();
    let expected = values.clone();

    alias_toml_value_key(&mut values, "focus_border_color", "border_color");

    assert_eq!(values, expected);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829av_editor267_source_first_host_value_alias_bench() {
    let values = benchmark_values();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(values.clone(), false));
            optimized_samples.push(measure(values.clone(), true));
        } else {
            optimized_samples.push(measure(values.clone(), true));
            legacy_samples.push(measure(values.clone(), false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR267_SOURCE_FIRST_HOST_VALUE_ALIAS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
entries={ENTRY_COUNT} lookup_passes={LOOKUP_PASSES} aliases_per_pass=3 \
legacy_tree_lookups_per_missing_alias=2 optimized_tree_lookups_per_missing_alias=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn benchmark_values() -> BTreeMap<String, toml::Value> {
    (0..ENTRY_COUNT)
        .map(|index| {
            let key = format!("unrelated.{index:086}");
            assert_eq!(key.len(), 96);
            (key, toml::Value::Integer(index as i64))
        })
        .collect()
}

fn measure(mut values: BTreeMap<String, toml::Value>, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..LOOKUP_PASSES {
        for (source, target) in ALIASES {
            if optimized {
                alias_toml_value_key(black_box(&mut values), source, target);
            } else {
                legacy_alias_toml_value_key(black_box(&mut values), source, target);
            }
        }
    }
    black_box(values.len());
    started.elapsed().as_nanos().max(1)
}

fn legacy_alias_toml_value_key(
    values: &mut BTreeMap<String, toml::Value>,
    source: &str,
    target: &str,
) {
    if values.contains_key(target) {
        return;
    }
    if let Some(value) = values.get(source).cloned() {
        values.insert(target.to_string(), value);
    }
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
