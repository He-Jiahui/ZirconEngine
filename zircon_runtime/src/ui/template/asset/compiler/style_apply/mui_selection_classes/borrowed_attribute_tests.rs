use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    autocomplete_has_popup_icon, autocomplete_has_value, borrowed_autocomplete_attribute,
    borrowed_autocomplete_attribute_from_attributes,
};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;
const VALUE_NAMES: &[&str] = &["value", "value_text", "query", "inputValue"];

#[test]
fn optimization_batch_20260826dy_runtime168_autocomplete_preserves_popup_and_value_behavior() {
    let mut node = UiTemplateNode::default();
    node.attributes.insert(
        "force_popup_icon".to_string(),
        Value::String("  false  ".to_string()),
    );
    assert!(!autocomplete_has_popup_icon(&node));

    node.attributes.remove("force_popup_icon");
    node.attributes
        .insert("value".to_string(), Value::String("   ".to_string()));
    node.attributes.insert(
        "inputValue".to_string(),
        Value::String("search text".to_string()),
    );
    assert!(autocomplete_has_value(&node));
    assert!(!autocomplete_has_value(&UiTemplateNode::default()));
}

#[test]
fn optimization_batch_20260826dy_runtime168_autocomplete_borrows_attributes() {
    let attributes = fixture_attributes();
    let stored = attributes.get("value_text").unwrap().as_str().unwrap();
    let borrowed =
        borrowed_autocomplete_attribute_from_attributes(&attributes, VALUE_NAMES).unwrap();
    assert_eq!(borrowed.as_ptr(), stored.as_ptr().wrapping_add(2));

    let mut node = UiTemplateNode::default();
    node.attributes = attributes;
    assert_eq!(
        borrowed_autocomplete_attribute(&node, VALUE_NAMES).unwrap(),
        "production-selection"
    );

    let source = include_str!("../mui_selection_classes.rs");
    assert!(!source.contains("string_attribute_any("));
    assert!(source.contains("borrowed_autocomplete_attribute_from_attributes("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dy_runtime168_autocomplete_borrowed_predicate_attributes_bench() {
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
        "RUNTIME168_AUTOCOMPLETE_BORROWED_PREDICATE_ATTRIBUTES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed autocomplete attribute lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "value_text".to_string(),
        Value::String("  production-selection  ".to_string()),
    )])
}

fn legacy_lookup(attributes: &BTreeMap<String, Value>) -> Option<String> {
    VALUE_NAMES.iter().find_map(|name| {
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
        checksum ^= black_box(borrowed_autocomplete_attribute_from_attributes(
            black_box(attributes),
            VALUE_NAMES,
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
