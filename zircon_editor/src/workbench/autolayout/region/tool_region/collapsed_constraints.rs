use super::super::super::WorkbenchChromeMetrics;
use super::super::super::constraints::fixed_zero_constraints;
use super::super::super::{AxisConstraint, PaneConstraints, ShellRegionId, StretchMode};

pub(super) fn collapsed_region_constraints(
    region: ShellRegionId,
    metrics: &WorkbenchChromeMetrics,
) -> PaneConstraints {
    let collapsed_primary = match region {
        ShellRegionId::Bottom => metrics.panel_header_height,
        ShellRegionId::Left | ShellRegionId::Right => metrics.rail_width,
        ShellRegionId::Document => 0.0,
    };

    match region {
        ShellRegionId::Bottom => PaneConstraints {
            width: AxisConstraint {
                min: 0.0,
                max: -1.0,
                preferred: 0.0,
                priority: 50,
                weight: 1.0,
                stretch_mode: StretchMode::Stretch,
            },
            height: AxisConstraint {
                min: collapsed_primary,
                max: collapsed_primary,
                preferred: collapsed_primary,
                priority: 100,
                weight: 1.0,
                stretch_mode: StretchMode::Fixed,
            },
        },
        ShellRegionId::Left | ShellRegionId::Right => PaneConstraints {
            width: AxisConstraint {
                min: collapsed_primary,
                max: collapsed_primary,
                preferred: collapsed_primary,
                priority: 100,
                weight: 1.0,
                stretch_mode: StretchMode::Fixed,
            },
            height: AxisConstraint {
                min: 0.0,
                max: -1.0,
                preferred: 0.0,
                priority: 50,
                weight: 1.0,
                stretch_mode: StretchMode::Stretch,
            },
        },
        ShellRegionId::Document => fixed_zero_constraints(),
    }
}
