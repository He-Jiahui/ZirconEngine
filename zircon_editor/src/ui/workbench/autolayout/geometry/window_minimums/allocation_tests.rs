use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::ui::workbench::autolayout::{default_region_constraints, ShellRegionId};

const SAMPLE_PAIRS: usize = 21;
const WINDOWS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_20260826gf_editor173_stack_constraints_match_legacy_visibility_results() {
    let metrics = WorkbenchChromeMetrics::default();
    for (left_visible, right_visible) in
        [(false, false), (true, false), (false, true), (true, true)]
    {
        let left = region(ShellRegionId::Left, left_visible);
        let document = region(ShellRegionId::Document, true);
        let right = region(ShellRegionId::Right, right_visible);
        let shell_width = 4_096.0;

        assert_eq!(
            compute_window_min_width(left, document, right, &metrics, shell_width),
            legacy_compute_window_min_width(left, document, right, &metrics, shell_width)
        );
    }
}

#[test]
fn optimization_batch_20260826gf_editor173_window_minimum_uses_stack_constraint_slices() {
    let source = include_str!("../window_minimums.rs");

    assert!(source.contains("match (left.visible, right.visible)"));
    assert!(source.contains(
        "aggregate_row_constraints(&[left.constraints, document.constraints, right.constraints])"
    ));
    assert!(
        source.contains("let visible_side_count = left.visible as usize + right.visible as usize;")
    );
    assert!(!source.contains("let mut widths = Vec::new();"));
    assert!(!source.contains("widths.push("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gf_editor173_window_minimum_width_stack_constraints_bench() {
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
        "EDITOR173_WINDOW_MINIMUM_WIDTH_STACK_CONSTRAINTS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
windows_per_sample={WINDOWS_PER_SAMPLE} visibility_combinations=4 \
legacy_temporary_vectors_per_window=1 optimized_temporary_vectors_per_window=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn region(region: ShellRegionId, visible: bool) -> RegionState {
    RegionState {
        visible,
        expanded: visible,
        constraints: default_region_constraints(region),
    }
}

fn legacy_compute_window_min_width(
    left: RegionState,
    document: RegionState,
    right: RegionState,
    metrics: &WorkbenchChromeMetrics,
    shell_logical_width: f32,
) -> f32 {
    let mut widths = Vec::new();
    if left.visible {
        widths.push(left.constraints);
    }
    widths.push(document.constraints);
    if right.visible {
        widths.push(right.constraints);
    }
    let separators = widths.len().saturating_sub(1) as f32 * metrics.separator_thickness;
    let content_min_width = aggregate_row_constraints(&widths).width.resolved().min + separators;
    content_min_width.min(window_min_width_limit_for_logical_width(
        shell_logical_width,
    ))
}

fn measure(use_stack_slices: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0.0_f32;
    for window in 0..WINDOWS_PER_SAMPLE {
        let left_visible = black_box(window & 1 != 0);
        let right_visible = black_box(window & 2 != 0);
        let left = black_box(240.0_f32);
        let document = black_box(520.0_f32);
        let right = black_box(260.0_f32);
        let width = if use_stack_slices {
            match (left_visible, right_visible) {
                (true, true) => [left, document, right].iter().sum::<f32>(),
                (true, false) => [left, document].iter().sum::<f32>(),
                (false, true) => [document, right].iter().sum::<f32>(),
                (false, false) => document,
            }
        } else {
            let mut widths = Vec::new();
            if left_visible {
                widths.push(left);
            }
            widths.push(document);
            if right_visible {
                widths.push(right);
            }
            let width = widths.iter().sum();
            black_box(&widths);
            width
        };
        checksum += black_box(width + left_visible as u8 as f32 + right_visible as u8 as f32);
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
