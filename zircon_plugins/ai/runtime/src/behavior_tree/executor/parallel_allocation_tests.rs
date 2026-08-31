use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

const BENCHMARK_ITEM_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn parallel_evaluation_summarizes_existing_cache_without_cloning_or_result_vec() {
    let source = include_str!("../executor.rs");
    let parallel = source
        .split("fn evaluate_parallel(")
        .nth(1)
        .and_then(|body| body.split("fn evaluate_decorator(").next())
        .expect("evaluate_parallel body");

    assert!(parallel.contains("std::mem::take(&mut state.terminal_children)"));
    assert!(parallel.contains("succeeded_child_count"));
    assert!(parallel.contains("last_succeeded_child"));
    assert!(!parallel.contains("terminal_children\n        .clone()"));
    assert!(!parallel.contains("child_results"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn parallel_terminal_cache_scalar_fold_release_benchmark_evidence() {
    let cache = terminal_cache(BENCHMARK_ITEM_COUNT);
    let mut legacy_cache = cache.clone();
    let mut optimized_cache = cache;
    assert_eq!(
        legacy_parallel_checksum(&mut legacy_cache),
        scalar_parallel_checksum(&mut optimized_cache)
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_parallel_checksum(black_box(&mut legacy_cache)),
        || scalar_parallel_checksum(black_box(&mut optimized_cache)),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_parallel_terminal_cache_scalar_fold entries={BENCHMARK_ITEM_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_entry_clones_per_sample={} optimized_entry_clones_per_sample=0 legacy_result_vec_allocations_per_sample=1 optimized_result_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        BENCHMARK_ITEM_COUNT * 3,
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
    );
}

#[derive(Clone)]
struct SyntheticResult {
    status: u8,
    payload: String,
}

fn terminal_cache(item_count: usize) -> BTreeMap<u32, SyntheticResult> {
    (0..item_count as u32)
        .map(|index| {
            (
                index,
                SyntheticResult {
                    status: (index % 3) as u8,
                    payload: format!("node-{index:04}-{}", "x".repeat(96)),
                },
            )
        })
        .collect()
}

fn legacy_parallel_checksum(cache: &mut BTreeMap<u32, SyntheticResult>) -> u64 {
    let cached = cache.clone();
    let mut child_results = Vec::with_capacity(BENCHMARK_ITEM_COUNT);
    for child in 0..BENCHMARK_ITEM_COUNT as u32 {
        let result = cached.get(&child).cloned().expect("cached child");
        cache.insert(child, result.clone());
        child_results.push(result);
    }
    black_box(&child_results);
    child_results
        .iter()
        .map(|result| u64::from(result.status) + result.payload.len() as u64)
        .sum()
}

fn scalar_parallel_checksum(cache: &mut BTreeMap<u32, SyntheticResult>) -> u64 {
    let cached = std::mem::take(cache);
    let mut status_counts = [0_u64; 3];
    let mut payload_bytes = 0_u64;
    for child in 0..BENCHMARK_ITEM_COUNT as u32 {
        let result = cached.get(&child).expect("cached child");
        status_counts[result.status as usize] += 1;
        payload_bytes += result.payload.len() as u64;
    }
    let checksum = status_counts
        .iter()
        .enumerate()
        .map(|(status, count)| status as u64 * count)
        .sum::<u64>()
        + payload_bytes;
    *cache = cached;
    black_box(cache);
    checksum
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
