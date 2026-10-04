use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
};

use super::{compile_behavior_tree, CompiledBehaviorTree};

const BENCHMARK_NODE_COUNT: usize = 4_096;
const BENCHMARK_ABORT_PROBE_COUNT: usize = 1_024;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn compiled_parent_index_preserves_root_child_and_grandchild_relationships() {
    let descriptor = AiBehaviorTreeDescriptor::new("nested", "Nested", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("branch"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("branch", AiBehaviorNodeKind::Sequence, "Branch")
                .with_child("leaf"),
        )
        .with_node(AiBehaviorNodeDescriptor::new(
            "leaf",
            AiBehaviorNodeKind::Task,
            "Leaf",
        ));
    let tree = compile_behavior_tree(&descriptor).expect("valid tree");

    assert_eq!(tree.parent_index(0), None);
    assert_eq!(tree.parent_index(1), Some(0));
    assert_eq!(tree.parent_index(2), Some(1));
    assert_eq!(tree.parent_index(3), None, "out of range stays absent");
}

#[test]
fn compiled_tree_owns_a_dense_parent_index() {
    let source = include_str!("../compile.rs");
    let fields = source
        .split("pub struct CompiledBehaviorTree {")
        .nth(1)
        .and_then(|body| body.split("impl CompiledBehaviorTree").next())
        .expect("compiled tree fields");

    assert!(fields.contains("parent_indices: Box<[Option<u32>]>"));
    assert!(source.contains("pub(crate) fn parent_index("));
}

#[test]
fn compiled_tree_caches_whether_any_node_observes_aborts() {
    let without_abort = wide_tree(4);
    assert!(!without_abort.has_abort_observers());

    let descriptor = AiBehaviorTreeDescriptor::new("abort", "Abort", "root").with_node(
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
            .with_abort_policy(AiBehaviorAbortPolicy::Self_),
    );
    let with_abort = compile_behavior_tree(&descriptor).expect("valid abort tree");
    assert!(with_abort.has_abort_observers());
}

#[test]
fn abort_observer_probe_reads_a_compiled_flag_without_scanning_nodes() {
    let source = include_str!("../compile.rs");
    let fields = source
        .split("pub struct CompiledBehaviorTree {")
        .nth(1)
        .and_then(|body| body.split("impl CompiledBehaviorTree").next())
        .expect("compiled tree fields");
    let probe = source
        .split("pub(crate) fn has_abort_observers(")
        .nth(1)
        .and_then(|body| {
            body.split("pub(crate) fn reachable_tree_has_abort_observers")
                .next()
        })
        .expect("has_abort_observers body");

    assert!(fields.contains("has_abort_observers: bool"));
    assert!(probe.contains("self.has_abort_observers"));
    assert!(!probe.contains("self.nodes"));
}

#[test]
fn compiled_subtree_targets_preserve_node_order_and_duplicates() {
    let descriptor = AiBehaviorTreeDescriptor::new("subtrees", "Subtrees", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("first")
                .with_child("second")
                .with_child("third"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("first", AiBehaviorNodeKind::Subtree, "First")
                .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, "target_b"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("second", AiBehaviorNodeKind::Subtree, "Second")
                .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, "target_a"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("third", AiBehaviorNodeKind::Subtree, "Third")
                .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, "target_b"),
        );
    let tree = compile_behavior_tree(&descriptor).expect("valid subtree tree");

    assert_eq!(
        tree.subtree_targets().collect::<Vec<_>>(),
        ["target_b", "target_a", "target_b"]
    );
}

#[test]
fn reachable_tree_traversal_uses_compiled_subtree_targets() {
    let source = include_str!("../compile.rs");
    let fields = source
        .split("pub struct CompiledBehaviorTree {")
        .nth(1)
        .and_then(|body| body.split("impl CompiledBehaviorTree").next())
        .expect("compiled tree fields");
    let reachable = source
        .split("pub(crate) fn reachable_behavior_trees")
        .nth(1)
        .and_then(|body| body.split("#[derive(Debug)]").next())
        .expect("reachable tree traversal");

    assert!(fields.contains("subtree_targets: Box<[String]>"));
    assert!(reachable.contains("tree.subtree_targets()"));
    assert!(!reachable.contains("node.parameters()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn indexed_behavior_parent_lookup_release_benchmark_evidence() {
    let tree = wide_tree(BENCHMARK_NODE_COUNT);
    let expected_checksum = (BENCHMARK_NODE_COUNT - 1) as u64;
    assert_eq!(legacy_parent_checksum(&tree), expected_checksum);
    assert_eq!(indexed_parent_checksum(&tree), expected_checksum);

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_parent_checksum(black_box(&tree)),
        || indexed_parent_checksum(black_box(&tree)),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    let legacy_child_comparisons = BENCHMARK_NODE_COUNT * (BENCHMARK_NODE_COUNT - 1) / 2;
    let optimized_parent_lookups = BENCHMARK_NODE_COUNT - 1;

    println!(
        "PERF_RESULT plugins15_indexed_behavior_parent_lookup nodes={BENCHMARK_NODE_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_child_comparisons_per_sample={legacy_child_comparisons} optimized_parent_lookups_per_sample={optimized_parent_lookups} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn compiled_abort_observer_probe_release_benchmark_evidence() {
    let tree = wide_tree(BENCHMARK_NODE_COUNT);
    assert_eq!(legacy_abort_probe_checksum(&tree), 0);
    assert_eq!(compiled_abort_probe_checksum(&tree), 0);

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_abort_probe_checksum(black_box(&tree)),
        || compiled_abort_probe_checksum(black_box(&tree)),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    let legacy_node_visits = BENCHMARK_NODE_COUNT * BENCHMARK_ABORT_PROBE_COUNT;

    println!(
        "PERF_RESULT plugins15_compiled_abort_observer_probe nodes={BENCHMARK_NODE_COUNT} probes_per_sample={BENCHMARK_ABORT_PROBE_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_node_visits_per_sample={legacy_node_visits} optimized_flag_reads_per_sample={BENCHMARK_ABORT_PROBE_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn compiled_subtree_target_scan_release_benchmark_evidence() {
    let tree = wide_subtree_tree(BENCHMARK_NODE_COUNT);
    let expected_checksum = (BENCHMARK_NODE_COUNT - 1) as u64 * "target".len() as u64;
    assert_eq!(legacy_subtree_target_checksum(&tree), expected_checksum);
    assert_eq!(compiled_subtree_target_checksum(&tree), expected_checksum);

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_subtree_target_checksum(black_box(&tree)),
        || compiled_subtree_target_checksum(black_box(&tree)),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_compiled_subtree_target_scan nodes={BENCHMARK_NODE_COUNT} targets={} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_node_visits_per_sample={BENCHMARK_NODE_COUNT} legacy_parameter_probes_per_sample={} optimized_compiled_target_visits_per_sample={} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        BENCHMARK_NODE_COUNT - 1,
        BENCHMARK_NODE_COUNT - 1,
        BENCHMARK_NODE_COUNT - 1,
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
    );
}

fn wide_tree(node_count: usize) -> CompiledBehaviorTree {
    assert!(node_count >= 2);
    let mut root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root");
    for index in 1..node_count {
        root = root.with_child(format!("leaf_{index:04}"));
    }
    let mut descriptor = AiBehaviorTreeDescriptor::new("wide", "Wide", "root").with_node(root);
    for index in 1..node_count {
        descriptor = descriptor.with_node(AiBehaviorNodeDescriptor::new(
            format!("leaf_{index:04}"),
            AiBehaviorNodeKind::Task,
            format!("Leaf {index}"),
        ));
    }
    compile_behavior_tree(&descriptor).expect("valid wide tree")
}

fn wide_subtree_tree(node_count: usize) -> CompiledBehaviorTree {
    assert!(node_count >= 2);
    let mut root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root");
    for index in 1..node_count {
        root = root.with_child(format!("subtree_{index:04}"));
    }
    let mut descriptor =
        AiBehaviorTreeDescriptor::new("wide_subtrees", "Wide subtrees", "root").with_node(root);
    for index in 1..node_count {
        descriptor = descriptor.with_node(
            AiBehaviorNodeDescriptor::new(
                format!("subtree_{index:04}"),
                AiBehaviorNodeKind::Subtree,
                format!("Subtree {index}"),
            )
            .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, "target"),
        );
    }
    compile_behavior_tree(&descriptor).expect("valid wide subtree tree")
}

fn legacy_parent_checksum(tree: &CompiledBehaviorTree) -> u64 {
    (1..tree.nodes().len() as u32)
        .map(|node_index| {
            legacy_parent_of(tree, node_index)
                .map(|parent| u64::from(parent) + 1)
                .unwrap_or_default()
        })
        .sum()
}

fn legacy_parent_of(tree: &CompiledBehaviorTree, node_index: u32) -> Option<u32> {
    tree.nodes().iter().enumerate().find_map(|(parent, node)| {
        tree.child_indices(node)
            .contains(&node_index)
            .then_some(parent as u32)
    })
}

fn indexed_parent_checksum(tree: &CompiledBehaviorTree) -> u64 {
    (1..tree.nodes().len() as u32)
        .map(|node_index| {
            tree.parent_index(node_index)
                .map(|parent| u64::from(parent) + 1)
                .unwrap_or_default()
        })
        .sum()
}

fn legacy_abort_probe_checksum(tree: &CompiledBehaviorTree) -> u64 {
    (0..BENCHMARK_ABORT_PROBE_COUNT)
        .map(|_| {
            black_box(tree)
                .nodes()
                .iter()
                .any(|node| node.abort_policy() != AiBehaviorAbortPolicy::None) as u64
        })
        .sum()
}

fn compiled_abort_probe_checksum(tree: &CompiledBehaviorTree) -> u64 {
    (0..BENCHMARK_ABORT_PROBE_COUNT)
        .map(|_| black_box(tree).has_abort_observers() as u64)
        .sum()
}

fn legacy_subtree_target_checksum(tree: &CompiledBehaviorTree) -> u64 {
    tree.nodes()
        .iter()
        .filter(|node| node.semantics() == BehaviorNodeSemantics::RunSubtree)
        .filter_map(|node| {
            node.parameters().iter().find_map(|parameter| {
                (parameter.key == SUBTREE_TARGET_PARAMETER_KEY)
                    .then_some(&parameter.value)
                    .and_then(AiBehaviorNodeParameterValue::as_string)
            })
        })
        .map(|target| target.len() as u64)
        .sum()
}

fn compiled_subtree_target_checksum(tree: &CompiledBehaviorTree) -> u64 {
    tree.subtree_targets()
        .map(|target| target.len() as u64)
        .sum()
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
