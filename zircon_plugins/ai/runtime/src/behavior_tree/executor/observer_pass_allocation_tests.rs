use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::BehaviorTreeInstanceState;

const BENCHMARK_TREE_COUNT: usize = 128;
const BENCHMARK_ITERATIONS: usize = 256;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn observer_pass_markers_deduplicate_each_pass_without_reallocating_tree_ids() {
    let mut instance = BehaviorTreeInstanceState::default();
    let first_pass = instance.next_observer_pass();
    assert!(instance.mark_observers_processed("root", first_pass));
    assert!(!instance.mark_observers_processed("root", first_pass));
    assert_eq!(instance.processed_observer_passes.len(), 1);

    let second_pass = instance.next_observer_pass();
    assert!(instance.mark_observers_processed("root", second_pass));
    assert!(!instance.mark_observers_processed("root", second_pass));
    assert_eq!(instance.processed_observer_passes.len(), 1);
}

#[test]
fn observer_pass_epoch_wrap_clears_stale_markers() {
    let mut instance = BehaviorTreeInstanceState::default();
    instance
        .processed_observer_passes
        .insert("stale".to_string(), 1);
    instance.observer_pass_epoch = u64::MAX;

    assert_eq!(instance.next_observer_pass(), 1);
    assert!(instance.processed_observer_passes.is_empty());
    assert!(instance.mark_observers_processed("stale", 1));
}

#[test]
fn observer_abort_processing_uses_persistent_pass_markers() {
    let executor = include_str!("../executor.rs");
    let abort = include_str!("abort.rs");

    assert!(executor.contains("processed_observer_passes: std::collections::HashMap<String, u64>"));
    assert!(executor.contains("fn next_observer_pass(&mut self) -> u64"));
    assert!(executor.contains("fn mark_observers_processed("));
    assert!(abort.contains("let observer_pass = context.observer_pass"));
    assert!(abort.contains("mark_observers_processed(tree.id(), observer_pass)"));
    assert!(!abort.contains("processed_observers.insert(tree.id().to_string())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn persistent_observer_pass_markers_release_benchmark_evidence() {
    let tree_ids = (0..BENCHMARK_TREE_COUNT)
        .map(|index| format!("tree_{index:04}"))
        .collect::<Vec<_>>();
    let mut optimized_instance = BehaviorTreeInstanceState::default();
    let warm_pass = optimized_instance.next_observer_pass();
    for tree_id in &tree_ids {
        assert!(optimized_instance.mark_observers_processed(tree_id, warm_pass));
    }

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            let mut processed = 0_usize;
            for _ in 0..BENCHMARK_ITERATIONS {
                let mut observer_ids = BTreeSet::new();
                for tree_id in black_box(&tree_ids) {
                    processed += observer_ids.insert(tree_id.to_string()) as usize;
                }
            }
            processed
        },
        || {
            let mut processed = 0_usize;
            for _ in 0..BENCHMARK_ITERATIONS {
                let observer_pass = optimized_instance.next_observer_pass();
                for tree_id in black_box(&tree_ids) {
                    processed += optimized_instance.mark_observers_processed(tree_id, observer_pass)
                        as usize;
                }
            }
            processed
        },
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);
    let legacy_string_allocations = BENCHMARK_TREE_COUNT * BENCHMARK_ITERATIONS;

    println!(
        "PERF_RESULT plugins15_persistent_observer_pass_markers trees={BENCHMARK_TREE_COUNT} iterations_per_sample={BENCHMARK_ITERATIONS} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_string_allocations_per_sample={legacy_string_allocations} optimized_string_allocations_per_sample=0 legacy_collection_rebuilds_per_sample={BENCHMARK_ITERATIONS} optimized_collection_rebuilds_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
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
