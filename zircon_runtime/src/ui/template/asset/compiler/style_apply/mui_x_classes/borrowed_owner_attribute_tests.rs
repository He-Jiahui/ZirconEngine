use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::string_attribute_any_from_attributes;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;
const NAMES: &[&str] = &["value", "date_value", "dateValue"];

#[test]
fn optimization_batch_20260826eb_runtime171_mui_x_owner_attribute_preserves_alias_filtering() {
    let attributes = BTreeMap::from([
        ("value".to_string(), Value::String("   ".to_string())),
        (
            "dateValue".to_string(),
            Value::String("  2026-08-26  ".to_string()),
        ),
    ]);
    assert_eq!(
        string_attribute_any_from_attributes(&attributes, NAMES),
        Some("2026-08-26")
    );
    assert_eq!(
        string_attribute_any_from_attributes(&BTreeMap::new(), NAMES),
        None
    );
}

#[test]
fn optimization_batch_20260826eb_runtime171_mui_x_owner_attribute_borrows_value() {
    let attributes = fixture_attributes();
    let stored = attributes.get("date_value").unwrap().as_str().unwrap();
    let borrowed = string_attribute_any_from_attributes(&attributes, NAMES).unwrap();
    assert_eq!(borrowed.as_ptr(), stored.as_ptr().wrapping_add(2));

    let source = include_str!("../mui_x_classes.rs");
    assert_eq!(
        source
            .matches("string_attribute_any_from_attributes(")
            .count(),
        6
    );
    let helper_start = source
        .find("fn string_attribute_any_from_attributes")
        .unwrap();
    let helper_end = source[helper_start..]
        .find("fn number_attribute_any")
        .map(|offset| helper_start + offset)
        .unwrap();
    assert!(!source[helper_start..helper_end].contains("to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eb_runtime171_mui_x_borrowed_owner_attributes_bench() {
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
        "RUNTIME171_MUI_X_BORROWED_OWNER_ATTRIBUTES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed MUI X owner attribute lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "date_value".to_string(),
        Value::String("  production-date-value  ".to_string()),
    )])
}

fn legacy_lookup(attributes: &BTreeMap<String, Value>) -> Option<String> {
    NAMES.iter().find_map(|name| {
        attributes
            .get(*name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
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
        checksum ^= black_box(string_attribute_any_from_attributes(
            black_box(attributes),
            NAMES,
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
