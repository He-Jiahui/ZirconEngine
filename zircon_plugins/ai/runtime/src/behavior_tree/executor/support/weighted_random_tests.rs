use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorNodeParameterValue,
    AiBehaviorTreeDescriptor,
};

use crate::behavior_tree::compile_behavior_tree;

use super::{scalar_parameter, weighted_random_child, CompiledBehaviorNode, CompiledBehaviorTree};

const BENCHMARK_CHILD_COUNT: usize = 1_024;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn borrowed_weight_keys_preserve_id_position_and_canonical_decimal_semantics() {
    let tree = weighted_tree(
        3,
        [
            (
                "weight.child_0000",
                AiBehaviorNodeParameterValue::Scalar(0.0),
            ),
            (
                "weight.child_0001",
                AiBehaviorNodeParameterValue::Bool(true),
            ),
            ("weight_1", AiBehaviorNodeParameterValue::Scalar(1.0)),
            ("weight_02", AiBehaviorNodeParameterValue::Scalar(100.0)),
            ("weight_2", AiBehaviorNodeParameterValue::Scalar(2.0)),
        ],
    );
    let node = tree.root();
    let children = tree.child_indices(node);

    for tick in 0..128 {
        assert_eq!(
            weighted_random_child(node, &tree, children, tick),
            legacy_weighted_random_child(node, &tree, children, tick),
        );
    }
}

#[test]
fn weighted_random_child_borrows_parameter_keys_without_formatting_strings() {
    let source = include_str!("../support.rs");
    let weighted_random = source
        .split("pub(super) fn weighted_random_child(")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn parameter").next())
        .expect("weighted_random_child body");

    assert!(weighted_random.contains("borrowed_child_weight("));
    assert!(!weighted_random.contains("format!("));
    assert!(source.contains("fn canonical_index_suffix_matches("));
}

#[test]
#[ignore = "release-only performance evidence"]
fn borrowed_weighted_random_keys_release_benchmark_evidence() {
    let parameters = (0..BENCHMARK_CHILD_COUNT)
        .map(|index| {
            (
                format!("weight.child_{index:04}"),
                AiBehaviorNodeParameterValue::Scalar((index % 7 + 1) as f32),
            )
        })
        .collect::<Vec<_>>();
    let tree = weighted_tree(BENCHMARK_CHILD_COUNT, parameters);
    let node = tree.root();
    let children = tree.child_indices(node);
    assert_eq!(
        legacy_weighted_random_child(node, &tree, children, 41),
        weighted_random_child(node, &tree, children, 41),
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_weighted_random_child(black_box(node), black_box(&tree), children, 41) as u64,
        || weighted_random_child(black_box(node), black_box(&tree), children, 41) as u64,
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    let legacy_key_allocations = BENCHMARK_CHILD_COUNT * 2;

    println!(
        "PERF_RESULT plugins15_borrowed_weighted_random_keys children={BENCHMARK_CHILD_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_key_string_allocations_per_sample={legacy_key_allocations} optimized_key_string_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
    );
}

fn weighted_tree<I, K>(child_count: usize, parameters: I) -> CompiledBehaviorTree
where
    I: IntoIterator<Item = (K, AiBehaviorNodeParameterValue)>,
    K: Into<String>,
{
    let mut root =
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Random Selector")
            .with_implementation("random_selector");
    for (key, value) in parameters {
        root = root.with_parameter(key, value);
    }
    for index in 0..child_count {
        root = root.with_child(format!("child_{index:04}"));
    }
    let mut descriptor =
        AiBehaviorTreeDescriptor::new("weighted", "Weighted", "root").with_node(root);
    for index in 0..child_count {
        descriptor = descriptor.with_node(AiBehaviorNodeDescriptor::new(
            format!("child_{index:04}"),
            AiBehaviorNodeKind::Task,
            format!("Child {index}"),
        ));
    }
    compile_behavior_tree(&descriptor).expect("valid weighted tree")
}

fn legacy_weighted_random_child(
    node: &CompiledBehaviorNode,
    tree: &CompiledBehaviorTree,
    children: &[u32],
    tick: u64,
) -> u32 {
    use std::hash::{Hash, Hasher};

    let weights = children
        .iter()
        .enumerate()
        .map(|(position, child)| {
            let id_key = format!("weight.{}", tree.node(*child as usize).id());
            let position_key = format!("weight_{position}");
            scalar_parameter(node, &[id_key.as_str(), position_key.as_str()])
                .unwrap_or(1.0)
                .max(0.0)
        })
        .collect::<Vec<_>>();
    let total = weights.iter().sum::<f32>();
    if total <= f32::EPSILON {
        return children[0];
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tree.id().hash(&mut hasher);
    node.id().hash(&mut hasher);
    tick.hash(&mut hasher);
    let mut sample = (hasher.finish() as f64 / u64::MAX as f64) as f32 * total;
    for (child, weight) in children.iter().zip(weights) {
        if sample < weight {
            return *child;
        }
        sample -= weight;
    }
    children[children.len() - 1]
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
