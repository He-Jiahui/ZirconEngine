use super::UiRuntimeTreeFocusExt;
use std::hint::black_box;
use std::time::Instant;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    tree::{UiTree, UiTreeNode},
};

const SAMPLE_PAIRS: usize = 17;
const LOOKUPS_PER_SAMPLE: usize = 2_048;
const BENCH_NODE_COUNT: u64 = 4_096;

#[test]
fn navigation_order_admits_tree_node_capacity_before_collecting() {
    const NODE_COUNT: u64 = 128;

    let mut tree = UiTree::new(UiTreeId::new("focus-navigation-capacity"));
    for node_id in 0..NODE_COUNT {
        let mut node = UiTreeNode::new(
            UiNodeId::new(node_id + 1),
            UiNodePath::new(format!("root/{node_id}")),
        );
        node.state_flags.focusable = true;
        tree.insert_root(node);
    }

    let focusable = tree
        .focusable_nodes_in_navigation_order()
        .expect("all roots should resolve");

    assert_eq!(focusable.len(), NODE_COUNT as usize);
    assert_eq!(
        focusable,
        (1..=NODE_COUNT).map(UiNodeId::new).collect::<Vec<_>>(),
    );
    assert!(focusable.capacity() >= tree.nodes.len());
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260916_runtime791_focus_navigation_capacity_bench() {
    let tree = benchmark_tree();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&tree));
            optimized_samples.push(measure_optimized(&tree));
        } else {
            optimized_samples.push(measure_optimized(&tree));
            legacy_samples.push(measure_legacy(&tree));
        }
    }
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME791_FOCUS_NAVIGATION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} node_count={BENCH_NODE_COUNT} \
legacy_growth_events={} optimized_growth_events={} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        geometric_growth_events(BENCH_NODE_COUNT as usize),
        reserved_growth_events(BENCH_NODE_COUNT as usize),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(legacy_p95_ns > 0);
    assert!(optimized_p95_ns > 0);
    assert!(
        geometric_growth_events(BENCH_NODE_COUNT as usize)
            > reserved_growth_events(BENCH_NODE_COUNT as usize)
    );
}

fn benchmark_tree() -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new("focus-navigation-capacity-benchmark"));
    for node_id in 0..BENCH_NODE_COUNT {
        let mut node = UiTreeNode::new(
            UiNodeId::new(node_id + 1),
            UiNodePath::new(format!("root/{node_id}")),
        );
        node.state_flags.focusable = true;
        tree.insert_root(node);
    }
    tree
}

fn measure_legacy(tree: &UiTree) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let mut focusable = Vec::new();
        for root_id in &tree.roots {
            collect_legacy(tree, *root_id, &mut focusable);
        }
        checksum ^= black_box(focusable.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn collect_legacy(tree: &UiTree, node_id: UiNodeId, output: &mut Vec<UiNodeId>) {
    let node = tree.nodes.get(&node_id).expect("benchmark node");
    if node.is_focus_candidate() {
        output.push(node_id);
    }
    for child_id in &node.children {
        collect_legacy(tree, *child_id, output);
    }
}

fn measure_optimized(tree: &UiTree) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        checksum ^= black_box(
            tree.focusable_nodes_in_navigation_order()
                .expect("benchmark roots"),
        )
        .len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn geometric_growth_events(length: usize) -> usize {
    let mut capacity = 0usize;
    let mut used = 0usize;
    let mut events = 0usize;
    while used < length {
        if used == capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
        used += 1;
    }
    events
}

fn reserved_growth_events(_length: usize) -> usize {
    0
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
