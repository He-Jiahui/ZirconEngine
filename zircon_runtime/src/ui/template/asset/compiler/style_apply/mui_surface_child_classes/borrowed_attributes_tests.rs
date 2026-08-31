use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{
    append_mobile_stepper_classes, borrowed_surface_child_attribute, drawer_anchor, drawer_variant,
    has_open_icon,
};

const SAMPLE_PAIRS: usize = 21;
const LOOKUPS_PER_SAMPLE: usize = 524_288;
const VARIANT_NAMES: &[&str] = &["variant", "mui_variant"];

#[test]
fn optimization_batch_20260826ea_runtime170_surface_child_preserves_defaults_and_classes() {
    let mut node = UiTemplateNode::default();
    append_mobile_stepper_classes(&mut node, "MuiMobileStepper");
    assert!(node
        .classes
        .iter()
        .any(|class| class == "MuiMobileStepper-positionBottom"));

    let attributes = BTreeMap::from([
        (
            "mui_variant".to_string(),
            Value::String("permanent".to_string()),
        ),
        ("open_icon".to_string(), Value::String("close".to_string())),
    ]);
    assert_eq!(drawer_anchor(&attributes), "left");
    assert_eq!(drawer_variant(&attributes), "permanent");
    assert!(has_open_icon(&attributes));
}

#[test]
fn optimization_batch_20260826ea_runtime170_surface_child_borrows_attributes() {
    let attributes = fixture_attributes();
    let stored = attributes.get("variant").unwrap().as_str().unwrap();
    let borrowed = borrowed_surface_child_attribute(&attributes, VARIANT_NAMES).unwrap();
    assert_eq!(borrowed.as_ptr(), stored.as_ptr().wrapping_add(2));

    let source = include_str!("../mui_surface_child_classes.rs");
    assert_eq!(
        source.matches("borrowed_surface_child_attribute(").count(),
        6
    );
    assert!(!source.contains("string_attribute_any"));
    assert!(!source.contains("string_from_attributes_any"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ea_runtime170_surface_child_borrowed_attributes_bench() {
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
        "RUNTIME170_SURFACE_CHILD_BORROWED_ATTRIBUTES_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_allocations_per_lookup=1 \
optimized_allocations_per_lookup=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed surface-child attribute lookup P95 {optimized_p95_ns}ns must be at most 70% of cloned lookup P95 {legacy_p95_ns}ns"
    );
}

fn fixture_attributes() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "variant".to_string(),
        Value::String("  production-permanent  ".to_string()),
    )])
}

fn legacy_lookup(attributes: &BTreeMap<String, Value>) -> Option<String> {
    VARIANT_NAMES.iter().find_map(|name| {
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
        checksum ^= black_box(borrowed_surface_child_attribute(
            black_box(attributes),
            VARIANT_NAMES,
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
