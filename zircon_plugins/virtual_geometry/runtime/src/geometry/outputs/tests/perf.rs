use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::render::RenderVirtualGeometryPageReplacementRecord;

use super::{flat_page_table_entries, page_replacements};

const BENCH_PAGE_COUNT: usize = 512;
const CHECKS_PER_SAMPLE: usize = 8;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn readback_page_projection_preserves_assignment_precedence_and_first_wins() {
    let assignments = [(7, 70), (7, 71)];
    let page_table_entries = [(7, 72), (8, 80)];
    let replacements = [(7, 1), (8, 2), (9, 3)];

    assert_eq!(
        page_replacements(&replacements, &page_table_entries, &assignments),
        vec![
            replacement(1, 7, 70),
            replacement(2, 8, 80),
            replacement(3, 9, 0),
        ]
    );
    assert_eq!(
        flat_page_table_entries(&[(7, 70), (8, 80)]),
        vec![7, 70, 8, 80]
    );
}

#[test]
#[ignore = "release-only readback page projection benchmark"]
fn readback_page_projection_release_benchmark_evidence() {
    let assignments = (0..BENCH_PAGE_COUNT / 2)
        .map(|page_id| (page_id as u32, page_id as u32 + 1_000))
        .collect::<Vec<_>>();
    let page_table_entries = (0..BENCH_PAGE_COUNT)
        .map(|page_id| (page_id as u32, page_id as u32 + 2_000))
        .collect::<Vec<_>>();
    let replacements = (0..BENCH_PAGE_COUNT)
        .map(|page_id| (page_id as u32, page_id as u32 + 10_000))
        .collect::<Vec<_>>();
    assert_eq!(
        page_replacements(&replacements, &page_table_entries, &assignments),
        legacy_page_replacements(&replacements, &page_table_entries, &assignments)
    );

    for _ in 0..4 {
        black_box(measure_legacy(
            &replacements,
            &page_table_entries,
            &assignments,
        ));
        black_box(measure_optimized(
            &replacements,
            &page_table_entries,
            &assignments,
        ));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(
                &replacements,
                &page_table_entries,
                &assignments,
            ));
            optimized_samples.push(measure_optimized(
                &replacements,
                &page_table_entries,
                &assignments,
            ));
        } else {
            optimized_samples.push(measure_optimized(
                &replacements,
                &page_table_entries,
                &assignments,
            ));
            legacy_samples.push(measure_legacy(
                &replacements,
                &page_table_entries,
                &assignments,
            ));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=readback_page_slot_index \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
page_count={BENCH_PAGE_COUNT} replacement_count={BENCH_PAGE_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_lookup=linear_per_replacement optimized_lookup=preallocated_hash_first_wins \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "indexed readback page projection must reduce P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(
    replacements: &[(u32, u32)],
    page_table_entries: &[(u32, u32)],
    assignments: &[(u32, u32)],
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_page_replacements(
            black_box(replacements),
            black_box(page_table_entries),
            black_box(assignments),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    replacements: &[(u32, u32)],
    page_table_entries: &[(u32, u32)],
    assignments: &[(u32, u32)],
) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(page_replacements(
            black_box(replacements),
            black_box(page_table_entries),
            black_box(assignments),
        ));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_page_replacements(
    replacements: &[(u32, u32)],
    page_table_entries: &[(u32, u32)],
    assignments: &[(u32, u32)],
) -> Vec<RenderVirtualGeometryPageReplacementRecord> {
    replacements
        .iter()
        .map(|&(new_page_id, old_page_id)| {
            let physical_slot = assignments
                .iter()
                .chain(page_table_entries.iter())
                .find_map(|&(candidate_page_id, slot)| {
                    (candidate_page_id == new_page_id).then_some(slot)
                })
                .unwrap_or_default();
            replacement(old_page_id, new_page_id, physical_slot)
        })
        .collect()
}

fn replacement(
    old_page_id: u32,
    new_page_id: u32,
    physical_slot: u32,
) -> RenderVirtualGeometryPageReplacementRecord {
    RenderVirtualGeometryPageReplacementRecord {
        old_page_id: u64::from(old_page_id),
        new_page_id: u64::from(new_page_id),
        physical_slot,
    }
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
