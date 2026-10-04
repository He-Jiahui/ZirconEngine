use std::time::{Duration, Instant};

use super::*;

const EDITOR54_CONSTRAINT_AGGREGATION_BENCH_V1: &str = "EDITOR54_CONSTRAINT_AGGREGATION_BENCH_V1";

fn axis(min: f32, max: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Fixed,
    }
}

// 旧多遍参考只服务等价断言与ignored性能证据，不参与生产壳布局。
fn legacy_aggregate_row_constraints(children: &[PaneConstraints]) -> PaneConstraints {
    if children.is_empty() {
        return fixed_zero_constraints();
    }
    PaneConstraints {
        width: AxisConstraint {
            min: children
                .iter()
                .map(|constraint| constraint.width.resolved().min)
                .sum(),
            max: sum_max(
                children
                    .iter()
                    .map(|constraint| constraint.width.resolved().max),
            ),
            preferred: children
                .iter()
                .map(|constraint| constraint.width.resolved().preferred)
                .sum(),
            priority: children
                .iter()
                .map(|constraint| constraint.width.priority)
                .max()
                .unwrap_or_default(),
            weight: children
                .iter()
                .map(|constraint| constraint.width.weight)
                .sum(),
            stretch_mode: StretchMode::Stretch,
        },
        height: AxisConstraint {
            min: children
                .iter()
                .map(|constraint| constraint.height.resolved().min)
                .fold(0.0_f32, f32::max),
            max: max_max(
                children
                    .iter()
                    .map(|constraint| constraint.height.resolved().max),
            ),
            preferred: children
                .iter()
                .map(|constraint| constraint.height.resolved().preferred)
                .fold(0.0_f32, f32::max),
            priority: children
                .iter()
                .map(|constraint| constraint.height.priority)
                .max()
                .unwrap_or_default(),
            weight: children
                .iter()
                .map(|constraint| constraint.height.weight)
                .sum(),
            stretch_mode: StretchMode::Stretch,
        },
    }
}

#[test]
fn optimization_wave_20260825vw_editor54_constraint_aggregation_preserves_semantics() {
    let bounded = [
        PaneConstraints {
            width: axis(-4.0, 10.0, 20.0, 2, 1.5),
            height: axis(3.0, 18.0, 9.0, 7, 2.0),
        },
        PaneConstraints {
            width: axis(5.0, 12.0, 8.0, 5, 0.5),
            height: axis(8.0, 14.0, 22.0, 1, 3.0),
        },
    ];
    let unbounded = [
        bounded[0],
        PaneConstraints {
            width: axis(2.0, -1.0, 6.0, -3, -0.5),
            height: axis(1.0, -1.0, 4.0, 9, -2.0),
        },
    ];

    assert_eq!(
        aggregate_row_constraints(&bounded),
        legacy_aggregate_row_constraints(&bounded)
    );
    assert_eq!(
        aggregate_row_constraints(&unbounded),
        legacy_aggregate_row_constraints(&unbounded)
    );
    assert_eq!(aggregate_row_constraints(&[]), fixed_zero_constraints());
}

#[test]
fn optimization_wave_20260825vw_editor54_constraint_aggregation_uses_one_child_pass() {
    let production = include_str!("../aggregation.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production section should exist")
        .split_whitespace()
        .collect::<String>();

    assert_eq!(production.matches("forchildinchildren").count(), 1);
    assert!(!production.contains("children.iter()"));
}

#[test]
#[ignore = "release-mode performance evidence"]
fn optimization_wave_20260825vw_editor54_constraint_aggregation_single_pass_evidence() {
    const CHILD_COUNT: usize = 100_000;
    const TARGET: Duration = Duration::from_millis(50);

    let children = vec![
        PaneConstraints {
            width: axis(4.0, 40.0, 12.0, 3, 1.0),
            height: axis(8.0, 80.0, 24.0, 5, 2.0),
        };
        CHILD_COUNT
    ];

    let started = Instant::now();
    let aggregate = aggregate_row_constraints(std::hint::black_box(&children));
    let elapsed = started.elapsed();

    assert_eq!(aggregate.width.min, 4.0 * CHILD_COUNT as f32);
    assert_eq!(aggregate.height.min, 8.0);
    assert!(
        elapsed <= TARGET,
        "{EDITOR54_CONSTRAINT_AGGREGATION_BENCH_V1}: expected {CHILD_COUNT} children within {TARGET:?}, got {elapsed:?}"
    );
    eprintln!(
        "{EDITOR54_CONSTRAINT_AGGREGATION_BENCH_V1} children={CHILD_COUNT} legacy_child_visits={} optimized_child_visits={CHILD_COUNT} child_visit_reduction_percent=90.00 legacy_resolved_calls={} optimized_resolved_calls={} resolved_call_reduction_percent=66.67 elapsed_us={} target_us={}",
        CHILD_COUNT * 10,
        CHILD_COUNT * 6,
        CHILD_COUNT * 2,
        elapsed.as_micros(),
        TARGET.as_micros()
    );
}
