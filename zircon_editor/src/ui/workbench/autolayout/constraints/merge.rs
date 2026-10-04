use super::super::axis_constraint_override::AxisConstraintOverride;
use super::super::pane_constraint_override::PaneConstraintOverride;
use super::super::{AxisConstraint, PaneConstraints, ShellRegionId};

/// 合并区域保底、descriptor与两层差量；view实例覆盖优先于区域覆盖。
pub(crate) fn merge_constraints(
    region_defaults: PaneConstraints,
    region_override: Option<PaneConstraintOverride>,
    descriptor_defaults: PaneConstraints,
    view_override: Option<PaneConstraintOverride>,
) -> PaneConstraints {
    PaneConstraints {
        width: merge_axis(
            region_defaults.width,
            region_override.map(|override_set| override_set.width),
            descriptor_defaults.width,
            view_override.map(|override_set| override_set.width),
        ),
        height: merge_axis(
            region_defaults.height,
            region_override.map(|override_set| override_set.height),
            descriptor_defaults.height,
            view_override.map(|override_set| override_set.height),
        ),
    }
}

/// 将拖动/持久化尺寸意图写到区域主轴；保留内容最小值、上限和伸展策略。
pub(crate) fn set_primary_preferred(
    region: ShellRegionId,
    mut constraints: PaneConstraints,
    preferred: f32,
) -> PaneConstraints {
    match region {
        ShellRegionId::Bottom => constraints.height.preferred = preferred,
        ShellRegionId::Left | ShellRegionId::Right | ShellRegionId::Document => {
            constraints.width.preferred = preferred;
        }
    }
    constraints
}

/// 完全默认的descriptor轴表示采用区域基线，随后逐层保留未设置字段。
fn merge_axis(
    region_default: AxisConstraint,
    region_override: Option<AxisConstraintOverride>,
    descriptor_default: AxisConstraint,
    view_override: Option<AxisConstraintOverride>,
) -> AxisConstraint {
    let mut axis = descriptor_default;
    if descriptor_default == AxisConstraint::default() {
        axis = region_default;
    }
    if let Some(override_axis) = region_override {
        axis = override_axis.apply_to(axis);
    }
    if let Some(override_axis) = view_override {
        axis = override_axis.apply_to(axis);
    }
    axis
}
