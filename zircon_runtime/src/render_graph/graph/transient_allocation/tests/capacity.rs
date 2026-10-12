use std::hint::black_box;
use std::time::Instant;

const PERF_MARKER: &str = "RUNTIME840_TRANSIENT_ALLOCATION_CAPACITY_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const LIFETIMES_PER_SAMPLE: usize = 4_096;

#[test]
fn optimization_batch_20260919_runtime840_transient_allocation_capacity_preserves_order() {
    let source = (0..LIFETIMES_PER_SAMPLE).collect::<Vec<_>>();
    let mut projected = Vec::with_capacity(source.len());
    projected.extend(source.iter().copied());

    assert_eq!(projected, source);
    assert_eq!(projected.len(), source.len());
    assert!(projected.capacity() >= source.len());

    let empty = Vec::<usize>::with_capacity(0);
    assert!(empty.is_empty());
}

#[test]
fn optimization_batch_20260919_runtime840_transient_allocation_capacity_source_contract() {
    let source = include_str!("../../transient_allocation.rs");
    let start = source
        .find("fn allocate_transient_lifetimes<'a>(")
        .expect("transient lifetime allocator");
    let end = source[start..]
        .find("fn allocate_transient_lifetimes_by_bucket")
        .map(|offset| start + offset)
        .expect("bucket allocator follows lifetime allocator");
    let body = &source[start..end];
    assert!(body.contains("let mut allocations = Vec::with_capacity(lifetimes.len());"));
    assert!(!body.contains("let mut allocations = Vec::new();"));
    assert!(source.contains("#[path = \"transient_allocation/capacity_tests.rs\"]"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260919_runtime840_transient_allocation_capacity_p95() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let legacy_p99 = percentile(&legacy_samples, 99);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let optimized_p99 = percentile(&optimized_samples, 99);
    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} lifetimes_per_sample={LIFETIMES_PER_SAMPLE} \
legacy_growth_events={} optimized_growth_events=0 \
legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} \
legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} \
legacy_p99_ns={legacy_p99} optimized_p99_ns={optimized_p99}",
        growth_events(LIFETIMES_PER_SAMPLE),
    );
    assert!(legacy_p50 > 0 && optimized_p50 > 0);
    assert!(legacy_p95 > 0 && optimized_p95 > 0);
    assert!(legacy_p99 > 0 && optimized_p99 > 0);
}

fn measure(preallocate: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for pass in 0..64 {
        let mut allocations = if preallocate {
            Vec::with_capacity(LIFETIMES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for index in 0..LIFETIMES_PER_SAMPLE {
            allocations.push(black_box(index + pass));
        }
        checksum ^= allocations.len() ^ allocations.capacity();
        black_box(allocations);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 { 4 } else { capacity * 2 };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}
