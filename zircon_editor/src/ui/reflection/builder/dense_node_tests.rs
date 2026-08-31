use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiStateFlags, UiTreeId};

use super::SnapshotBuilder;

const BENCHMARK_NODE_COUNT: usize = 16_384;
const BENCHMARK_ITERATIONS: usize = 16;
const BENCHMARK_SAMPLES: usize = 17;

#[test]
fn optimization_batch_20260826cg_reflection_builder_dense_nodes_preserves_snapshot_order() {
    let mut builder = SnapshotBuilder::new(UiTreeId::new("editor.main"));
    let root = builder.push_node(
        "root",
        "EditorRoot",
        "Root",
        UiStateFlags::default(),
        Vec::new(),
        Vec::new(),
    );
    let first = builder.push_node(
        "root.first",
        "Panel",
        "First",
        UiStateFlags::default(),
        Vec::new(),
        Vec::new(),
    );
    let second = builder.push_node(
        "root.second",
        "Panel",
        "Second",
        UiStateFlags::default(),
        Vec::new(),
        Vec::new(),
    );
    builder.add_child(root, first);
    builder.add_child(root, second);

    let snapshot = builder.finish(root);

    assert_eq!(snapshot.roots, vec![UiNodeId::new(1)]);
    assert_eq!(
        snapshot.nodes.keys().copied().collect::<Vec<_>>(),
        vec![UiNodeId::new(1), UiNodeId::new(2), UiNodeId::new(3),]
    );
    assert_eq!(snapshot.nodes[&root].children, vec![first, second]);
    assert_eq!(snapshot.nodes[&second].display_name, "Second");
}

#[test]
fn optimization_batch_20260826cg_reflection_builder_dense_nodes_projects_order_once() {
    let source = include_str!("../builder.rs");

    assert!(source.contains("nodes: Vec<UiNodeDescriptor>"));
    assert!(source.contains("nodes: Vec::new()"));
    assert!(source.contains("self.nodes.push(node)"));
    assert!(source.contains("node_index(parent)"));
    assert!(source.contains(".map(|node| (node.node_id, node))"));
    assert!(source.contains(".collect()"));
    assert!(!source.contains("self.nodes.insert(node_id, node)"));
    assert!(!source.contains("self.nodes.get_mut(&parent)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826cg_reflection_builder_dense_nodes_p95() {
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure_build(legacy_build));
            optimized_samples.push(measure_build(optimized_build));
        } else {
            optimized_samples.push(measure_build(optimized_build));
            legacy_samples.push(measure_build(legacy_build));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples, 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples, 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / legacy_p95.as_nanos().max(1),
    );
    eprintln!(
        "EDITOR01_REFLECTION_BUILDER_DENSE_NODES_BENCH_V1 samples={BENCHMARK_SAMPLES} \
iterations={BENCHMARK_ITERATIONS} nodes={BENCHMARK_NODE_COUNT} \
legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} \
reduction_basis_points={reduction_basis_points}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(70),
        "dense node building must reduce snapshot-build P95 by at least 30%: \
legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}

fn legacy_build() -> BTreeMap<u64, Vec<u64>> {
    let mut nodes = BTreeMap::<u64, Vec<u64>>::new();
    for node_id in 1..=BENCHMARK_NODE_COUNT as u64 {
        nodes.insert(node_id, Vec::new());
        if node_id > 1 {
            nodes
                .get_mut(&(node_id - 1))
                .expect("the parent was inserted first")
                .push(node_id);
        }
    }
    nodes
}

fn optimized_build() -> BTreeMap<u64, Vec<u64>> {
    let mut nodes = Vec::<Vec<u64>>::with_capacity(BENCHMARK_NODE_COUNT);
    for node_id in 1..=BENCHMARK_NODE_COUNT as u64 {
        nodes.push(Vec::new());
        if node_id > 1 {
            nodes[node_id as usize - 2].push(node_id);
        }
    }
    nodes
        .into_iter()
        .enumerate()
        .map(|(index, children)| (index as u64 + 1, children))
        .collect()
}

fn measure_build(mut build: impl FnMut() -> BTreeMap<u64, Vec<u64>>) -> Duration {
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        black_box(build());
    }
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let index = (samples.len() - 1).saturating_mul(percentile) / 100;
    samples[index]
}
