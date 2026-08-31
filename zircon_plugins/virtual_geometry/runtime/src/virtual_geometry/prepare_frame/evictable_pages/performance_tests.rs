use std::hint::black_box;
use std::time::Instant;

use crate::virtual_geometry::{VirtualGeometryPreparePage, VirtualGeometryRuntimeState};

use super::evictable_pages;

const BENCH_PAGE_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn preallocated_evictable_projection_preserves_order_and_skips_missing_residents() {
    let mut state = VirtualGeometryRuntimeState::default();
    state.insert_page_size(10, 100);
    state.insert_page_size(30, 300);
    state.insert_resident_page_slot(10, 1);
    state.insert_resident_page_slot(30, 3);
    state.replace_evictable_pages(vec![30, 20, 10]);

    assert_eq!(
        evictable_pages(&state),
        vec![
            VirtualGeometryPreparePage {
                page_id: 30,
                slot: 3,
                size_bytes: 300,
            },
            VirtualGeometryPreparePage {
                page_id: 10,
                slot: 1,
                size_bytes: 100,
            },
        ]
    );
}

#[test]
#[ignore = "release-only evictable page projection benchmark"]
fn evictable_page_preallocation_release_benchmark_evidence() {
    let state = benchmark_state();
    assert_eq!(evictable_pages(&state), legacy_evictable_pages(&state));

    let (legacy_samples, optimized_samples) =
        paired_samples(|| measure_legacy(&state), || measure_optimized(&state));
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=evictable_page_projection_preallocation \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
evictable_page_count={BENCH_PAGE_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_capacity_lower_bound=0 optimized_capacity_lower_bound={BENCH_PAGE_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(19),
        "preallocated evictable page projection must reduce P95 by at least 5%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_evictable_pages(state: &VirtualGeometryRuntimeState) -> Vec<VirtualGeometryPreparePage> {
    state
        .evictable_page_ids()
        .iter()
        .filter_map(|page_id| {
            state
                .resident_slot(*page_id)
                .map(|slot| VirtualGeometryPreparePage {
                    page_id: *page_id,
                    slot,
                    size_bytes: state.page_size_bytes(*page_id),
                })
        })
        .collect()
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

fn measure_legacy(state: &VirtualGeometryRuntimeState) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_evictable_pages(black_box(state)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(state: &VirtualGeometryRuntimeState) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(evictable_pages(black_box(state)));
    }
    started.elapsed().as_nanos().max(1)
}

fn benchmark_state() -> VirtualGeometryRuntimeState {
    let mut state = VirtualGeometryRuntimeState::default();
    for page_id in 0..BENCH_PAGE_COUNT as u32 {
        state.insert_page_size(page_id, u64::from(page_id) + 4_096);
        state.insert_resident_page_slot(page_id, page_id + 10_000);
    }
    state.replace_evictable_pages((0..BENCH_PAGE_COUNT as u32).rev().collect());
    state
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
