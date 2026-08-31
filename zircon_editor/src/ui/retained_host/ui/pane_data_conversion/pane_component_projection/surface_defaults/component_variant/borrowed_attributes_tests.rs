use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{borrowed_component_string_any, projected_component_variant};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;
const KEYS: &[&str] = &["mui_variant", "component_variant", "variant"];

#[test]
fn optimization_batch_20260826du_editor110_component_variant_preserves_tokens() {
    let attributes = BTreeMap::from([
        (
            "mui_variant".to_string(),
            Value::String("outlined".to_string()),
        ),
        ("animation".to_string(), Value::String("fade".to_string())),
        ("color".to_string(), Value::String("primary".to_string())),
    ]);
    assert_eq!(
        projected_component_variant(&attributes, "timeline-dot"),
        "outlined fade primary"
    );
    let duplicate = BTreeMap::from([
        ("variant".to_string(), Value::String("fade".to_string())),
        ("animation".to_string(), Value::String("fade".to_string())),
    ]);
    assert_eq!(projected_component_variant(&duplicate, "surface"), "fade");
}

#[test]
fn optimization_batch_20260826du_editor110_component_variant_borrows_attributes() {
    let attributes = fixture_attributes();
    let stored = attributes
        .get("component_variant")
        .unwrap()
        .as_str()
        .unwrap();
    let borrowed = borrowed_component_string_any(&attributes, KEYS).unwrap();
    assert_eq!(borrowed.as_ptr(), stored.as_ptr());

    let source = include_str!("../component_variant.rs");
    assert!(source.contains("borrowed_component_string_any("));
    assert!(source
        .contains("if let Some(animation) = borrowed_component_string(attributes, \"animation\")"));
    assert!(!source.contains("and_then(value_as_string)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826du_editor110_component_variant_borrowed_attributes_bench() {
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
        "EDITOR110_COMPONENT_VARIANT_BORROWED_ATTRIBUTES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed component variant lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "component_variant".to_string(),
        Value::String("production_outlined".to_string()),
    )])
}

fn legacy_lookup(attributes: &BTreeMap<String, Value>) -> Option<String> {
    KEYS.iter().find_map(|key| {
        attributes
            .get(*key)
            .and_then(Value::as_str)
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
        checksum ^= black_box(borrowed_component_string_any(black_box(attributes), KEYS))
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
