use std::hint::black_box;
use std::time::Instant;

use super::{drawer_dock_tab_capacity_from_rows, drawer_tab_capacity_from_rows};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 128;
const DOCKS_PER_BUILD: usize = 3;
const TABS_PER_DOCK: usize = 1_365;
const TABS_PER_BUILD: usize = DOCKS_PER_BUILD * TABS_PER_DOCK;

#[test]
fn optimization_batch_20260826gn_editor180_capacity_sums_all_fixed_docks() {
    assert_eq!(drawer_dock_tab_capacity_from_rows(7, 11, 13), 31);
    assert_eq!(drawer_dock_tab_capacity_from_rows(0, 0, 0), 0);
    assert_eq!(
        drawer_dock_tab_capacity_from_rows(usize::MAX, 1, 1),
        usize::MAX
    );
}

#[test]
fn optimization_batch_20260826gn_editor180_drawer_tabs_preallocate_dock_rows() {
    let source = include_str!("../drawer_tabs.rs");

    assert!(source.contains("Vec::with_capacity(drawer_tab_capacity(scene))"));
    assert!(source.contains("scene.left_dock.tab_frames.row_count()"));
    assert!(source.contains("scene.right_dock.tab_frames.row_count()"));
    assert!(source.contains("scene.bottom_dock.tab_frames.row_count()"));
}

#[test]
fn optimization_batch_dn_drawer_tabs_capacity_includes_floating_window_rows() {
    assert_eq!(drawer_tab_capacity_from_rows(7, 11, 13, [2, 3]), 36);
    assert_eq!(drawer_tab_capacity_from_rows(0, 0, 0, []), 0);
    assert_eq!(
        drawer_tab_capacity_from_rows(usize::MAX, 1, 1, [4]),
        usize::MAX
    );
}

#[test]
fn optimization_batch_dn_drawer_tabs_preallocate_all_tab_sources() {
    let source = include_str!("../drawer_tabs.rs");

    assert!(source.contains("Vec::with_capacity(drawer_tab_capacity(scene))"));
    assert!(source.contains("floating_windows.iter()"));
    assert!(source.contains("tab_frames.row_count()"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dn_drawer_tabs_all_sources_capacity_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const BUILDS_PER_SAMPLE: usize = 2_048;
    const FIXED_DOCK_TABS: usize = 384;
    const FLOATING_WINDOW_TABS: usize = 128;
    const TOTAL_TABS: usize = FIXED_DOCK_TABS + FLOATING_WINDOW_TABS;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_drawer_capacity(BUILDS_PER_SAMPLE, false));
            optimized_samples.push(measure_drawer_capacity(BUILDS_PER_SAMPLE, true));
        } else {
            optimized_samples.push(measure_drawer_capacity(BUILDS_PER_SAMPLE, true));
            legacy_samples.push(measure_drawer_capacity(BUILDS_PER_SAMPLE, false));
        }
    }

    let legacy_p95 = percentile(&mut legacy_samples);
    let optimized_p95 = percentile(&mut optimized_samples);
    println!(
        "EDITOR350_DRAWER_TABS_ALL_SOURCES_CAPACITY_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "all-source drawer tab capacity p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn measure_drawer_capacity(build_count: usize, preallocate_floating: bool) -> u128 {
    let started_at = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..build_count {
        let initial_capacity = if preallocate_floating {
            TOTAL_TABS
        } else {
            FIXED_DOCK_TABS
        };
        let mut frames = Vec::with_capacity(initial_capacity);
        for index in 0..TOTAL_TABS {
            frames.push(std::hint::black_box(index));
        }
        checksum ^= std::hint::black_box(frames.len() ^ frames.capacity());
    }
    std::hint::black_box(checksum);
    started_at.elapsed().as_nanos().max(1)
}

fn percentile(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gn_editor180_drawer_dock_tab_capacity_bench() {
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
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR180_DRAWER_DOCK_TAB_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} docks_per_build={DOCKS_PER_BUILD} \
tabs_per_dock={TABS_PER_DOCK} tabs_per_build={TABS_PER_BUILD} profile_frame_usize_fields=16 \
legacy_initial_capacity=0 optimized_initial_capacity={TABS_PER_BUILD} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(preallocate: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for build in 0..BUILDS_PER_SAMPLE {
        let mut frames = if preallocate {
            Vec::with_capacity(TABS_PER_BUILD)
        } else {
            Vec::new()
        };
        for dock in 0..DOCKS_PER_BUILD {
            let surface_frames = (0..TABS_PER_DOCK)
                .map(|tab| {
                    let value = black_box(build * TABS_PER_BUILD + dock * TABS_PER_DOCK + tab);
                    [value; 16]
                })
                .collect::<Vec<_>>();
            frames.extend(surface_frames);
        }
        checksum ^= black_box(frames.len() ^ frames.capacity() ^ build);
        black_box(frames);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
