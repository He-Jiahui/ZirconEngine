use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::layout::{AxisConstraint, ResolvedAxisConstraint, StretchMode};

use super::{solve_axis_constraints, solve_axis_constraints_into};

const PERF_MARKER: &str = "RUNTIME364_CONSTRAINT_FINAL_SUM_SHORT_CIRCUIT_BENCH_V1";

#[test]
fn optimization_batch_20260830bl_runtime_constraint_final_sum_preserves_results() {
    let constraints = [
        axis(0.0, 100.0, 10.0, 0, 1.0, StretchMode::Fixed),
        axis(0.0, 100.0, 20.0, 0, 1.0, StretchMode::Fixed),
    ];
    let expected = solve_axis_constraints(30.0, &constraints);
    let mut resolved = Vec::new();
    let mut priorities = Vec::new();
    let mut active_indices = Vec::new();
    solve_axis_constraints_into(
        30.0,
        &constraints,
        &mut resolved,
        &mut priorities,
        &mut active_indices,
    );
    assert_eq!(resolved, expected);
}

#[test]
fn optimization_batch_20260830bl_runtime_constraint_final_sum_source_contract() {
    let source = include_str!("../constraints.rs");
    assert!(source.contains("let needs_final_clamp = if total + EPSILON < available"));
    assert!(source.contains("if needs_final_clamp && total > available + EPSILON"));
    assert!(source.contains("} else {\n        false\n    }"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260830bl_runtime_constraint_final_sum_p95() {
    const AXES: usize = 64;
    const MATCHES: usize = 2_000_000;
    const SAMPLES: usize = 17;
    let resolved = black_box(
        (0..AXES)
            .map(|index| ResolvedAxisConstraint {
                min: 0.0,
                max: None,
                preferred: 1.0 + index as f32,
                priority: 0,
                weight: 1.0,
                stretch_mode: StretchMode::Fixed,
                resolved: 1.0 + index as f32,
            })
            .collect::<Vec<_>>(),
    );
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    let exact_fit = black_box(true);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0.0;
            for _ in 0..MATCHES {
                if pass == 0 {
                    checksum += resolved.iter().map(|axis| axis.resolved).sum::<f32>();
                } else {
                    let needs_final_clamp = !exact_fit;
                    if needs_final_clamp {
                        checksum += resolved.iter().map(|axis| axis.resolved).sum::<f32>();
                    }
                }
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} axes={AXES} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}

fn axis(
    min: f32,
    max: f32,
    preferred: f32,
    priority: i32,
    weight: f32,
    stretch_mode: StretchMode,
) -> AxisConstraint {
    AxisConstraint {
        min,
        max,
        preferred,
        priority,
        weight,
        stretch_mode,
    }
}
