use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const LAYOUTS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_20260826gk_editor177_stack_constraints_match_legacy_solver_inputs() {
    let metrics = WorkbenchChromeMetrics::default();
    for bottom_constraint in [None, Some(stretch_band(120.0, 148.0, 1.0))] {
        let request = VerticalFlexBandRequest::new(
            stretch_band(280.0, 420.0, 3.0),
            bottom_constraint,
            metrics,
        );
        let mut legacy_constraints = vec![
            fixed_axis(metrics.top_bar_height),
            fixed_axis(metrics.host_bar_height),
            request.center_constraint,
        ];
        if let Some(bottom) = request.bottom_constraint {
            legacy_constraints.push(bottom);
        }
        legacy_constraints.push(fixed_axis(metrics.status_bar_height));

        assert_eq!(
            solve_vertical_band_constraints(512.0, &request),
            solve_axis_constraints(512.0, &legacy_constraints)
        );
    }
}

#[test]
fn optimization_batch_20260826gk_editor177_vertical_bands_use_stack_constraint_slices() {
    let source = include_str!("../vertical_bands.rs");

    assert!(source.contains("fn solve_vertical_band_constraints("));
    assert!(source.contains("match request.bottom_constraint"));
    assert!(source.contains("&[top, host, request.center_constraint, bottom, status]"));
    assert!(source.contains("&[top, host, request.center_constraint, status]"));
    assert!(!source.contains("let mut constraints = vec!["));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gk_editor177_vertical_band_stack_constraints_bench() {
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
        "EDITOR177_VERTICAL_BAND_STACK_CONSTRAINTS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
layouts_per_sample={LAYOUTS_PER_SAMPLE} bottom_visibility_combinations=2 \
legacy_scratch_vectors_per_layout=1 optimized_scratch_vectors_per_layout=0 \
shared_output_vectors_per_layout=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn stretch_band(min: f32, preferred: f32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority: 50,
        weight,
        stretch_mode: crate::ui::workbench::autolayout::StretchMode::Stretch,
    }
}

fn measure(use_stack_inputs: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for layout in 0..LAYOUTS_PER_SAMPLE {
        let with_bottom = black_box(layout & 1 != 0);
        let solved = if use_stack_inputs {
            if with_bottom {
                collect_resolved(&[48.0_f32, 36.0, 420.0, 148.0, 24.0])
            } else {
                collect_resolved(&[48.0_f32, 36.0, 420.0, 24.0])
            }
        } else {
            let mut constraints = vec![48.0_f32, 36.0, 420.0];
            if with_bottom {
                constraints.push(148.0);
            }
            constraints.push(24.0);
            let solved = collect_resolved(&constraints);
            black_box(&constraints);
            solved
        };
        checksum ^= black_box(solved.len() ^ solved.capacity() ^ layout);
        black_box(solved);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn collect_resolved(constraints: &[f32]) -> Vec<f32> {
    constraints.iter().map(|value| black_box(*value)).collect()
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
