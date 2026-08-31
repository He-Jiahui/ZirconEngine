use std::{collections::BTreeMap, hint::black_box, time::Instant};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826au_accessibility_focus_preserves_valid_and_fallback_results() {
    let root = UiNodeId::new(1);
    let previous = UiNodeId::new(2);
    let requested = UiNodeId::new(3);
    let mut snapshot = UiAccessibilityTreeSnapshot {
        roots: vec![root],
        nodes: vec![
            node(root, false),
            node(previous, true),
            node(requested, false),
        ],
        focused: Some(requested),
        ..UiAccessibilityTreeSnapshot::default()
    };
    let nodes = node_index(&snapshot);
    let mut diagnostics = Vec::new();

    validate_focus(&mut snapshot, &nodes, &mut diagnostics);

    assert_eq!(snapshot.focused, Some(requested));
    assert!(!snapshot.nodes[1].state.focused);
    assert!(snapshot.nodes[2].state.focused);
    assert!(diagnostics.is_empty());

    snapshot.focused = Some(UiNodeId::new(99));
    validate_focus(&mut snapshot, &nodes, &mut diagnostics);

    assert_eq!(snapshot.focused, Some(root));
    assert!(snapshot.nodes[0].state.focused);
    assert!(!snapshot.nodes[2].state.focused);
    assert_eq!(
        diagnostics.last().map(|diagnostic| diagnostic.code),
        Some(UiAccessibilityDiagnosticCode::ExcludedFocusedNode)
    );
}

#[test]
fn optimization_batch_20260826au_accessibility_focus_uses_existing_node_index() {
    let source = include_str!("../diagnostics.rs");
    let focus = bounded_source(source, "fn validate_focus(", "fn is_interactive(");

    assert!(focus.contains("nodes.get(&focused)"));
    assert!(focus.contains("snapshot.nodes.get_mut("));
    assert!(!focus.contains(".iter_mut().find("));
    assert!(!focus.contains("node.node_id == focused"));
    assert!(!focus.contains("node.node_id == fallback"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826au_accessibility_indexed_focus_p95() {
    const NODE_COUNT: usize = 32_768;
    const VALIDATIONS: usize = 64;
    let focused = UiNodeId::new(NODE_COUNT as u64);
    let snapshot = UiAccessibilityTreeSnapshot {
        roots: vec![UiNodeId::new(1)],
        nodes: (1..=NODE_COUNT)
            .map(|id| node(UiNodeId::new(id as u64), id == 1))
            .collect(),
        focused: Some(focused),
        ..UiAccessibilityTreeSnapshot::default()
    };
    let nodes = node_index(&snapshot);
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        let mut legacy_snapshot = snapshot.clone();
        let mut optimized_snapshot = snapshot.clone();
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(VALIDATIONS, || {
                legacy_validate_focus(black_box(&mut legacy_snapshot), &nodes)
            }));
            optimized_ns.push(measure_ns(VALIDATIONS, || {
                let mut diagnostics = Vec::new();
                validate_focus(black_box(&mut optimized_snapshot), &nodes, &mut diagnostics);
                usize::from(optimized_snapshot.nodes[NODE_COUNT - 1].state.focused)
            }));
        } else {
            optimized_ns.push(measure_ns(VALIDATIONS, || {
                let mut diagnostics = Vec::new();
                validate_focus(black_box(&mut optimized_snapshot), &nodes, &mut diagnostics);
                usize::from(optimized_snapshot.nodes[NODE_COUNT - 1].state.focused)
            }));
            legacy_ns.push(measure_ns(VALIDATIONS, || {
                legacy_validate_focus(black_box(&mut legacy_snapshot), &nodes)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "indexed focus P95 must be at least 25% below post-index linear focus lookup: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME03_ACCESSIBILITY_INDEXED_FOCUS_BENCH_V1 nodes={NODE_COUNT} validations_per_sample={VALIDATIONS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_node_visits_per_sample={} optimized_node_visits_per_sample={} legacy_post_index_linear_scans={VALIDATIONS} optimized_post_index_linear_scans=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        NODE_COUNT * VALIDATIONS * 2,
        NODE_COUNT * VALIDATIONS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn node(node_id: UiNodeId, focused: bool) -> UiAccessibilityNode {
    UiAccessibilityNode {
        node_id,
        state: zircon_runtime_interface::ui::accessibility::UiA11yState {
            focused,
            ..Default::default()
        },
        ..UiAccessibilityNode::default()
    }
}

fn node_index(snapshot: &UiAccessibilityTreeSnapshot) -> BTreeMap<UiNodeId, usize> {
    snapshot
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_id, index))
        .collect()
}

fn legacy_validate_focus(
    snapshot: &mut UiAccessibilityTreeSnapshot,
    nodes: &BTreeMap<UiNodeId, usize>,
) -> usize {
    for node in &mut snapshot.nodes {
        node.state.focused = false;
    }
    let focused = snapshot.focused.expect("focused node");
    let valid = nodes
        .get(&focused)
        .and_then(|index| snapshot.nodes.get(*index))
        .is_some_and(|node| !node.state.hidden && !node.state.disabled);
    if valid {
        if let Some(node) = snapshot
            .nodes
            .iter_mut()
            .find(|node| node.node_id == focused)
        {
            node.state.focused = true;
        }
    }
    usize::from(valid)
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
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
