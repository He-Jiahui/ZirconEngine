use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ia_runtime_tree_lookup_borrows_identity_and_label() {
    let identity = "node/identity/".repeat(256);
    let label = "Node label ".repeat(256);
    let identity_allocation = identity.as_ptr();
    let label_allocation = label.as_ptr();
    let node = UiValue::Map(BTreeMap::from([
        ("id".to_string(), UiValue::String(identity)),
        ("label".to_string(), UiValue::String(label)),
    ]));
    let UiValue::Map(values) = &node else {
        panic!("expected map node");
    };

    assert_eq!(
        tree_node_identity(values).unwrap().as_ptr(),
        identity_allocation
    );
    assert_eq!(
        find_tree_node_label(&node, tree_node_identity(values).unwrap())
            .unwrap()
            .as_ptr(),
        label_allocation
    );
}

#[test]
fn optimization_batch_20260828ia_runtime_focused_tree_lookup_moves_selected_id() {
    let source = include_str!("../editing.rs");
    let focused_target = source
        .split("fn focused_edit_target")
        .nth(1)
        .and_then(|body| body.split("fn clear_editing_state").next())
        .expect("focused edit target implementation");
    let recursive_lookup = source
        .split("fn find_tree_node_label")
        .nth(1)
        .and_then(|body| body.split("fn borrowed_string_value").next())
        .expect("tree node label lookup implementation");

    assert!(focused_target.contains("node_ids.into_iter().nth(index)"));
    assert!(!focused_target.contains("node_ids.get(index)?.clone()"));
    assert!(recursive_lookup.contains("then_some(value.as_str())"));
    assert!(!recursive_lookup.contains("and_then(super::string_value)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ia_runtime_borrowed_tree_node_label_lookup_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let (tree, target_id) = benchmark_tree(2_048, 4 * 1024);

    black_box(legacy_find_tree_node_label(&tree, &target_id));
    black_box(find_tree_node_label(&tree, &target_id));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_find_tree_node_label(
                    black_box(&tree),
                    black_box(&target_id),
                ));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(find_tree_node_label(
                    black_box(&tree),
                    black_box(&target_id),
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME273_BORROWED_TREE_NODE_LABEL_LOOKUP_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_tree(count: usize, field_bytes: usize) -> (UiValue, String) {
    let suffix = "x".repeat(field_bytes);
    let nodes = (0..count)
        .map(|index| {
            UiValue::Map(BTreeMap::from([
                (
                    "id".to_string(),
                    UiValue::String(format!("node-{index:05}-{suffix}")),
                ),
                (
                    "label".to_string(),
                    UiValue::String(format!("Node {index:05} {suffix}")),
                ),
            ]))
        })
        .collect::<Vec<_>>();
    (
        UiValue::Array(nodes),
        format!("node-{:05}-{suffix}", count - 1),
    )
}

fn legacy_find_tree_node_label(value: &UiValue, target_id: &str) -> Option<String> {
    match value {
        UiValue::Array(values) => values
            .iter()
            .find_map(|value| legacy_find_tree_node_label(value, target_id)),
        UiValue::String(value) | UiValue::Enum(value) => {
            (value == target_id).then(|| value.clone())
        }
        UiValue::Map(values) => {
            if legacy_tree_node_identity(values).as_deref() == Some(target_id) {
                return legacy_tree_node_display_text(values);
            }
            for property in ["children", "nodes", "items", "options"] {
                if let Some(value) = values.get(property) {
                    if let Some(label) = legacy_find_tree_node_label(value, target_id) {
                        return Some(label);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn legacy_tree_node_identity(values: &BTreeMap<String, UiValue>) -> Option<String> {
    for property in ["id", "value", "row_id", "rowId", "node_id", "nodeId", "key"] {
        if let Some(value) = values.get(property).and_then(legacy_string_value) {
            return Some(value);
        }
    }
    None
}

fn legacy_tree_node_display_text(values: &BTreeMap<String, UiValue>) -> Option<String> {
    for property in ["label", "text", "name", "title", "id", "value"] {
        if let Some(value) = values.get(property).and_then(legacy_string_value) {
            return Some(value);
        }
    }
    None
}

fn legacy_string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    }
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
