use std::hint::black_box;
use std::time::Instant;

use super::AbortRequest;

const BENCHMARK_REQUEST_COUNT: usize = 8_192;
const BENCHMARK_SAMPLE_COUNT: usize = 21;
const REUSE_BENCHMARK_REQUEST_COUNT: usize = 32;
const REUSE_BENCHMARK_ITERATIONS: usize = 4_096;

#[test]
fn unstable_abort_request_sort_preserves_unique_priority_order() {
    let requests = benchmark_requests(64);
    assert_eq!(legacy_sort(&requests), optimized_sort(&requests));
    assert!(optimized_sort(&requests)
        .windows(2)
        .all(|pair| pair[0].priority() < pair[1].priority()));
}

#[test]
fn observer_abort_processing_preallocates_and_uses_in_place_sort() {
    let source = include_str!("../abort.rs");
    let processing = source
        .split("pub(super) fn process_observer_aborts(")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn abort_active_root").next())
        .expect("observer abort processing body");

    assert!(processing.contains("std::mem::take(&mut context.instance.abort_request_scratch)"));
    assert!(processing.contains("requests.reserve(observers.len())"));
    assert!(processing.contains("requests.sort_unstable_by_key"));
    assert!(processing.contains("context.instance.abort_request_scratch = requests"));
    assert!(!processing.contains("requests.sort_by_key"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn in_place_abort_request_sort_release_benchmark_evidence() {
    let requests = benchmark_requests(BENCHMARK_REQUEST_COUNT);
    assert_eq!(legacy_sort(&requests), optimized_sort(&requests));

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || black_box(legacy_sort(black_box(&requests))).len(),
        || black_box(optimized_sort(black_box(&requests))).len(),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_in_place_abort_request_sort requests={BENCHMARK_REQUEST_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank priorities=unique legacy_stable_sort=1 optimized_stable_sort=0 optimized_in_place_sort=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn reusable_abort_request_scratch_release_benchmark_evidence() {
    let requests = benchmark_requests(REUSE_BENCHMARK_REQUEST_COUNT);
    let mut scratch = Vec::new();
    optimized_sort_into(&requests, &mut scratch);
    assert_eq!(scratch, legacy_sort(&requests));

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            let mut sorted = 0_usize;
            for _ in 0..REUSE_BENCHMARK_ITERATIONS {
                sorted += black_box(legacy_sort(black_box(&requests))).len();
            }
            sorted
        },
        || {
            let mut sorted = 0_usize;
            for _ in 0..REUSE_BENCHMARK_ITERATIONS {
                optimized_sort_into(black_box(&requests), &mut scratch);
                sorted += black_box(scratch.len());
            }
            sorted
        },
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_reusable_abort_request_scratch requests={REUSE_BENCHMARK_REQUEST_COUNT} iterations_per_sample={REUSE_BENCHMARK_ITERATIONS} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank priorities=unique legacy_request_vec_allocations_per_sample={REUSE_BENCHMARK_ITERATIONS} optimized_request_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
    );
}

fn benchmark_requests(request_count: usize) -> Vec<AbortRequest> {
    (0..request_count as u32)
        .rev()
        .map(|priority| {
            if priority % 2 == 0 {
                AbortRequest::SelfSubtree {
                    node_index: priority,
                }
            } else {
                AbortRequest::LowerPriority {
                    observer_index: priority,
                }
            }
        })
        .collect()
}

fn legacy_sort(requests: &[AbortRequest]) -> Vec<AbortRequest> {
    let mut requests = requests.to_vec();
    requests.sort_by_key(|request| request.priority());
    requests
}

fn optimized_sort(requests: &[AbortRequest]) -> Vec<AbortRequest> {
    let mut requests = requests.to_vec();
    requests.sort_unstable_by_key(|request| request.priority());
    requests
}

fn optimized_sort_into(requests: &[AbortRequest], scratch: &mut Vec<AbortRequest>) {
    scratch.clear();
    scratch.extend_from_slice(requests);
    scratch.sort_unstable_by_key(|request| request.priority());
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
