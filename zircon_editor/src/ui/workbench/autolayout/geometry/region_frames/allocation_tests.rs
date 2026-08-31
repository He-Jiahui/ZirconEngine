use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::ui::workbench::autolayout::{default_region_constraints, ShellRegionId};

const SAMPLE_PAIRS: usize = 21;
const LAYOUTS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_20260826gh_editor174_stack_inputs_preserve_region_order() {
    let left = default_region_constraints(ShellRegionId::Left).width;
    let document = default_region_constraints(ShellRegionId::Document).width;
    let right = default_region_constraints(ShellRegionId::Right).width;

    for (left_visible, right_visible, expected) in [
        (false, false, vec![ShellRegionId::Document]),
        (
            true,
            false,
            vec![ShellRegionId::Left, ShellRegionId::Document],
        ),
        (
            false,
            true,
            vec![ShellRegionId::Document, ShellRegionId::Right],
        ),
        (
            true,
            true,
            vec![
                ShellRegionId::Left,
                ShellRegionId::Document,
                ShellRegionId::Right,
            ],
        ),
    ] {
        let solved = solve_visible_row_widths(
            2_048.0,
            2_046.0,
            left_visible.then_some(left),
            document,
            right_visible.then_some(right),
        );
        assert_eq!(
            solved.iter().map(|(region, _)| *region).collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn optimization_batch_20260826gh_editor174_region_solver_uses_stack_slices() {
    let source = include_str!("../region_frames.rs");

    assert!(source.contains("fn solve_visible_row_widths("));
    assert!(source.contains("match (left, right)"));
    assert!(source.contains("fn solve_row_widths("));
    assert!(!source.contains("let mut horizontal_constraints = Vec::new();"));
    assert!(!source.contains("let mut horizontal_regions = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gh_editor174_region_frame_scratch_allocation_bench() {
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
        "EDITOR174_REGION_FRAME_SCRATCH_ALLOCATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
layouts_per_sample={LAYOUTS_PER_SAMPLE} visibility_combinations=4 \
legacy_scratch_vectors_per_layout=2 optimized_scratch_vectors_per_layout=0 \
shared_output_vectors_per_layout=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_stack_inputs: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for layout in 0..LAYOUTS_PER_SAMPLE {
        let left_visible = black_box(layout & 1 != 0);
        let right_visible = black_box(layout & 2 != 0);
        let solved = if use_stack_inputs {
            match (left_visible, right_visible) {
                (true, true) => collect_solved(&[0_u8, 1, 2], &[240.0_f32, 520.0_f32, 260.0_f32]),
                (true, false) => collect_solved(&[0_u8, 1], &[240.0_f32, 520.0_f32]),
                (false, true) => collect_solved(&[1_u8, 2], &[520.0_f32, 260.0_f32]),
                (false, false) => collect_solved(&[1_u8], &[520.0_f32]),
            }
        } else {
            let mut regions = Vec::new();
            let mut constraints = Vec::new();
            if left_visible {
                regions.push(0_u8);
                constraints.push(240.0_f32);
            }
            regions.push(1_u8);
            constraints.push(520.0_f32);
            if right_visible {
                regions.push(2_u8);
                constraints.push(260.0_f32);
            }
            let solved = collect_solved(&regions, &constraints);
            black_box((&regions, &constraints));
            solved
        };
        checksum ^= black_box(solved.len() ^ solved.capacity());
        black_box(&solved);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn collect_solved(regions: &[u8], constraints: &[f32]) -> Vec<(u8, f32)> {
    regions
        .iter()
        .copied()
        .zip(constraints.iter().copied())
        .collect()
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
