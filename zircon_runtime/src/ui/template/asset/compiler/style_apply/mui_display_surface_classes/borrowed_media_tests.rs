use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::borrowed_media_attribute_from_attributes;

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;
const MEDIA_NAMES: &[&str] = &["src", "srcSet", "image", "source", "media"];

#[test]
fn optimization_batch_20260826dv_runtime165_avatar_media_preserves_alias_and_empty_behavior() {
    let attributes = BTreeMap::from([
        ("src".to_string(), Value::String("   ".to_string())),
        (
            "image".to_string(),
            Value::String("  avatar.png  ".to_string()),
        ),
    ]);
    assert_eq!(
        borrowed_media_attribute_from_attributes(&attributes, MEDIA_NAMES),
        Some("avatar.png")
    );
    assert_eq!(
        borrowed_media_attribute_from_attributes(&BTreeMap::new(), MEDIA_NAMES),
        None
    );
}

#[test]
fn optimization_batch_20260826dv_runtime165_avatar_media_borrows_value() {
    let attributes = fixture_attributes();
    let stored = attributes.get("image").unwrap().as_str().unwrap();
    let borrowed = borrowed_media_attribute_from_attributes(&attributes, MEDIA_NAMES).unwrap();
    assert_eq!(borrowed.as_ptr(), stored.as_ptr().wrapping_add(2));

    let source = include_str!("../mui_display_surface_classes.rs");
    let avatar_start = source.find("fn append_avatar_classes").unwrap();
    let avatar_end = source[avatar_start..]
        .find("fn append_card_action_area_classes")
        .map(|offset| avatar_start + offset)
        .unwrap();
    let avatar_source = &source[avatar_start..avatar_end];
    assert!(avatar_source.contains("borrowed_media_attribute_from_attributes("));
    assert!(!avatar_source.contains("string_attribute_any("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dv_runtime165_avatar_borrowed_media_presence_bench() {
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
        "RUNTIME165_AVATAR_BORROWED_MEDIA_PRESENCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed avatar media lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "image".to_string(),
        Value::String("  production-avatar.png  ".to_string()),
    )])
}

fn legacy_lookup(attributes: &BTreeMap<String, Value>) -> Option<String> {
    MEDIA_NAMES.iter().find_map(|name| {
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
        checksum ^= black_box(borrowed_media_attribute_from_attributes(
            black_box(attributes),
            MEDIA_NAMES,
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
