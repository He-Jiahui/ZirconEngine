use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{skeleton_animation, variant_has_any_token};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826dw_editor112_skeleton_animation_preserves_defaults_and_disabling() {
    let configured = BTreeMap::from([(
        "animation".to_string(),
        Value::String("custom-wave".to_string()),
    )]);
    assert_eq!(skeleton_animation(&configured, "text"), Some("custom-wave"));
    assert_eq!(skeleton_animation(&BTreeMap::new(), "text"), Some("pulse"));
    assert_eq!(
        skeleton_animation(
            &BTreeMap::from([("animation".to_string(), Value::Boolean(false))]),
            "text"
        ),
        None
    );
    assert_eq!(skeleton_animation(&configured, "text wave"), None);
}

#[test]
fn optimization_batch_20260826dw_editor112_skeleton_animation_borrows_value() {
    let attributes = fixture_attributes();
    let stored = attributes.get("animation").unwrap().as_str().unwrap();
    let borrowed = skeleton_animation(&attributes, "text").unwrap();
    assert_eq!(borrowed.as_ptr(), stored.as_ptr());

    let source = include_str!("../skeleton.rs");
    let animation_start = source.find("fn skeleton_animation").unwrap();
    let animation_end = source[animation_start..]
        .find("fn skeleton_has_children")
        .map(|offset| animation_start + offset)
        .unwrap();
    let animation_source = &source[animation_start..animation_end];
    assert!(animation_source.contains(".as_str()"));
    assert!(!animation_source.contains("value_as_string"));
    assert!(!animation_source.contains("to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dw_editor112_skeleton_borrowed_animation_bench() {
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
        "EDITOR112_SKELETON_BORROWED_ANIMATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed skeleton animation lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "animation".to_string(),
        Value::String("production-custom-animation".to_string()),
    )])
}

fn legacy_animation(attributes: &BTreeMap<String, Value>, variant: &str) -> Option<String> {
    if variant_has_any_token(variant, &["pulse", "wave"]) {
        return None;
    }
    attributes.get("animation").and_then(|value| match value {
        Value::Boolean(false) => None,
        Value::String(value) if !value.is_empty() && value != "false" => Some(value.clone()),
        _ => None,
    })
}

fn measure_legacy(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(legacy_animation(black_box(attributes), "text"))
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
        checksum ^= black_box(skeleton_animation(black_box(attributes), "text"))
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
