use super::super::{AxisConstraint, PaneConstraints, StretchMode};
use super::axis_factory::fixed_zero_constraints;

/// 把并排区域合为父容器需求：宽度累加、高度取共同包络；分隔线另由调用方计入。
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
// 以下仅用于测试参考路径，保留旧聚合口径以验证单遍实现的边界语义。
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
#[path = "tests/aggregation.rs"]
mod tests;
