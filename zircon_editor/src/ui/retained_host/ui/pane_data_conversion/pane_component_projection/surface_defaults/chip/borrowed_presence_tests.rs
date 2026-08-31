use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{append_chip_variant_tokens, borrowed_chip_string_attribute};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;
const DELETE_ICON_NAMES: &[&str] = &["deleteIcon", "delete_icon"];

#[test]
fn optimization_batch_20260826dx_editor113_chip_presence_preserves_alias_and_type_behavior() {
    let attributes = BTreeMap::from([
        (
            "delete_icon".to_string(),
            Value::String("close".to_string()),
        ),
        ("icon".to_string(), Value::String("star".to_string())),
        ("avatar".to_string(), Value::String(String::new())),
    ]);
    let mut variant = String::new();
    append_chip_variant_tokens(&attributes, &mut variant);
    assert!(variant
        .split_whitespace()
        .any(|token| token == "hasDeleteIcon"));
    assert!(variant.split_whitespace().any(|token| token == "hasIcon"));
    assert!(!variant.split_whitespace().any(|token| token == "hasAvatar"));

    let first_wrong_type = BTreeMap::from([
        ("deleteIcon".to_string(), Value::Boolean(false)),
        (
            "delete_icon".to_string(),
            Value::String("close".to_string()),
        ),
    ]);
    assert_eq!(
        borrowed_chip_string_attribute(&first_wrong_type, DELETE_ICON_NAMES),
        None
    );
}

#[test]
fn optimization_batch_20260826dx_editor113_chip_presence_borrows_value() {
    let attributes = fixture_attributes();
    let stored = attributes.get("delete_icon").unwrap().as_str().unwrap();
    let borrowed = borrowed_chip_string_attribute(&attributes, DELETE_ICON_NAMES).unwrap();
    assert_eq!(borrowed.as_ptr(), stored.as_ptr());

    let source = include_str!("../chip.rs");
    assert_eq!(source.matches("borrowed_chip_string_attribute(").count(), 3);
    assert_eq!(source.matches(".and_then(value_as_string)").count(), 3);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dx_editor113_chip_borrowed_media_presence_bench() {
    let attributes = fixture_attributes();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&attributes));
            optimized_samples.push(measure_optimized(&attributes));
        } else {
            optimized_samples.push(measure_optimized(&attributes));
            legacy_samples.push(measure_legacy(&attributes));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR113_CHIP_BORROWED_MEDIA_PRESENCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed chip media presence lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "delete_icon".to_string(),
        Value::String("production-delete-icon".to_string()),
    )])
}

fn legacy_lookup(attributes: &BTreeMap<String, Value>) -> Option<String> {
    DELETE_ICON_NAMES
        .iter()
        .find_map(|name| attributes.get(*name))
        .and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            _ => None,
        })
}

fn measure_legacy(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(legacy_lookup(black_box(attributes)))
            .unwrap()
            .len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(borrowed_chip_string_attribute(
            black_box(attributes),
            DELETE_ICON_NAMES,
        ))
        .unwrap()
        .len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
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
