use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use accesskit::{Node, NodeId, Role};
use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::accesskit_focus_node_id;

const BENCHMARK_NODE_COUNT: usize = 16_384;
const BENCHMARK_SELECTIONS_PER_SAMPLE: usize = 4;
const BENCHMARK_WARMUP_PAIRS: usize = 4;
const BENCHMARK_SAMPLE_PAIRS: usize = 21;

#[test]
fn runtime78_accesskit_focus_membership_matches_legacy() {
    let nodes = fixture_nodes(64);
    let root = NodeId(0);

    for focused in [Some(UiNodeId::new(63)), Some(UiNodeId::new(128)), None] {
        assert_eq!(
            accesskit_focus_node_id(&nodes, focused, root),
            legacy_accesskit_focus_node_id(&nodes, focused, root),
        );
    }
}

#[test]
#[ignore = "performance acceptance benchmark"]
fn runtime78_accesskit_focus_membership_performance_acceptance() {
    let nodes = fixture_nodes(BENCHMARK_NODE_COUNT);
    let focused = Some(UiNodeId::new((BENCHMARK_NODE_COUNT - 1) as u64));
    let root = NodeId(0);

    for _ in 0..BENCHMARK_WARMUP_PAIRS {
        black_box(time_legacy(&nodes, focused, root));
        black_box(time_optimized(&nodes, focused, root));
    }

    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut legacy_checksum = 0_u64;
    let mut optimized_checksum = 0_u64;
    for pair in 0..BENCHMARK_SAMPLE_PAIRS {
        let ((legacy_ns, legacy_result), (optimized_ns, optimized_result)) = if pair % 2 == 0 {
            (
                time_legacy(&nodes, focused, root),
                time_optimized(&nodes, focused, root),
            )
        } else {
            let optimized = time_optimized(&nodes, focused, root);
            let legacy = time_legacy(&nodes, focused, root);
            (legacy, optimized)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
        legacy_checksum = legacy_checksum.wrapping_add(legacy_result);
        optimized_checksum = optimized_checksum.wrapping_add(optimized_result);
    }

    let legacy_p50_ns = nearest_rank(&legacy_samples, 50);
    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&optimized_samples, 50);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    let legacy_index_entries = BENCHMARK_NODE_COUNT * BENCHMARK_SELECTIONS_PER_SAMPLE;

    println!(
        "RUNTIME78_ACCESSKIT_FOCUS_MEMBERSHIP_PERF nodes={} selections_per_sample={} warmup_pairs={} sample_pairs={} order=alternating percentile=nearest-rank legacy_index_entries={} optimized_index_entries=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_checksum={} optimized_checksum={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        BENCHMARK_NODE_COUNT,
        BENCHMARK_SELECTIONS_PER_SAMPLE,
        BENCHMARK_WARMUP_PAIRS,
        BENCHMARK_SAMPLE_PAIRS,
        legacy_index_entries,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_checksum,
        optimized_checksum,
        legacy_samples,
        optimized_samples,
    );

    assert_eq!(legacy_index_entries, 65_536);
    assert_eq!(legacy_checksum, optimized_checksum);
    assert_ne!(optimized_checksum, 0);
    assert!(
        optimized_p50_ns.saturating_mul(10) <= legacy_p50_ns,
        "direct membership must reduce P50 by at least 90%: legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns",
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(15),
        "direct membership must reduce P95 by at least 85%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns",
    );
}

fn fixture_nodes(count: usize) -> Vec<(NodeId, Node)> {
    (0..count)
        .map(|index| (NodeId(index as u64), Node::new(Role::GenericContainer)))
        .collect()
}

fn legacy_accesskit_focus_node_id(
    nodes: &[(NodeId, Node)],
    focused: Option<UiNodeId>,
    root: NodeId,
) -> NodeId {
    let node_ids = nodes
        .iter()
        .map(|(node_id, _)| *node_id)
        .collect::<BTreeSet<_>>();
    focused
        .map(|focused| NodeId(focused.0))
        .filter(|focused| node_ids.contains(focused))
        .unwrap_or(root)
}

fn time_legacy(nodes: &[(NodeId, Node)], focused: Option<UiNodeId>, root: NodeId) -> (u128, u64) {
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..BENCHMARK_SELECTIONS_PER_SAMPLE {
        checksum = checksum.wrapping_add(
            black_box(legacy_accesskit_focus_node_id(
                black_box(nodes),
                black_box(focused),
                root,
            ))
            .0,
        );
    }
    (started.elapsed().as_nanos(), checksum)
}

fn time_optimized(
    nodes: &[(NodeId, Node)],
    focused: Option<UiNodeId>,
    root: NodeId,
) -> (u128, u64) {
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..BENCHMARK_SELECTIONS_PER_SAMPLE {
        checksum = checksum.wrapping_add(
            black_box(accesskit_focus_node_id(
                black_box(nodes),
                black_box(focused),
                root,
            ))
            .0,
        );
    }
    (started.elapsed().as_nanos(), checksum)
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}
