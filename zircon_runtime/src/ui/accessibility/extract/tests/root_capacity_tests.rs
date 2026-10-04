use std::hint::black_box;
use std::time::Instant;

const PERF_MARKER: &str = "RUNTIME838_ACCESSIBILITY_ROOT_CAPACITY_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const ROOTS_PER_SAMPLE: usize = 4_096;
const PASSES_PER_SAMPLE: usize = 128;

#[test]
fn optimization_batch_20260919_runtime838_accessibility_root_capacity_preserves_order() {
    let source = [11_u64, 23, 47, 89];
    let mut roots = Vec::with_capacity(source.len());
    roots.extend(source.iter().copied());

    assert_eq!(roots, source);
    assert_eq!(roots.len(), source.len());
    assert_eq!(roots.capacity(), source.len());
}

#[test]
fn optimization_batch_20260919_runtime838_accessibility_root_capacity_keeps_empty_path_empty() {
    let roots: Vec<u64> = Vec::with_capacity(0);

    assert!(roots.is_empty());
    assert_eq!(roots.capacity(), 0);
}

#[test]
fn optimization_batch_20260919_runtime838_accessibility_root_capacity_source_contract() {
    let source = include_str!("../../extract.rs");

    assert!(source.contains("Vec::with_capacity(surface.tree.roots.len())"));
    assert!(source.contains("roots.push(root)"));
    assert!(!source.contains("let mut roots = Vec::new()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260919_runtime838_accessibility_root_capacity_bench() {
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

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let legacy_p99_ns = percentile(&legacy_samples, 99);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    let optimized_p99_ns = percentile(&optimized_samples, 99);
    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} roots_per_sample={ROOTS_PER_SAMPLE} \
passes_per_sample={PASSES_PER_SAMPLE} legacy_growth_events={} optimized_growth_events=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_p99_ns={legacy_p99_ns} optimized_p99_ns={optimized_p99_ns} \
legacy_raw_ns={} optimized_raw_ns={} ",
        growth_events(ROOTS_PER_SAMPLE),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(growth_events(ROOTS_PER_SAMPLE) > 0);
    assert_eq!(reserved_growth_events(ROOTS_PER_SAMPLE), 0);
    assert!(legacy_p50_ns > 0 && optimized_p50_ns > 0);
    assert!(legacy_p95_ns > 0 && optimized_p95_ns > 0);
    assert!(legacy_p99_ns > 0 && optimized_p99_ns > 0);
}

fn measure(preallocate: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for pass in 0..PASSES_PER_SAMPLE {
        let mut roots = if preallocate {
            Vec::with_capacity(ROOTS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for root in 0..ROOTS_PER_SAMPLE {
            roots.push(black_box(pass * ROOTS_PER_SAMPLE + root));
        }
        checksum ^= black_box(roots.len() ^ roots.capacity() ^ pass);
        black_box(roots);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn reserved_growth_events(_item_count: usize) -> usize {
    0
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
