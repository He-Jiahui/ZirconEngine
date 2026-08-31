use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
};

use super::{
    compile_behavior_tree, reachable_behavior_trees, CompiledBehaviorTree,
    SUBTREE_TARGET_PARAMETER_KEY,
};

const BENCHMARK_NODE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn reachable_abort_probe_preserves_root_target_and_cycle_semantics() {
    let root_abort = task_tree("root_abort", AiBehaviorAbortPolicy::Self_);
    assert!(root_abort.reachable_tree_has_abort_observers(&[]));

    let root = subtree_tree("root", "target", AiBehaviorAbortPolicy::None);
    let target = task_tree("target", AiBehaviorAbortPolicy::LowerPriority);
    assert!(root.reachable_tree_has_abort_observers(std::slice::from_ref(&target)));

    let cycle_a = subtree_tree("cycle_a", "cycle_b", AiBehaviorAbortPolicy::None);
    let cycle_b = subtree_tree("cycle_b", "cycle_a", AiBehaviorAbortPolicy::None);
    assert!(!cycle_a.reachable_tree_has_abort_observers(&[cycle_a.clone(), cycle_b]));
}

#[test]
fn reachable_abort_probe_short_circuits_without_materializing_all_reachable_trees() {
    let source = include_str!("../compile.rs");
    let probe = source
        .split("pub(crate) fn reachable_tree_has_abort_observers(")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn reachable_behavior_trees").next())
        .expect("reachable abort probe");

    assert!(probe.contains("while let Some(tree) = pending.pop()"));
    assert!(probe.contains("if tree.has_abort_observers()"));
    assert!(probe.contains("return true;"));
    assert!(!probe.contains("reachable_behavior_trees(self"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn early_exit_reachable_abort_probe_release_benchmark_evidence() {
    let root = wide_abort_subtree_tree(BENCHMARK_NODE_COUNT);
    let target = task_tree("target", AiBehaviorAbortPolicy::None);
    let registered = [target];
    assert!(legacy_reachable_abort_probe(&root, &registered));
    assert!(root.reachable_tree_has_abort_observers(&registered));

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_reachable_abort_probe(black_box(&root), black_box(&registered)) as u64,
        || black_box(&root).reachable_tree_has_abort_observers(black_box(&registered)) as u64,
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_early_exit_reachable_abort_probe nodes={BENCHMARK_NODE_COUNT} subtree_targets={} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_pending_target_pushes_per_sample={} optimized_pending_target_pushes_per_sample=0 legacy_reachable_vec_allocations_per_sample=1 optimized_reachable_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        BENCHMARK_NODE_COUNT - 1,
        BENCHMARK_NODE_COUNT - 1,
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 1% of legacy P95 {legacy_p95}ns"
    );
}

fn task_tree(id: &str, abort_policy: AiBehaviorAbortPolicy) -> CompiledBehaviorTree {
    let descriptor = AiBehaviorTreeDescriptor::new(id, id, "root").with_node(
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
            .with_abort_policy(abort_policy),
    );
    compile_behavior_tree(&descriptor).expect("valid task tree")
}

fn subtree_tree(
    id: &str,
    target: &str,
    abort_policy: AiBehaviorAbortPolicy,
) -> CompiledBehaviorTree {
    let descriptor = AiBehaviorTreeDescriptor::new(id, id, "root").with_node(
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Root")
            .with_parameter(SUBTREE_TARGET_PARAMETER_KEY, target)
            .with_abort_policy(abort_policy),
    );
    compile_behavior_tree(&descriptor).expect("valid subtree tree")
}

fn wide_abort_subtree_tree(node_count: usize) -> CompiledBehaviorTree {
    assert!(node_count >= 2);
    let mut root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
        .with_abort_policy(AiBehaviorAbortPolicy::Self_);
    for index in 1..node_count {
        root = root.with_child(format!("subtree_{index:04}"));
    }
    let mut descriptor =
        AiBehaviorTreeDescriptor::new("wide_abort", "Wide abort", "root").with_node(root);
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
    compile_behavior_tree(&descriptor).expect("valid wide abort tree")
}

fn legacy_reachable_abort_probe(
    root: &CompiledBehaviorTree,
    registered_trees: &[CompiledBehaviorTree],
) -> bool {
    reachable_behavior_trees(root, registered_trees)
        .into_iter()
        .any(CompiledBehaviorTree::has_abort_observers)
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
