use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::project_page_table_entries;

const BENCH_ASSIGNMENT_COUNT: usize = 8_192;
const BENCH_RESIDENT_COUNT: usize = 2_048;
const CHECKS_PER_SAMPLE: usize = 8;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn page_table_projection_preserves_duplicate_nonresident_assignment_count() {
    let words = [10, 100, 11, 101, 12, 102, 13, 103, 14, 104];
    let resident_slots = [10, 11, 11];
    let assignments = [(20, 11), (21, 20), (22, 20)];

    assert_eq!(
        project_page_table_entries(&words, 2, &resident_slots, &assignments),
        vec![(10, 100), (11, 101), (12, 102), (13, 103)]
    );
}

#[test]
#[ignore = "release-only page-table decode membership benchmark"]
fn page_table_projection_release_benchmark_evidence() {
    let resident_slots = (0..BENCH_RESIDENT_COUNT as u32).collect::<Vec<_>>();
    let assignments = (0..BENCH_ASSIGNMENT_COUNT as u32)
        .map(|index| {
            let slot = if index % 2 == 0 {
                index % BENCH_RESIDENT_COUNT as u32
            } else {
                index + BENCH_RESIDENT_COUNT as u32
            };
            (index, slot)
        })
        .collect::<Vec<_>>();
    let words = (0..BENCH_RESIDENT_COUNT as u32)
        .flat_map(|index| [index, index + 10_000])
        .collect::<Vec<_>>();
    assert_eq!(
        project_page_table_entries(&words, BENCH_RESIDENT_COUNT, &resident_slots, &assignments,),
        legacy_projection(&words, BENCH_RESIDENT_COUNT, &resident_slots, &assignments,)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&words, &resident_slots, &assignments));
        black_box(measure_optimized(&words, &resident_slots, &assignments));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&words, &resident_slots, &assignments));
            optimized_samples.push(measure_optimized(&words, &resident_slots, &assignments));
        } else {
            optimized_samples.push(measure_optimized(&words, &resident_slots, &assignments));
            legacy_samples.push(measure_legacy(&words, &resident_slots, &assignments));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=page_table_decode_slot_membership \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
resident_count={BENCH_RESIDENT_COUNT} assignment_count={BENCH_ASSIGNMENT_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_slot_index=btree optimized_slot_index=preallocated_hash optimized_output=preallocated \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "hashed page-table slot membership must reduce P95 by at least 25%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(words: &[u32], resident_slots: &[u32], assignments: &[(u32, u32)]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_projection(
            black_box(words),
            BENCH_RESIDENT_COUNT,
            black_box(resident_slots),
            black_box(assignments),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(words: &[u32], resident_slots: &[u32], assignments: &[(u32, u32)]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(project_page_table_entries(
            black_box(words),
            BENCH_RESIDENT_COUNT,
            black_box(resident_slots),
            black_box(assignments),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_projection(
    words: &[u32],
    resident_entry_count: usize,
    resident_slots: &[u32],
    assignments: &[(u32, u32)],
) -> Vec<(u32, u32)> {
    let resident_slots = resident_slots.iter().copied().collect::<BTreeSet<_>>();
    let appended_entry_count = assignments
        .iter()
        .filter(|(_, slot)| !resident_slots.contains(slot))
        .count();
    words
        .chunks_exact(2)
        .take(resident_entry_count + appended_entry_count)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect()
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
