use std::hint::black_box;
use std::time::Instant;

use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::{preferred_drawer_slot_from_candidates, DrawerPreference};

const SAMPLE_PAIRS: usize = 21;
const SCANS_PER_SAMPLE: usize = 8_192;
const CANDIDATE_COUNT: usize = 256;

#[test]
fn optimization_batch_20260826ec_editor118_drawer_scan_preserves_global_priority_and_order() {
    let candidates = [
        (ActivityDrawerSlot::LeftTop, DrawerPreference::Visible),
        (ActivityDrawerSlot::LeftBottom, DrawerPreference::Populated),
        (ActivityDrawerSlot::RightTop, DrawerPreference::ActiveTab),
        (
            ActivityDrawerSlot::RightBottom,
            DrawerPreference::ActiveView,
        ),
        (ActivityDrawerSlot::Bottom, DrawerPreference::ActiveView),
    ];

    assert_eq!(
        preferred_drawer_slot_from_candidates(candidates, ActivityDrawerSlot::Bottom,),
        ActivityDrawerSlot::RightBottom
    );
    assert_eq!(
        preferred_drawer_slot_from_candidates([], ActivityDrawerSlot::Bottom,),
        ActivityDrawerSlot::Bottom
    );
}

#[test]
fn optimization_batch_20260826ec_editor118_drawer_scan_uses_one_candidate_loop() {
    let source = include_str!("../host_resolution.rs");
    let helper_start = source
        .find("fn preferred_drawer_slot_from_candidates")
        .unwrap();
    let helper_end = source[helper_start..]
        .find("fn preferred_workspace_path")
        .map(|offset| helper_start + offset)
        .unwrap();
    let helper_source = &source[helper_start..helper_end];
    assert_eq!(
        helper_source
            .matches("for (slot, preference) in candidates")
            .count(),
        1
    );
    assert!(!helper_source.contains(".or_else(||"));
    assert!(!helper_source.contains("slots.iter()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ec_editor118_preferred_drawer_single_scan_bench() {
    let candidates = fixture_candidates();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&candidates));
            optimized_samples.push(measure_optimized(&candidates));
        } else {
            optimized_samples.push(measure_optimized(&candidates));
            legacy_samples.push(measure_legacy(&candidates));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR118_PREFERRED_DRAWER_SINGLE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
scans_per_sample={SCANS_PER_SAMPLE} candidates_per_scan={CANDIDATE_COUNT} legacy_passes=4 \
optimized_passes=1 legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single drawer candidate scan P95 {optimized_p95_ns}ns must be at most 70% of four-pass P95 {legacy_p95_ns}ns"
    );
}

fn fixture_candidates() -> Vec<(ActivityDrawerSlot, DrawerPreference)> {
    (0..CANDIDATE_COUNT)
        .map(|index| {
            let preference = match index {
                64 => DrawerPreference::Visible,
                128 => DrawerPreference::Populated,
                192 => DrawerPreference::ActiveTab,
                255 => DrawerPreference::ActiveView,
                _ => DrawerPreference::Unavailable,
            };
            (ActivityDrawerSlot::LeftTop, preference)
        })
        .collect()
}

fn legacy_preferred_drawer_slot(
    candidates: &[(ActivityDrawerSlot, DrawerPreference)],
) -> ActivityDrawerSlot {
    [
        DrawerPreference::ActiveView,
        DrawerPreference::ActiveTab,
        DrawerPreference::Populated,
        DrawerPreference::Visible,
    ]
    .into_iter()
    .find_map(|expected| {
        candidates
            .iter()
            .find(|(_, preference)| *preference == expected)
            .map(|(slot, _)| *slot)
    })
    .unwrap_or(ActivityDrawerSlot::Bottom)
}

fn measure_legacy(candidates: &[(ActivityDrawerSlot, DrawerPreference)]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        checksum ^= black_box(legacy_preferred_drawer_slot(black_box(candidates))) as usize;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(candidates: &[(ActivityDrawerSlot, DrawerPreference)]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SCANS_PER_SAMPLE {
        checksum ^= black_box(preferred_drawer_slot_from_candidates(
            black_box(candidates.iter().copied()),
            ActivityDrawerSlot::Bottom,
        )) as usize;
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
