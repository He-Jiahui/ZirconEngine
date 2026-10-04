use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::VirtualGeometryRuntimeState;

const BENCH_PROBE_COUNT: usize = 16_384;
const BENCH_RESIDENT_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 16;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn resident_page_id_index_contains_exactly_the_resident_pages() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.insert_resident_page_slot(7, 70);
    state.insert_resident_page_slot(9, 90);

    let mut page_ids = state
        .resident_page_id_index()
        .into_iter()
        .collect::<Vec<_>>();
    page_ids.sort_unstable();
    assert_eq!(page_ids, vec![7, 9]);
}

#[test]
#[ignore = "release-only resident page membership index benchmark"]
fn resident_page_id_index_release_benchmark_evidence() {
    let mut state = VirtualGeometryRuntimeState::default();
    for page_id in 0..BENCH_RESIDENT_COUNT as u32 {
        state.insert_resident_page_slot(page_id * 2, page_id);
    }
    let probes = (0..BENCH_PROBE_COUNT as u32).collect::<Vec<_>>();
    assert_eq!(
        legacy_match_count(&state, &probes),
        optimized_match_count(&state, &probes)
    );

    for _ in 0..4 {
        black_box(measure_legacy(&state, &probes));
        black_box(measure_optimized(&state, &probes));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&state, &probes));
            optimized_samples.push(measure_optimized(&state, &probes));
        } else {
            optimized_samples.push(measure_optimized(&state, &probes));
            legacy_samples.push(measure_legacy(&state, &probes));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=resident_page_membership_index \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
resident_count={BENCH_RESIDENT_COUNT} probe_count={BENCH_PROBE_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_index=btree_copy optimized_index=preallocated_hash_snapshot consumers=hot_pages_and_evictable_pages \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(5) <= legacy_p95_ns.saturating_mul(3),
        "resident page hash snapshot must reduce P95 by at least 40%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_legacy(state: &VirtualGeometryRuntimeState, probes: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_match_count(black_box(state), black_box(probes)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(state: &VirtualGeometryRuntimeState, probes: &[u32]) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(optimized_match_count(black_box(state), black_box(probes)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_match_count(state: &VirtualGeometryRuntimeState, probes: &[u32]) -> usize {
    let resident_page_ids = state.resident_page_ids().collect::<BTreeSet<_>>();
    probes
        .iter()
        .filter(|page_id| resident_page_ids.contains(page_id))
        .count()
}

fn optimized_match_count(state: &VirtualGeometryRuntimeState, probes: &[u32]) -> usize {
    let resident_page_ids = state.resident_page_id_index();
    probes
        .iter()
        .filter(|page_id| resident_page_ids.contains(page_id))
        .count()
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
