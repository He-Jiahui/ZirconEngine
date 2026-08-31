use std::{collections::BTreeMap, hint::black_box, time::Instant};

use zircon_runtime_interface::ui::component::UiValue;

use super::{
    TREE_CHILD_PROPERTIES, flattened_tree_node_position, insert_tree_node_child,
    reparent_tree_node_values, tree_node_map_id,
};

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826am_tree_reparent_preserves_order_and_rejects_cycles() {
    let values = vec![
        tree_node("source", vec![tree_node("child", Vec::new())]),
        tree_node("parent", Vec::new()),
    ];

    let result = reparent_tree_node_values(values.clone(), "source", "parent")
        .expect("source should move below parent");
    assert_eq!(result.from, 0);
    assert_eq!(result.to, 1);
    assert_eq!(result.parent_id, "parent");
    assert_eq!(
        flattened_tree_node_position(&result.values, "source"),
        Some(1)
    );
    assert_eq!(
        flattened_tree_node_position(&result.values, "child"),
        Some(2)
    );
    assert!(reparent_tree_node_values(values, "source", "child").is_none());
}

#[test]
fn optimization_batch_20260826am_tree_reparent_uses_borrowed_single_owner_traversal() {
    let source = include_str!("../tree_view_reparent.rs");

    assert!(source.contains("source: &mut Option<UiValue>"));
    assert!(source.contains("let Some(source) = source.take() else"));
    assert!(source.contains("fn tree_node_id(value: &UiValue) -> Option<&str>"));
    assert!(source.contains("fn flattened_tree_node_position("));
    assert!(!source.contains("source.clone()"));
    assert!(!source.contains("flattened_tree_node_ids("));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826am_tree_reparent_borrowed_traversal_p95() {
    const CANDIDATE_PARENTS: usize = 2_048;
    const SOURCE_CHILDREN: usize = 64;
    let target_id = format!("candidate-{}", CANDIDATE_PARENTS - 1);
    let values = (0..CANDIDATE_PARENTS)
        .map(|index| tree_node(&format!("candidate-{index}"), Vec::new()))
        .collect::<Vec<_>>();
    let source = tree_node(
        "source",
        (0..SOURCE_CHILDREN)
            .map(|index| tree_node(&format!("source-child-{index}"), Vec::new()))
            .collect(),
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_legacy(values.clone(), &target_id, source.clone()));
            optimized_ns.push(measure_optimized(
                values.clone(),
                &target_id,
                source.clone(),
            ));
        } else {
            optimized_ns.push(measure_optimized(
                values.clone(),
                &target_id,
                source.clone(),
            ));
            legacy_ns.push(measure_legacy(values.clone(), &target_id, source.clone()));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns,
        "borrowed reparent insertion P95 must be at least 80% below clone-per-candidate traversal: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME77_TREE_REPARENT_BORROWED_TRAVERSAL_BENCH_V1 candidate_parents={CANDIDATE_PARENTS} source_nodes={} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_source_clones={} optimized_source_clones=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        SOURCE_CHILDREN + 1,
        CANDIDATE_PARENTS - 1,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn tree_node(id: &str, children: Vec<UiValue>) -> UiValue {
    let mut values = BTreeMap::new();
    values.insert("id".to_string(), UiValue::String(id.to_string()));
    values.insert("children".to_string(), UiValue::Array(children));
    UiValue::Map(values)
}

fn measure_legacy(mut values: Vec<UiValue>, target_id: &str, source: UiValue) -> u128 {
    let started = Instant::now();
    assert!(legacy_insert_tree_node_child(
        black_box(&mut values),
        black_box(target_id),
        black_box(source)
    ));
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(flattened_tree_node_position(&values, "source"), Some(2_048));
    elapsed
}

fn measure_optimized(mut values: Vec<UiValue>, target_id: &str, source: UiValue) -> u128 {
    let mut source = Some(source);
    let started = Instant::now();
    assert!(insert_tree_node_child(
        black_box(&mut values),
        black_box(target_id),
        black_box(&mut source)
    ));
    let elapsed = started.elapsed().as_nanos();
    assert!(source.is_none());
    assert_eq!(flattened_tree_node_position(&values, "source"), Some(2_048));
    elapsed
}

fn legacy_insert_tree_node_child(values: &mut [UiValue], parent_id: &str, source: UiValue) -> bool {
    for value in values {
        if let UiValue::Map(node) = value {
            if tree_node_map_id(node) == Some(parent_id) {
                let child_property = TREE_CHILD_PROPERTIES
                    .iter()
                    .copied()
                    .find(|property| matches!(node.get(*property), Some(UiValue::Array(_))))
                    .unwrap_or("children");
                let children = node
                    .entry(child_property.to_string())
                    .or_insert_with(|| UiValue::Array(Vec::new()));
                if let UiValue::Array(children) = children {
                    children.push(source);
                    return true;
                }
                return false;
            }
            for property in TREE_CHILD_PROPERTIES {
                if let Some(UiValue::Array(children)) = node.get_mut(property) {
                    if legacy_insert_tree_node_child(children, parent_id, source.clone()) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
