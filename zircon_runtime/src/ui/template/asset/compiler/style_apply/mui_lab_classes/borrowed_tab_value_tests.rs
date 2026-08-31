use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::template::UiTemplateNode;

use super::{borrowed_lab_attribute, has_mismatched_tab_value};

const SAMPLE_PAIRS: usize = 21;
const COMPARISONS_PER_SAMPLE: usize = 524_288;
const VALUE_NAMES: &[&str] = &["value", "value_text"];
const SELECTED_NAMES: &[&str] = &[
    "context_value",
    "contextValue",
    "selected_value",
    "selectedValue",
];

#[test]
fn optimization_batch_20260826dw_runtime166_tab_value_preserves_alias_and_mismatch_behavior() {
    let mut node = UiTemplateNode::default();
    node.attributes.insert(
        "value_text".to_string(),
        Value::String("  current-tab  ".to_string()),
    );
    node.attributes.insert(
        "selectedValue".to_string(),
        Value::String("other-tab".to_string()),
    );
    assert!(has_mismatched_tab_value(&node));

    node.attributes.insert(
        "selectedValue".to_string(),
        Value::String("current-tab".to_string()),
    );
    assert!(!has_mismatched_tab_value(&node));
    assert!(!has_mismatched_tab_value(&UiTemplateNode::default()));
}

#[test]
fn optimization_batch_20260826dw_runtime166_tab_value_borrows_attributes() {
    let node = fixture_node();
    let stored = node.attributes.get("value_text").unwrap().as_str().unwrap();
    let borrowed = borrowed_lab_attribute(&node, VALUE_NAMES).unwrap();
    assert_eq!(borrowed.as_ptr(), stored.as_ptr().wrapping_add(2));

    let source = include_str!("../mui_lab_classes.rs");
    let mismatch_start = source.find("fn has_mismatched_tab_value").unwrap();
    let mismatch_end = source[mismatch_start..]
        .find("fn has_opposite_content")
        .map(|offset| mismatch_start + offset)
        .unwrap();
    let mismatch_source = &source[mismatch_start..mismatch_end];
    assert_eq!(
        mismatch_source.matches("borrowed_lab_attribute(").count(),
        2
    );
    assert!(!mismatch_source.contains("string_attribute_any("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dw_runtime166_tab_panel_borrowed_value_comparison_bench() {
    let node = fixture_node();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&node));
            optimized_samples.push(measure_optimized(&node));
        } else {
            optimized_samples.push(measure_optimized(&node));
            legacy_samples.push(measure_legacy(&node));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME166_TAB_PANEL_BORROWED_VALUE_COMPARISON_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
comparisons_per_sample={COMPARISONS_PER_SAMPLE} legacy_allocations_per_comparison=2 \
optimized_allocations_per_comparison=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed tab value comparison P95 {optimized_p95_ns}ns must be at most 70% of cloned comparison P95 {legacy_p95_ns}ns"
    );
}

fn fixture_node() -> UiTemplateNode {
    let mut node = UiTemplateNode::default();
    node.attributes.insert(
        "value_text".to_string(),
        Value::String("  production-current-tab  ".to_string()),
    );
    node.attributes.insert(
        "selectedValue".to_string(),
        Value::String("production-other-tab".to_string()),
    );
    node
}

fn legacy_attribute(node: &UiTemplateNode, names: &[&str]) -> Option<String> {
    names.iter().find_map(|name| {
        node.attributes
            .get(*name)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn legacy_mismatch(node: &UiTemplateNode) -> bool {
    let Some(value) = legacy_attribute(node, VALUE_NAMES) else {
        return false;
    };
    legacy_attribute(node, SELECTED_NAMES).is_some_and(|selected| selected != value)
}

fn measure_legacy(node: &UiTemplateNode) -> u128 {
    let started = Instant::now();
    let mut checksum = false;
    for _ in 0..COMPARISONS_PER_SAMPLE {
        checksum ^= black_box(legacy_mismatch(black_box(node)));
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(node: &UiTemplateNode) -> u128 {
    let started = Instant::now();
    let mut checksum = false;
    for _ in 0..COMPARISONS_PER_SAMPLE {
        checksum ^= black_box(has_mismatched_tab_value(black_box(node)));
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
