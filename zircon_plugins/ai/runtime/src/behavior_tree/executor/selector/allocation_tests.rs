use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::AiDecisionStatus;

use super::BehaviorTreeExecution;

const BENCHMARK_ITEM_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn selector_evaluation_takes_and_restores_terminal_cache_without_cloning_it() {
    let source = include_str!("../selector.rs");
    let selector = source
        .split("pub(super) fn evaluate_selector(")
        .nth(1)
        .and_then(|body| body.split("enum SelectorBranchEligibility").next())
        .expect("evaluate_selector body");

    assert!(selector.contains("std::mem::take(&mut state.terminal_children)"));
    assert!(selector.contains("state.terminal_children = cached"));
    assert!(!selector.contains("terminal_children\n        .clone()"));
}

#[test]
fn parallel_probe_uses_scalar_summary_state_without_result_vectors() {
    let source = include_str!("../selector.rs");
    let probe = source
        .split("fn probe_parallel_children(")
        .nth(1)
        .and_then(|body| body.split("fn fixed_parallel_child_status").next())
        .expect("probe_parallel_children body");

    assert!(probe.contains("eligible_child_count"));
    assert!(probe.contains("fixed_all_failed"));
    assert!(!probe.contains("eligible_children"));
    assert!(!probe.contains("collect::<Option<Vec<_>>>"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn selector_terminal_cache_take_release_benchmark_evidence() {
    let cache = terminal_cache(BENCHMARK_ITEM_COUNT);
    let mut optimized_cache = cache.clone();
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || cloned_cache_checksum(black_box(&cache)),
        || taken_cache_checksum(black_box(&mut optimized_cache)),
    );
    assert_eq!(optimized_cache.len(), cache.len());
    assert_eq!(
        cache_key_checksum(&optimized_cache),
        cache_key_checksum(&cache)
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_selector_terminal_cache_take entries={BENCHMARK_ITEM_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_entry_clones_per_sample={BENCHMARK_ITEM_COUNT} optimized_entry_clones_per_sample=0 legacy_map_allocations_per_sample=1 optimized_map_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn scalar_parallel_probe_summary_release_benchmark_evidence() {
    let children = (0..BENCHMARK_ITEM_COUNT)
        .map(|index| SyntheticParallelChild {
            reactive: index % 2 == 0,
            eligible: index % 3 == 0,
            status: (index % 4) as u8,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_parallel_checksum(&children),
        scalar_parallel_checksum(&children)
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_parallel_checksum(black_box(&children)),
        || scalar_parallel_checksum(black_box(&children)),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_scalar_parallel_probe_summary children={BENCHMARK_ITEM_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_probe_vec_allocations_per_sample=2 optimized_probe_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95 * 7,
        "optimized P95 {optimized_p95}ns must be no more than 70% of legacy P95 {legacy_p95}ns"
    );
}

fn terminal_cache(item_count: usize) -> BTreeMap<u32, BehaviorTreeExecution> {
    (0..item_count as u32)
        .map(|index| {
            (
                index,
                BehaviorTreeExecution {
                    status: AiDecisionStatus::Failed,
                    active_node: Some(format!("node_{index:04}")),
                    diagnostic: None,
                },
            )
        })
        .collect()
}

fn cloned_cache_checksum(cache: &BTreeMap<u32, BehaviorTreeExecution>) -> u64 {
    let cloned = cache.clone();
    black_box(&cloned);
    cloned.keys().map(|value| u64::from(*value) + 1).sum()
}

fn cache_key_checksum(cache: &BTreeMap<u32, BehaviorTreeExecution>) -> u64 {
    cache.keys().map(|value| u64::from(*value) + 1).sum()
}

fn taken_cache_checksum(cache: &mut BTreeMap<u32, BehaviorTreeExecution>) -> u64 {
    let taken = std::mem::take(cache);
    let checksum = taken.keys().map(|value| u64::from(*value) + 1).sum();
    *cache = taken;
    black_box(cache);
    checksum
}

#[derive(Clone, Copy)]
struct SyntheticParallelChild {
    reactive: bool,
    eligible: bool,
    status: u8,
}

fn legacy_parallel_checksum(children: &[SyntheticParallelChild]) -> u64 {
    let eligible = children
        .iter()
        .filter(|child| child.reactive && child.eligible)
        .collect::<Vec<_>>();
    let fixed = children
        .iter()
        .filter(|child| !child.reactive)
        .map(|child| child.status)
        .collect::<Vec<_>>();
    black_box(&eligible);
    black_box(&fixed);
    (eligible.len() + fixed.len()) as u64
        + fixed.iter().map(|status| u64::from(*status)).sum::<u64>()
}

fn scalar_parallel_checksum(children: &[SyntheticParallelChild]) -> u64 {
    let mut eligible_count = 0_usize;
    let mut fixed_count = 0_usize;
    let mut fixed_status_sum = 0_u64;
    for child in children {
        if child.reactive {
            eligible_count += usize::from(child.eligible);
        } else {
            fixed_count += 1;
            fixed_status_sum += u64::from(child.status);
        }
    }
    black_box(eligible_count + fixed_count);
    (eligible_count + fixed_count) as u64 + fixed_status_sum
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
