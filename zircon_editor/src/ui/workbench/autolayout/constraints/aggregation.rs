use super::super::{AxisConstraint, PaneConstraints, StretchMode};
use super::axis_factory::fixed_zero_constraints;

pub(crate) fn aggregate_row_constraints(children: &[PaneConstraints]) -> PaneConstraints {
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
