use std::hint::black_box;
use std::time::Instant;

use crate::virtual_geometry::types::{VirtualGeometryPrepareFrame, VirtualGeometryPreparePage};

use super::{evictable_slots_and_reclaimable_bytes, resident_entries_and_slots};

const BENCH_PAGE_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn frame_projections_preserve_order_and_saturate_reclaimable_bytes() {
    let frame = VirtualGeometryPrepareFrame {
        resident_pages: vec![page(10, 1, 100), page(20, 2, 200)],
        evictable_pages: vec![page(30, 3, u64::MAX), page(40, 4, 1)],
        ..VirtualGeometryPrepareFrame::default()
    };

    assert_eq!(
        resident_entries_and_slots(&frame),
        (vec![[10, 1], [20, 2]], vec![1, 2])
    );
    assert_eq!(
        evictable_slots_and_reclaimable_bytes(&frame),
        (vec![3, 4], u32::MAX)
    );
}

#[test]
#[ignore = "release-only resident frame projection benchmark"]
fn resident_frame_projection_release_benchmark_evidence() {
    let frame = benchmark_frame();
    assert_eq!(
        resident_entries_and_slots(&frame),
        legacy_resident_projection(&frame)
    );
    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_resident_legacy(&frame),
        || measure_resident_optimized(&frame),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=resident_frame_single_pass_projection \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
resident_page_count={BENCH_PAGE_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_source_scans=2 optimized_source_scans=1 optimized_vectors_preallocated=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "single-pass resident frame projection must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "release-only evictable frame projection benchmark"]
fn evictable_frame_projection_release_benchmark_evidence() {
    let frame = benchmark_frame();
    assert_eq!(
        evictable_slots_and_reclaimable_bytes(&frame),
        legacy_evictable_projection(&frame)
    );
    let (legacy_samples, optimized_samples) = paired_samples(
        || measure_evictable_legacy(&frame),
        || measure_evictable_optimized(&frame),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins17 task=evictable_frame_single_pass_projection \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
evictable_page_count={BENCH_PAGE_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_source_scans=2 optimized_source_scans=1 optimized_vector=preallocated \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns.saturating_mul(17),
        "single-pass evictable frame projection must reduce P95 by at least 15%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
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

fn measure_resident_legacy(frame: &VirtualGeometryPrepareFrame) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_resident_projection(black_box(frame)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_resident_optimized(frame: &VirtualGeometryPrepareFrame) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(resident_entries_and_slots(black_box(frame)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_evictable_legacy(frame: &VirtualGeometryPrepareFrame) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_evictable_projection(black_box(frame)));
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_evictable_optimized(frame: &VirtualGeometryPrepareFrame) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(evictable_slots_and_reclaimable_bytes(black_box(frame)));
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_resident_projection(frame: &VirtualGeometryPrepareFrame) -> (Vec<[u32; 2]>, Vec<u32>) {
    let entries = frame
        .resident_pages
        .iter()
        .map(|page| [page.page_id, page.slot])
        .collect();
    let slots = frame.resident_pages.iter().map(|page| page.slot).collect();
    (entries, slots)
}

fn legacy_evictable_projection(frame: &VirtualGeometryPrepareFrame) -> (Vec<u32>, u32) {
    let slots = frame.evictable_pages.iter().map(|page| page.slot).collect();
    let reclaimable_bytes = frame
        .evictable_pages
        .iter()
        .fold(0_u64, |bytes, page| bytes.saturating_add(page.size_bytes))
        .min(u64::from(u32::MAX)) as u32;
    (slots, reclaimable_bytes)
}

fn benchmark_frame() -> VirtualGeometryPrepareFrame {
    VirtualGeometryPrepareFrame {
        resident_pages: (0..BENCH_PAGE_COUNT as u32)
            .map(|page_id| page(page_id, page_id + 10_000, 4_096))
            .collect(),
        evictable_pages: (0..BENCH_PAGE_COUNT as u32)
            .map(|page_id| page(page_id, page_id + 20_000, u64::from(page_id) + 1))
            .collect(),
        ..VirtualGeometryPrepareFrame::default()
    }
}

fn page(page_id: u32, slot: u32, size_bytes: u64) -> VirtualGeometryPreparePage {
    VirtualGeometryPreparePage {
        page_id,
        slot,
        size_bytes,
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
