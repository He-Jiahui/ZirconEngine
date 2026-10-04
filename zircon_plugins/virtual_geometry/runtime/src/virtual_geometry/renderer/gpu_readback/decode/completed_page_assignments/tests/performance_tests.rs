use std::hint::black_box;
use std::time::Instant;

use super::project_completed_assignments;

const BENCH_COMPLETION_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 32;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn completed_assignment_projection_ignores_incomplete_tail_and_preserves_sentinel() {
    let words = [4, 10, 1, u32::MAX, 20, 2, 99, 30, 3];

    assert_eq!(
        project_completed_assignments(&words),
        (vec![(10, 1), (20, 2)], vec![10, 20], vec![(20, 99)])
    );
}

#[test]
#[ignore = "release-only completed assignment projection benchmark"]
fn completed_assignment_projection_release_benchmark_evidence() {
    let mut words = Vec::with_capacity(1 + BENCH_COMPLETION_COUNT * 3);
    words.push(BENCH_COMPLETION_COUNT as u32);
    for index in 0..BENCH_COMPLETION_COUNT as u32 {
        words.extend_from_slice(&[
            index,
            index + 10_000,
            if index % 3 == 0 {
                u32::MAX
            } else {
                index + 20_000
            },
        ]);
    }
    assert_eq!(
        project_completed_assignments(&words),
        legacy_projection(&words)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&words));
        black_box(measure_optimized(&words));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&words));
            optimized_samples.push(measure_optimized(&words));
        } else {
            optimized_samples.push(measure_optimized(&words));
            legacy_samples.push(measure_legacy(&words));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=completed_assignment_single_pass_projection \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
completion_count={BENCH_COMPLETION_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_triplet_copy=full legacy_projection_passes=3 optimized_triplet_copy=none \
optimized_projection_passes=1 optimized_vectors_preallocated=3 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "single-pass completed assignment projection must reduce P95 by at least 25%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(words: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_projection(black_box(words)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(words: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(project_completed_assignments(black_box(words)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_projection(words: &[u32]) -> (Vec<(u32, u32)>, Vec<u32>, Vec<(u32, u32)>) {
    let completed_count = words.first().copied().unwrap_or_default() as usize;
    let triplets = words
        .iter()
        .copied()
        .skip(1)
        .take(completed_count.saturating_mul(3))
        .collect::<Vec<_>>();
    let assignments = triplets
        .chunks_exact(3)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect::<Vec<_>>();
    let page_ids = assignments
        .iter()
        .map(|(page_id, _)| *page_id)
        .collect::<Vec<_>>();
    let replacements = triplets
        .chunks_exact(3)
        .filter_map(|chunk| (chunk[2] != u32::MAX).then_some((chunk[0], chunk[2])))
        .collect::<Vec<_>>();
    (assignments, page_ids, replacements)
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
