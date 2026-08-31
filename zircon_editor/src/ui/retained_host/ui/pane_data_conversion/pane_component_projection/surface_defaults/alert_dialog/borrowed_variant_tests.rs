use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{append_alert_variant_tokens, borrowed_alert_variant};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;

#[test]
fn optimization_batch_20260826dy_editor114_alert_variant_preserves_alias_and_default() {
    let aliased = BTreeMap::from([(
        "mui_variant".to_string(),
        Value::String("outlined".to_string()),
    )]);
    let mut variant = String::new();
    append_alert_variant_tokens(&aliased, &mut variant);
    assert!(variant.split_whitespace().any(|token| token == "outlined"));
    assert_eq!(borrowed_alert_variant(&BTreeMap::new()), "standard");

    let wrong_type_primary = BTreeMap::from([
        ("variant".to_string(), Value::Boolean(false)),
        (
            "mui_variant".to_string(),
            Value::String("filled".to_string()),
        ),
    ]);
    assert_eq!(borrowed_alert_variant(&wrong_type_primary), "standard");
}

#[test]
fn optimization_batch_20260826dy_editor114_alert_variant_borrows_value() {
    let attributes = fixture_attributes();
    let stored = attributes.get("variant").unwrap().as_str().unwrap();
    let borrowed = borrowed_alert_variant(&attributes);
    assert_eq!(borrowed.as_ptr(), stored.as_ptr());

    let source = include_str!("../alert_dialog.rs");
    let helper_start = source.find("fn borrowed_alert_variant").unwrap();
    let helper_end = source[helper_start..]
        .find("fn alert_has_visible_icon")
        .map(|offset| helper_start + offset)
        .unwrap();
    let helper_source = &source[helper_start..helper_end];
    assert!(!helper_source.contains("value_as_string"));
    assert!(!helper_source.contains("to_string()"));
    assert!(source.contains("append_variant_token(variant, alert_variant)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dy_editor114_alert_borrowed_variant_bench() {
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
        "EDITOR114_ALERT_BORROWED_VARIANT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed alert variant lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "variant".to_string(),
        Value::String("production-outlined".to_string()),
    )])
}

fn legacy_variant(attributes: &BTreeMap<String, Value>) -> String {
    attributes
        .get("variant")
        .or_else(|| attributes.get("mui_variant"))
        .and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            _ => None,
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "standard".to_string())
}

fn measure_legacy(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(legacy_variant(black_box(attributes))).len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(attributes: &BTreeMap<String, Value>) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(borrowed_alert_variant(black_box(attributes))).len();
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
