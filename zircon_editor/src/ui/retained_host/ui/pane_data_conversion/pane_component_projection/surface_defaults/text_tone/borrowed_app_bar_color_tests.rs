use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{borrowed_app_bar_color, projected_text_tone};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826dz_editor115_app_bar_color_preserves_text_tone_behavior() {
    let inherited = BTreeMap::from([("color".to_string(), Value::String("inherit".to_string()))]);
    assert_eq!(projected_text_tone(&inherited, "app-bar", ""), "primary");
    let primary = BTreeMap::from([("color".to_string(), Value::String("primary".to_string()))]);
    assert_eq!(projected_text_tone(&primary, "app-bar", ""), "inverse");
    assert_eq!(borrowed_app_bar_color(&BTreeMap::new()), "primary");
}

#[test]
fn optimization_batch_20260826dz_editor115_app_bar_color_borrows_value() {
    let attributes = fixture_attributes();
    let stored = attributes.get("color").unwrap().as_str().unwrap();
    let borrowed = borrowed_app_bar_color(&attributes);
    assert_eq!(borrowed.as_ptr(), stored.as_ptr());

    let source = include_str!("../text_tone.rs");
    assert!(source.contains("borrowed_app_bar_color(attributes),"));
    assert!(!source.contains("app_bar_color(attributes).as_str()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dz_editor115_text_tone_borrowed_app_bar_color_bench() {
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
        "EDITOR115_TEXT_TONE_BORROWED_APP_BAR_COLOR_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed app bar color lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "color".to_string(),
        Value::String("production-app-bar-color".to_string()),
    )])
}

fn legacy_app_bar_color(attributes: &BTreeMap<String, Value>) -> String {
    attributes
        .get("color")
        .and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            _ => None,
        })
        .filter(|color| !color.is_empty())
        .unwrap_or_else(|| "primary".to_string())
}

fn measure_legacy(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(legacy_app_bar_color(black_box(attributes))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(borrowed_app_bar_color(black_box(attributes))).len();
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
