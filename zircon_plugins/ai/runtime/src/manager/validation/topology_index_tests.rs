use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor, AiManagerError,
};

use super::{
    validate_behavior_tree_descriptor, validate_behavior_tree_descriptor_for_compile,
    validate_behavior_tree_topology,
};

const BENCHMARK_NODE_COUNT: usize = 256;
const BENCHMARK_ITERATIONS: usize = 32;
const BENCHMARK_REGISTERED_TREE_COUNT: usize = 512;
const BENCHMARK_SAMPLE_COUNT: usize = 21;
const BENCHMARK_SUBTREE_TARGET_COUNT: usize = 128;

#[test]
fn behavior_topology_uses_one_index_and_dense_visit_state() {
    let source = include_str!("../validation.rs");
    let validation = function_body(
        source,
        "fn validate_behavior_tree_descriptor_inner(",
        "fn validate_behavior_node_child_count(",
    );
    assert!(validation.contains("HashMap::with_capacity(descriptor.nodes.len())"));
    assert!(validation.contains("node_indices.insert(node.id.as_str(), node_index)"));
    assert!(!validation.contains("let mut node_ids = HashSet::new()"));

    let topology = function_body(
        source,
        "fn validate_behavior_tree_topology(",
        "fn invalid_behavior_tree_topology",
    );
    assert!(topology.contains("let mut visit_states = vec![VISIT_UNSEEN; descriptor.nodes.len()]"));
    assert!(topology.contains("let mut incoming_edges = vec![0_usize; descriptor.nodes.len()]"));
    assert!(!topology.contains("collect::<HashMap"));
    assert!(!topology.contains("HashSet::new()"));
}

#[test]
fn indexed_topology_preserves_cycle_diagnostic_precedence() {
    let descriptor = AiBehaviorTreeDescriptor::new("cycle", "Cycle", "a")
        .with_node(
            AiBehaviorNodeDescriptor::new("a", AiBehaviorNodeKind::Sequence, "A").with_child("b"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("b", AiBehaviorNodeKind::Sequence, "B").with_child("a"),
        );

    let error = validate_behavior_tree_descriptor_for_compile(&descriptor)
        .expect_err("cycle must be rejected before root incoming-edge checks");
    assert!(matches!(
        error,
        AiManagerError::InvalidBehaviorTreeTopology {
            node_id,
            reason: "node participates in a cycle",
            ..
        } if node_id == "a"
    ));
}

#[test]
fn subtree_target_validation_uses_one_borrowed_registered_tree_index() {
    let source = include_str!("../validation.rs");
    let validation = function_body(
        source,
        "fn validate_behavior_tree_descriptor_inner(",
        "fn validate_behavior_node_child_count(",
    );
    assert!(validation.contains("let registered_tree_index ="));
    assert!(validation.contains("HashSet::with_capacity(registered_tree_ids.len())"));

    let subtree_validation = function_body(
        source,
        "fn validate_subtree_target_parameter(",
        "fn invalid_subtree_target",
    );
    assert!(subtree_validation.contains("registered_tree_index"));
    assert!(!subtree_validation.contains("registered_tree_ids.contains"));

    let descriptor = subtree_tree(3);
    let registered = ["target-0000", "target-0001"];
    let error = validate_behavior_tree_descriptor(&descriptor, &registered)
        .expect_err("the first unregistered subtree target must be rejected");
    assert!(matches!(
        error,
        AiManagerError::InvalidBehaviorSubtreeTarget { target_tree, .. }
            if target_tree == "target-0002"
    ));
}

#[test]
#[ignore = "release-only performance evidence"]
fn dense_behavior_topology_index_release_benchmark_evidence() {
    let descriptor = benchmark_tree(BENCHMARK_NODE_COUNT);
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                legacy_validate_topology(&descriptor);
            }
        },
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                let mut node_indices = HashMap::with_capacity(descriptor.nodes.len());
                for (node_index, node) in descriptor.nodes.iter().enumerate() {
                    assert!(node_indices.insert(node.id.as_str(), node_index).is_none());
                }
                validate_behavior_tree_topology(&descriptor, &node_indices)
                    .expect("indexed topology validates");
            }
        },
    );
    let metrics = metrics(&legacy_samples, &optimized_samples);
    println!(
        "PERF_RESULT plugins15_dense_behavior_topology_index nodes={BENCHMARK_NODE_COUNT} iterations={BENCHMARK_ITERATIONS} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_hash_container_allocations_per_iteration=5 optimized_hash_container_allocations_per_iteration=1 legacy_dense_state_allocations_per_iteration=0 optimized_dense_state_allocations_per_iteration=2 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        metrics.legacy_p50,
        metrics.legacy_p95,
        metrics.optimized_p50,
        metrics.optimized_p95,
        metrics.legacy_ns,
        metrics.optimized_ns,
    );
    assert!(
        metrics.optimized_p95.saturating_mul(5) <= metrics.legacy_p95.saturating_mul(3),
        "dense topology P95 must be at most 60% of repeated hash topology P95"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn indexed_registered_subtree_targets_release_benchmark_evidence() {
    let registered_storage = (0..BENCHMARK_REGISTERED_TREE_COUNT)
        .map(|index| format!("target-{index:04}"))
        .collect::<Vec<_>>();
    let registered = registered_storage
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let targets = registered
        .iter()
        .rev()
        .take(BENCHMARK_SUBTREE_TARGET_COUNT)
        .copied()
        .collect::<Vec<_>>();
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                for target in &targets {
                    black_box(registered.contains(target));
                }
            }
        },
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                let mut registered_tree_index =
                    HashSet::with_capacity(BENCHMARK_REGISTERED_TREE_COUNT);
                registered_tree_index.extend(registered.iter().copied());
                for target in &targets {
                    black_box(registered_tree_index.contains(target));
                }
            }
        },
    );
    let metrics = metrics(&legacy_samples, &optimized_samples);
    let legacy_key_comparisons = BENCHMARK_SUBTREE_TARGET_COUNT
        * (2 * BENCHMARK_REGISTERED_TREE_COUNT - BENCHMARK_SUBTREE_TARGET_COUNT + 1)
        / 2;
    let optimized_hash_operations =
        BENCHMARK_REGISTERED_TREE_COUNT + BENCHMARK_SUBTREE_TARGET_COUNT;
    println!(
        "PERF_RESULT plugins15_indexed_registered_subtree_targets registered_trees={BENCHMARK_REGISTERED_TREE_COUNT} subtree_targets={BENCHMARK_SUBTREE_TARGET_COUNT} iterations={BENCHMARK_ITERATIONS} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_key_comparisons_per_iteration={legacy_key_comparisons} optimized_hash_operations_per_iteration={optimized_hash_operations} legacy_index_allocations_per_iteration=0 optimized_index_allocations_per_iteration=1 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        metrics.legacy_p50,
        metrics.legacy_p95,
        metrics.optimized_p50,
        metrics.optimized_p95,
        metrics.legacy_ns,
        metrics.optimized_ns,
    );
    assert!(
        metrics.optimized_p95.saturating_mul(4) <= metrics.legacy_p95,
        "indexed subtree target P95 must be at most 25% of slice scan P95"
    );
}

fn benchmark_tree(node_count: usize) -> AiBehaviorTreeDescriptor {
    let mut descriptor = AiBehaviorTreeDescriptor::new("benchmark", "Benchmark", "node-0000");
    for index in 0..node_count {
        let node_id = format!("node-{index:04}");
        let mut node = AiBehaviorNodeDescriptor::new(
            node_id,
            if index + 1 == node_count {
                AiBehaviorNodeKind::Task
            } else {
                AiBehaviorNodeKind::Sequence
            },
            format!("Node {index}"),
        );
        if index + 1 < node_count {
            node = node.with_child(format!("node-{:04}", index + 1));
        }
        descriptor = descriptor.with_node(node);
    }
    descriptor
}

fn subtree_tree(target_count: usize) -> AiBehaviorTreeDescriptor {
    let mut root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Sequence, "Root");
    let mut descriptor = AiBehaviorTreeDescriptor::new("subtrees", "Subtrees", "root");
    for index in 0..target_count {
        let node_id = format!("subtree-{index:04}");
        root = root.with_child(node_id.as_str());
        descriptor = descriptor.with_node(
            AiBehaviorNodeDescriptor::new(
                node_id,
                AiBehaviorNodeKind::Subtree,
                format!("Subtree {index}"),
            )
            .with_parameter("behavior_tree", format!("target-{index:04}")),
        );
    }
    descriptor.with_node(root)
}

fn legacy_validate_topology(descriptor: &AiBehaviorTreeDescriptor) {
    let mut node_ids = HashSet::new();
    for node in &descriptor.nodes {
        assert!(node_ids.insert(node.id.as_str()));
    }
    assert!(node_ids.contains(descriptor.root_node.as_str()));
    let nodes = descriptor
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    legacy_visit(
        descriptor.root_node.as_str(),
        &nodes,
        &mut visiting,
        &mut visited,
    );
    let mut incoming_edges = descriptor
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), 0_usize))
        .collect::<HashMap<_, _>>();
    for node in &descriptor.nodes {
        for child in &node.children {
            *incoming_edges
                .get_mut(child.as_str())
                .expect("benchmark child exists") += 1;
        }
    }
    assert_eq!(visited.len(), descriptor.nodes.len());
    black_box((node_ids, incoming_edges));
}

fn legacy_visit<'a>(
    node_id: &'a str,
    nodes: &HashMap<&'a str, &'a AiBehaviorNodeDescriptor>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
) {
    if visited.contains(node_id) {
        return;
    }
    assert!(visiting.insert(node_id));
    let node = nodes.get(node_id).expect("benchmark node exists");
    for child in &node.children {
        legacy_visit(child, nodes, visiting, visited);
    }
    visiting.remove(node_id);
    visited.insert(node_id);
}

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("function start exists");
    let end = source[start..]
        .find(end)
        .map(|offset| start + offset)
        .expect("function end exists");
    &source[start..end]
}

struct BenchmarkMetrics {
    legacy_p50: u128,
    legacy_p95: u128,
    optimized_p50: u128,
    optimized_p95: u128,
    legacy_ns: String,
    optimized_ns: String,
}

fn metrics(legacy_samples: &[u128], optimized_samples: &[u128]) -> BenchmarkMetrics {
    BenchmarkMetrics {
        legacy_p50: percentile(legacy_samples, 50),
        legacy_p95: percentile(legacy_samples, 95),
        optimized_p50: percentile(optimized_samples, 50),
        optimized_p95: percentile(optimized_samples, 95),
        legacy_ns: samples_csv(legacy_samples),
        optimized_ns: samples_csv(optimized_samples),
    }
}

fn benchmark_paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
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

fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(&result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
