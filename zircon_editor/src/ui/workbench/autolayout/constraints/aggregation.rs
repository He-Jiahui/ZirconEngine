use super::super::{AxisConstraint, PaneConstraints, StretchMode};
use super::axis_factory::fixed_zero_constraints;

pub(crate) fn aggregate_row_constraints(children: &[PaneConstraints]) -> PaneConstraints {
    if children.is_empty() {
        return fixed_zero_constraints();
    }

    let mut width_min = 0.0;
    let mut width_max = Some(0.0);
    let mut width_preferred = 0.0;
    let mut width_priority = i32::MIN;
    let mut width_weight = 0.0;
    let mut height_min = 0.0_f32;
    let mut height_max = Some(0.0_f32);
    let mut height_preferred = 0.0_f32;
    let mut height_priority = i32::MIN;
    let mut height_weight = 0.0;

    for child in children {
        let width = child.width.resolved();
        width_min += width.min;
        width_max = width_max.zip(width.max).map(|(total, max)| total + max);
        width_preferred += width.preferred;
        width_priority = width_priority.max(child.width.priority);
        width_weight += child.width.weight;

        let height = child.height.resolved();
        height_min = height_min.max(height.min);
        height_max = height_max
            .zip(height.max)
            .map(|(current, max)| current.max(max));
        height_preferred = height_preferred.max(height.preferred);
        height_priority = height_priority.max(child.height.priority);
        height_weight += child.height.weight;
    }

    PaneConstraints {
        width: AxisConstraint {
            min: width_min,
            max: width_max.unwrap_or(-1.0),
            preferred: width_preferred,
            priority: width_priority,
            weight: width_weight,
            stretch_mode: StretchMode::Stretch,
        },
        height: AxisConstraint {
            min: height_min,
            max: height_max.unwrap_or(-1.0),
            preferred: height_preferred,
            priority: height_priority,
            weight: height_weight,
            stretch_mode: StretchMode::Stretch,
        },
    }
}

#[cfg(test)]
fn sum_max(values: impl Iterator<Item = Option<f32>>) -> f32 {
    let mut total = 0.0;
    for value in values {
        let Some(value) = value else {
            return -1.0;
        };
        total += value;
    }
    total
}

#[cfg(test)]
fn max_max(values: impl Iterator<Item = Option<f32>>) -> f32 {
    let mut max_value: f32 = 0.0;
    for value in values {
        let Some(value) = value else {
            return -1.0;
        };
        max_value = max_value.max(value);
    }
    max_value
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    const EDITOR54_CONSTRAINT_AGGREGATION_BENCH_V1: &str =
        "EDITOR54_CONSTRAINT_AGGREGATION_BENCH_V1";

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
        let production = include_str!("aggregation.rs")
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
}
