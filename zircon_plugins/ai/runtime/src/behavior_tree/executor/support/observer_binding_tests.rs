use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
    AiBlackboardSchemaDescriptor, AiManagerError,
};

use crate::behavior_tree::{compile_behavior_tree, CompiledBehaviorTree};
use crate::blackboard::BlackboardLayout;

use super::{bind_reachable_observers, BehaviorTreeInstanceState};

const BENCHMARK_TREE_COUNT: usize = 128;
const BENCHMARK_ITERATIONS: usize = 256;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn observer_binding_cache_preserves_reachable_bindings_and_schema_rebinds() {
    let trees = behavior_tree_chain(4);
    let first_layout = layout("schema_a");
    let second_layout = layout("schema_b");
    let mut instance = BehaviorTreeInstanceState::default();

    bind_reachable_observers(&mut instance, &trees[0], &trees, &first_layout)
        .expect("first observer binding");
    assert_eq!(instance.observers.len(), trees.len());
    assert_eq!(instance.observer_binding_root.as_deref(), Some("tree_000"));
    assert_eq!(
        instance.observer_binding_schema.as_deref(),
        Some("schema_a")
    );

    bind_reachable_observers(&mut instance, &trees[0], &trees, &first_layout)
        .expect("cached observer binding");
    assert_eq!(instance.observers.len(), trees.len());

    bind_reachable_observers(&mut instance, &trees[0], &trees, &second_layout)
        .expect("schema observer rebind");
    assert_eq!(instance.observers.len(), trees.len());
    assert!(instance
        .observers
        .values()
        .all(|observers| observers.schema_id() == "schema_b"));
}

#[test]
fn observer_binding_cache_can_be_invalidated_after_tree_catalog_changes() {
    let trees = behavior_tree_chain(2);
    let layout = layout("schema");
    let mut instance = BehaviorTreeInstanceState::default();

    bind_reachable_observers(&mut instance, &trees[0], &trees, &layout)
        .expect("initial observer binding");
    assert!(!instance.observers.is_empty());

    instance.invalidate_observer_bindings();

    assert!(instance.observers.is_empty());
    assert!(instance.observer_binding_root.is_none());
    assert!(instance.observer_binding_schema.is_none());
}

#[test]
fn node_owner_revocation_invalidates_surviving_instance_bindings() {
    let manager = include_str!("../../../manager/behavior_tree.rs");
    let revoke = manager
        .split("pub(super) fn revoke_node_owner(")
        .nth(1)
        .expect("revoke_node_owner body");

    assert!(revoke.contains("instance.invalidate_observer_bindings()"));
}

#[test]
fn steady_observer_binding_checks_cache_before_reachable_tree_walk() {
    let source = include_str!("../support.rs");
    let binding = source
        .split("pub(super) fn bind_reachable_observers(")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn weighted_random_child").next())
        .expect("observer binding body");
    let cache_check = binding
        .find("instance.observer_binding_root")
        .expect("cache check");
    let early_return = binding[cache_check..]
        .find("return Ok(())")
        .expect("early return");
    let reachable_walk = binding
        .find("reachable_behavior_trees")
        .expect("reachable tree walk");

    assert!(cache_check + early_return < reachable_walk);
    assert!(binding.contains("instance.observer_binding_schema"));
    assert!(binding.contains("Some(root.id().to_string())"));
    assert!(binding.contains("Some(layout.schema_id().to_string())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn cached_observer_binding_release_benchmark_evidence() {
    let trees = behavior_tree_chain(BENCHMARK_TREE_COUNT);
    let layout = layout("benchmark");
    let mut legacy_instance = BehaviorTreeInstanceState::default();
    let mut optimized_instance = BehaviorTreeInstanceState::default();
    legacy_bind_reachable_observers(&mut legacy_instance, &trees[0], &trees, &layout)
        .expect("warm legacy bindings");
    bind_reachable_observers(&mut optimized_instance, &trees[0], &trees, &layout)
        .expect("warm optimized bindings");

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                legacy_bind_reachable_observers(
                    black_box(&mut legacy_instance),
                    black_box(&trees[0]),
                    black_box(&trees),
                    black_box(&layout),
                )
                .expect("legacy binding");
            }
            legacy_instance.observers.len()
        },
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                bind_reachable_observers(
                    black_box(&mut optimized_instance),
                    black_box(&trees[0]),
                    black_box(&trees),
                    black_box(&layout),
                )
                .expect("optimized binding");
            }
            optimized_instance.observers.len()
        },
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_cached_observer_binding reachable_trees={BENCHMARK_TREE_COUNT} iterations_per_sample={BENCHMARK_ITERATIONS} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_reachable_walks_per_sample={BENCHMARK_ITERATIONS} optimized_reachable_walks_per_sample=0 legacy_vec_allocations_per_sample={BENCHMARK_ITERATIONS} legacy_hash_set_allocations_per_sample={BENCHMARK_ITERATIONS} optimized_traversal_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
    );
}

fn behavior_tree_chain(tree_count: usize) -> Arc<[CompiledBehaviorTree]> {
    assert!(tree_count > 0);
    (0..tree_count)
        .map(|index| {
            let id = format!("tree_{index:03}");
            let root = if index + 1 < tree_count {
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Subtree")
                    .with_implementation("run_subtree")
                    .with_parameter("behavior_tree", format!("tree_{:03}", index + 1))
            } else {
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Observed")
                    .with_parameter("blackboard_key", "flag")
                    .with_abort_policy(AiBehaviorAbortPolicy::Self_)
            };
            compile_behavior_tree(
                &AiBehaviorTreeDescriptor::new(id, "Tree", "root").with_node(root),
            )
            .expect("valid compiled tree")
        })
        .collect::<Vec<_>>()
        .into()
}

fn layout(id: &str) -> BlackboardLayout {
    BlackboardLayout::from_schema(
        &AiBlackboardSchemaDescriptor::new(id, "Schema").with_key("flag", "bool", false),
    )
    .expect("valid blackboard layout")
}

fn legacy_bind_reachable_observers(
    instance: &mut BehaviorTreeInstanceState,
    root: &CompiledBehaviorTree,
    registered_trees: &[CompiledBehaviorTree],
    layout: &BlackboardLayout,
) -> Result<(), AiManagerError> {
    for tree in crate::behavior_tree::reachable_behavior_trees(root, registered_trees) {
        instance.bind_observers(tree, layout)?;
    }
    Ok(())
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
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

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
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
