use super::super::{AxisConstraint, PaneConstraints, StretchMode};

pub(crate) fn fixed_zero_constraints() -> PaneConstraints {
    PaneConstraints {
        width: fixed_axis(0.0),
        height: fixed_axis(0.0),
    }
}

pub(super) fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

pub(super) fn stretch_axis(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}
