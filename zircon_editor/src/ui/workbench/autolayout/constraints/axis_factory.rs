use super::super::{AxisConstraint, PaneConstraints, StretchMode};

/// 不存在内容的壳区域占位；可见性仍由RegionState独立声明。
pub(crate) fn fixed_zero_constraints() -> PaneConstraints {
    PaneConstraints {
        width: fixed_axis(0.0),
        height: fixed_axis(0.0),
    }
}

/// 固定chrome轴需求，避免伸展内容挤占已约定的壳装饰尺寸。
pub(crate) fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

/// 默认内容可伸展轴；优先级与权重交由共享求解器分配剩余空间。
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
