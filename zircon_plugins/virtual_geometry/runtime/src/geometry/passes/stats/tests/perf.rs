use std::hint::black_box;
use std::time::Instant;

use super::visit_coalesced_execution_copy_ranges;

const BENCH_RECORD_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn coalesced_copy_ranges_preserve_source_and_destination_layout() {
    let mut ranges = Vec::new();
    visit_coalesced_execution_copy_ranges(
        [7, 8, 9, 4, 10, 11, u32::MAX, 0],
        |source, destination, count| ranges.push((source, destination, count)),
    );

    assert_eq!(
        ranges,
        vec![
            (7, 0, 3),
            (4, 3, 1),
            (10, 4, 2),
            (u64::from(u32::MAX), 6, 1),
            (0, 7, 1)
        ]
    );
}

#[test]
#[ignore = "release-only indirect execution copy-range benchmark"]
fn coalesced_execution_copy_ranges_release_benchmark_evidence() {
    let draw_ref_indices = (0..BENCH_RECORD_COUNT as u32).collect::<Vec<_>>();
    let mut optimized_ranges = Vec::new();
    visit_coalesced_execution_copy_ranges(
        draw_ref_indices.iter().copied(),
        |source, destination, count| optimized_ranges.push((source, destination, count)),
    );
    assert_eq!(optimized_ranges, vec![(0, 0, BENCH_RECORD_COUNT as u64)]);

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_legacy(&draw_ref_indices),
        || measure_optimized(&draw_ref_indices),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=coalesced_execution_copy_ranges \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
record_count={BENCH_RECORD_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_copy_commands={BENCH_RECORD_COUNT} optimized_copy_commands=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(4),
        "coalesced execution copy ranges must reduce P95 by at least 20%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn visit_legacy_execution_copy_ranges(
    draw_ref_indices: impl IntoIterator<Item = u32>,
    mut visit: impl FnMut(u64, u64, u64),
) {
    for (destination_index, draw_ref_index) in draw_ref_indices.into_iter().enumerate() {
        visit(draw_ref_index.into(), destination_index as u64, 1);
    }
}

fn benchmark_visit(mut visit_ranges: impl FnMut(&mut dyn FnMut(u64, u64, u64))) -> u64 {
    let mut checksum = 0_u64;
    visit_ranges(&mut |source, destination, count| {
        checksum = black_box(
            checksum
                .wrapping_add(source)
                .wrapping_add(destination)
                .wrapping_add(count),
        );
    });
    checksum
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_legacy(draw_ref_indices: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(benchmark_visit(|visit| {
            visit_legacy_execution_copy_ranges(draw_ref_indices.iter().copied(), visit)
        }));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(draw_ref_indices: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(benchmark_visit(|visit| {
            visit_coalesced_execution_copy_ranges(draw_ref_indices.iter().copied(), visit)
        }));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
