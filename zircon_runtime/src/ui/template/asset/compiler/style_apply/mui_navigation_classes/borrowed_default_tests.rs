use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::borrowed_navigation_attribute_from_attributes;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;
const NAMES: &[&str] = &["textColor", "text_color"];

#[test]
fn optimization_batch_20260826du_runtime164_mui_navigation_attribute_preserves_alias_default() {
    let attributes = BTreeMap::from([(
        "text_color".to_string(),
        Value::String("  primary  ".to_string()),
    )]);
    assert_eq!(
        borrowed_navigation_attribute_from_attributes(&attributes, NAMES, "inherit"),
        "primary"
    );
    assert_eq!(
        borrowed_navigation_attribute_from_attributes(&BTreeMap::new(), NAMES, "inherit"),
        "inherit"
    );
}

#[test]
fn optimization_batch_20260826du_runtime164_mui_navigation_attribute_borrows_value() {
    let attributes = fixture_attributes();
    let stored = attributes.get("text_color").unwrap().as_str().unwrap();
    let borrowed = borrowed_navigation_attribute_from_attributes(&attributes, NAMES, "inherit");
    assert_eq!(borrowed.as_ptr(), stored.as_ptr().wrapping_add(2));

    let source = include_str!("../mui_navigation_classes.rs");
    assert_eq!(source.matches("borrowed_navigation_attribute(").count(), 7);
    assert_eq!(
        source
            .matches("borrowed_navigation_attribute_from_attributes(")
            .count(),
        4
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826du_runtime164_mui_navigation_borrowed_default_attributes_bench() {
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
        "RUNTIME164_MUI_NAVIGATION_BORROWED_DEFAULT_ATTRIBUTES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples), sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed navigation attribute lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "text_color".to_string(),
        Value::String("  production_primary  ".to_string()),
    )])
}

fn legacy_lookup(attributes: &BTreeMap<String, Value>) -> String {
    NAMES
        .iter()
        .find_map(|name| {
            attributes
                .get(*name)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "inherit".to_string())
}

fn measure_legacy(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(legacy_lookup(black_box(attributes))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(borrowed_navigation_attribute_from_attributes(
            black_box(attributes),
            NAMES,
            "inherit",
        ))
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
