use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::normalized_page_table_entries;

const BENCH_ENTRY_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 32;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn page_table_normalization_preserves_reverse_last_wins_conflict_semantics() {
    let entries = [(1, 10), (2, 20), (1, 30), (3, 20), (4, 40)];

    let optimized = normalized_page_table_entries(&entries);
    let legacy = legacy_normalize(&entries);

    assert_eq!(optimized, legacy);
    assert_eq!(optimized, vec![(3, 20), (1, 30), (4, 40)]);
}

#[test]
#[ignore = "release-only page-table normalization benchmark"]
fn page_table_normalization_release_benchmark_evidence() {
    let entries = (0..BENCH_ENTRY_COUNT)
        .map(|index| {
            let shuffled = (index * 1_549) % BENCH_ENTRY_COUNT;
            (shuffled as u32, (BENCH_ENTRY_COUNT - shuffled) as u32)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_normalize(&entries),
        normalized_page_table_entries(&entries)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&entries));
        black_box(measure_optimized(&entries));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&entries));
            optimized_samples.push(measure_optimized(&entries));
        } else {
            optimized_samples.push(measure_optimized(&entries));
            legacy_samples.push(measure_legacy(&entries));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=page_table_normalization_index \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
entry_count={BENCH_ENTRY_COUNT} unique_page_count={BENCH_ENTRY_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_index=btree_unreserved optimized_index=hash_preallocated \
legacy_sort=stable optimized_sort=unstable_in_place \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "hashed page-table normalization must reduce P95 by at least 30%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(entries: &[(u32, u32)]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_normalize(black_box(entries)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(entries: &[(u32, u32)]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(normalized_page_table_entries(black_box(entries)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_normalize(entries: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut seen_page_ids = BTreeSet::new();
    let mut seen_slots = BTreeSet::new();
    let mut normalized_entries = Vec::new();
    for &(page_id, slot) in entries.iter().rev() {
        if seen_page_ids.contains(&page_id) || seen_slots.contains(&slot) {
            continue;
        }
        seen_page_ids.insert(page_id);
        seen_slots.insert(slot);
        normalized_entries.push((page_id, slot));
    }
    normalized_entries.sort_by_key(|(_page_id, slot)| *slot);
    normalized_entries
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
