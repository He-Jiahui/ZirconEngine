use crate::snapshot::ViewContentKind;

use super::axis_constraint_override::AxisConstraintOverride;
use super::pane_constraint_override::PaneConstraintOverride;
use super::{AxisConstraint, PaneConstraints, ShellRegionId, StretchMode};

pub(super) fn merge_constraints(
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

pub(super) fn set_primary_preferred(
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

pub(super) fn aggregate_row_constraints(children: &[PaneConstraints]) -> PaneConstraints {
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

pub fn default_region_constraints(region: ShellRegionId) -> PaneConstraints {
    match region {
        ShellRegionId::Left | ShellRegionId::Right => PaneConstraints {
            width: stretch_axis(240.0, 308.0, 50, 1.0),
            height: stretch_axis(180.0, 320.0, 50, 1.0),
        },
        ShellRegionId::Bottom => PaneConstraints {
            width: stretch_axis(0.0, 0.0, 50, 1.0),
            height: stretch_axis(120.0, 164.0, 50, 1.0),
        },
        ShellRegionId::Document => PaneConstraints {
            width: stretch_axis(520.0, 960.0, 100, 3.0),
            height: stretch_axis(280.0, 640.0, 100, 3.0),
        },
    }
}

pub fn default_constraints_for_content(kind: ViewContentKind) -> PaneConstraints {
    match kind {
        ViewContentKind::Welcome => default_region_constraints(ShellRegionId::Document),
        ViewContentKind::Scene | ViewContentKind::Game | ViewContentKind::PrefabEditor => {
            PaneConstraints {
                width: stretch_axis(640.0, 1080.0, 100, 4.0),
                height: stretch_axis(360.0, 720.0, 100, 4.0),
            }
        }
        ViewContentKind::Inspector => PaneConstraints {
            width: stretch_axis(260.0, 312.0, 60, 1.0),
            height: stretch_axis(220.0, 360.0, 60, 1.0),
        },
        ViewContentKind::Hierarchy | ViewContentKind::Project => PaneConstraints {
            width: stretch_axis(220.0, 280.0, 55, 1.0),
            height: stretch_axis(180.0, 320.0, 55, 1.0),
        },
        ViewContentKind::Console => PaneConstraints {
            width: stretch_axis(0.0, 0.0, 50, 1.0),
            height: stretch_axis(140.0, 200.0, 50, 1.0),
        },
        ViewContentKind::Assets
        | ViewContentKind::AssetBrowser
        | ViewContentKind::UiAssetEditor => PaneConstraints {
            width: stretch_axis(420.0, 720.0, 80, 2.0),
            height: stretch_axis(260.0, 480.0, 80, 2.0),
        },
        ViewContentKind::Placeholder => default_region_constraints(ShellRegionId::Document),
    }
}

pub(super) fn fixed_zero_constraints() -> PaneConstraints {
    PaneConstraints {
        width: fixed_axis(0.0),
        height: fixed_axis(0.0),
    }
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

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn stretch_axis(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}
