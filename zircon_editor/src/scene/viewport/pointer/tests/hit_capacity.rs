use std::hint::black_box;
use std::time::Instant;

use super::runtime_pointer_hit_capacity;

const BENCHMARK_CANDIDATE_COUNT: usize = 32_768;
const BENCHMARK_RENDERER_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260915_editor765_runtime_pointer_hit_capacity_covers_both_sources() {
    assert_eq!(runtime_pointer_hit_capacity(0, 0), 0);
    assert_eq!(runtime_pointer_hit_capacity(3, 5), 8);
    assert_eq!(runtime_pointer_hit_capacity(usize::MAX, 1), usize::MAX);
}

#[test]
fn optimization_batch_20260915_editor765_runtime_pointer_hit_capacity_is_an_upper_bound() {
    let stacked_count = 7;
    let renderer_count = 11;
    let capacity = runtime_pointer_hit_capacity(stacked_count, renderer_count);

    assert!(capacity >= stacked_count);
    assert!(capacity >= renderer_count);
    assert_eq!(capacity, stacked_count + renderer_count);
}

fn legacy_hit_capacity(stacked_count: usize, renderer_count: usize) -> usize {
    let mut hits = Vec::new();
    for index in 0..stacked_count.saturating_add(renderer_count) {
        hits.push(index);
    }
    hits.capacity()
}

fn optimized_hit_capacity(stacked_count: usize, renderer_count: usize) -> usize {
    let mut hits = Vec::with_capacity(runtime_pointer_hit_capacity(stacked_count, renderer_count));
    for index in 0..stacked_count.saturating_add(renderer_count) {
        hits.push(index);
    }
    hits.capacity()
}

fn elapsed_nanos(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_nanos()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260915_editor765_runtime_pointer_hit_capacity_p95() {
    let legacy_capacity = legacy_hit_capacity(
        black_box(BENCHMARK_CANDIDATE_COUNT),
        black_box(BENCHMARK_RENDERER_COUNT),
    );
    let optimized_capacity = optimized_hit_capacity(
        black_box(BENCHMARK_CANDIDATE_COUNT),
        black_box(BENCHMARK_RENDERER_COUNT),
    );
    assert!(optimized_capacity <= legacy_capacity);
    assert_eq!(
        optimized_capacity,
        BENCHMARK_CANDIDATE_COUNT + BENCHMARK_RENDERER_COUNT
    );

    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
        let measure_legacy = || {
            elapsed_nanos(|| {
                black_box(legacy_hit_capacity(
                    black_box(BENCHMARK_CANDIDATE_COUNT),
                    black_box(BENCHMARK_RENDERER_COUNT),
                ));
            })
        };
        let measure_optimized = || {
            elapsed_nanos(|| {
                black_box(optimized_hit_capacity(
                    black_box(BENCHMARK_CANDIDATE_COUNT),
                    black_box(BENCHMARK_RENDERER_COUNT),
                ));
            })
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
    println!(
        "EDITOR765_RUNTIME_POINTER_HIT_CAPACITY_BENCH_V1 candidate_count={} renderer_count={} legacy_capacity={} optimized_capacity={} legacy_p95_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        BENCHMARK_CANDIDATE_COUNT,
        BENCHMARK_RENDERER_COUNT,
        legacy_capacity,
        optimized_capacity,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
}
