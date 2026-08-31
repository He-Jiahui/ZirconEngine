use std::hint::black_box;
use std::time::Instant;

use super::BehaviorTreeStack;

const BENCHMARK_TREE_DEPTH: usize = 64;
const BENCHMARK_ITERATIONS: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn behavior_tree_stack_reuses_string_and_vector_capacity_after_pop() {
    let mut stack = BehaviorTreeStack::default();
    stack.push("root_tree");
    stack.push("child_tree");
    assert!(stack.contains("root_tree"));
    assert!(stack.contains("child_tree"));
    let ids_allocation = stack.ids.as_ptr();
    let string_allocations = stack
        .ids
        .iter()
        .map(|tree_id| (tree_id.as_ptr(), tree_id.capacity()))
        .collect::<Vec<_>>();

    stack.pop();
    assert!(!stack.contains("child_tree"));
    stack.pop();
    assert_eq!(stack.depth, 0);
    assert_eq!(stack.ids.len(), 2);

    stack.reset();
    stack.push("root");
    stack.push("child");
    assert_eq!(stack.ids.as_ptr(), ids_allocation);
    for (tree_id, (allocation, capacity)) in stack.ids.iter().zip(string_allocations) {
        assert_eq!(tree_id.as_ptr(), allocation);
        assert_eq!(tree_id.capacity(), capacity);
    }
}

#[test]
fn behavior_tree_execution_uses_instance_stack_scratch() {
    let executor = include_str!("../executor.rs");
    let selector = include_str!("selector.rs");

    assert!(executor.contains("tree_stack_scratch: BehaviorTreeStack"));
    assert!(executor.contains("std::mem::take(&mut instance.tree_stack_scratch)"));
    assert!(executor.contains("context.instance.tree_stack_scratch = tree_stack"));
    assert!(executor.contains("tree_stack.push(descriptor.id())"));
    assert!(executor.contains("tree_stack.contains(target_tree_id)"));
    assert!(!executor.contains("tree_stack.push(descriptor.id().to_string())"));
    assert!(!executor.contains("&mut Vec<String>"));
    assert!(selector.contains("tree_stack: &mut super::BehaviorTreeStack"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn reusable_behavior_tree_stack_release_benchmark_evidence() {
    let tree_ids = (0..BENCHMARK_TREE_DEPTH)
        .map(|index| format!("tree_{index:04}"))
        .collect::<Vec<_>>();
    let mut optimized_stack = BehaviorTreeStack::default();
    for tree_id in &tree_ids {
        optimized_stack.push(tree_id);
    }
    assert!(optimized_stack.contains(tree_ids.last().expect("last tree")));
    optimized_stack.reset();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            let mut matches = 0_usize;
            for _ in 0..BENCHMARK_ITERATIONS {
                let mut stack = Vec::new();
                for tree_id in black_box(&tree_ids) {
                    stack.push(tree_id.to_string());
                }
                matches += stack
                    .iter()
                    .any(|candidate| candidate == tree_ids.last().expect("last tree"))
                    as usize;
            }
            matches
        },
        || {
            let mut matches = 0_usize;
            for _ in 0..BENCHMARK_ITERATIONS {
                optimized_stack.reset();
                for tree_id in black_box(&tree_ids) {
                    optimized_stack.push(tree_id);
                }
                matches += optimized_stack.contains(tree_ids.last().expect("last tree")) as usize;
            }
            matches
        },
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    let legacy_string_allocations = BENCHMARK_TREE_DEPTH * BENCHMARK_ITERATIONS;

    println!(
        "PERF_RESULT plugins15_reusable_behavior_tree_stack depth={BENCHMARK_TREE_DEPTH} iterations_per_sample={BENCHMARK_ITERATIONS} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_string_allocations_per_sample={legacy_string_allocations} optimized_string_allocations_per_sample=0 legacy_stack_vec_rebuilds_per_sample={BENCHMARK_ITERATIONS} optimized_stack_vec_rebuilds_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 20% of legacy P95 {legacy_p95}ns"
    );
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
