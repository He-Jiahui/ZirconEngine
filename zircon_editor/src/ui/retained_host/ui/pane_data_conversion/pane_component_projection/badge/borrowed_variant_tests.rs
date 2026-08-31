use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{badge_variant, projected_badge_value_text};

const SAMPLE_PAIRS: usize = 21;
const CHECKS_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826ds_editor108_badge_variant_preserves_projection() {
    let mut attributes = BTreeMap::from([
        ("variant".to_string(), Value::String("dot".to_string())),
        ("badgeContent".to_string(), Value::Integer(42)),
    ]);
    assert_eq!(
        projected_badge_value_text("badge", &attributes),
        Some(String::new())
    );

    attributes.insert("variant".to_string(), Value::Integer(7));
    assert_eq!(
        projected_badge_value_text("badge", &attributes),
        Some("42".to_string())
    );
    assert_eq!(projected_badge_value_text("button", &attributes), None);
}

#[test]
fn optimization_batch_20260826ds_editor108_badge_variant_borrows_attribute_text() {
    let attributes = fixture_attributes();
    let stored = attributes.get("mui_variant").unwrap().as_str().unwrap();
    let variant = badge_variant(&attributes);
    assert_eq!(variant, "outlined");
    assert_eq!(variant.as_ptr(), stored.as_ptr());

    let source = include_str!("../badge.rs");
    assert!(source.contains("let variant = badge_variant(attributes);"));
    assert!(source.contains(".and_then(toml::Value::as_str)"));
    assert!(!source.contains(".unwrap_or_else(|| \"standard\".to_string())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ds_editor108_badge_variant_borrowed_lookup_bench() {
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
        "EDITOR108_BADGE_VARIANT_BORROWED_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} legacy_allocations_per_check=1 \
optimized_allocations_per_check=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed badge variant lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([
        (
            "mui_variant".to_string(),
            Value::String("outlined".to_string()),
        ),
        ("badgeContent".to_string(), Value::Integer(17)),
    ])
}

fn legacy_badge_variant(attributes: &BTreeMap<String, Value>) -> String {
    attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| "standard".to_string())
}

fn measure_legacy(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        checksum ^= black_box(legacy_badge_variant(black_box(attributes))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..CHECKS_PER_SAMPLE {
        checksum ^= black_box(badge_variant(black_box(attributes))).len();
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
