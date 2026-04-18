use super::super::axis_constraint_override::AxisConstraintOverride;
use super::super::pane_constraint_override::PaneConstraintOverride;
use super::super::{AxisConstraint, PaneConstraints, ShellRegionId};

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
