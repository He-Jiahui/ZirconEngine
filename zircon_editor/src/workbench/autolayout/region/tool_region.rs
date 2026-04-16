use std::collections::{BTreeMap, HashMap};

use crate::layout::{ActivityDrawerMode, ActivityDrawerSlot, WorkbenchLayout};
use crate::snapshot::ViewContentKind;
use crate::view::{ViewDescriptor, ViewDescriptorId};
use crate::workbench::model::WorkbenchViewModel;

use super::super::active_tab::active_tool_tab;
use super::super::constraints::{
    default_constraints_for_content, default_region_constraints, fixed_zero_constraints,
    merge_constraints, set_primary_preferred,
};
use super::super::region_state::RegionState;
use super::super::{PaneConstraints, ShellRegionId, WorkbenchChromeMetrics};

pub(crate) fn build_tool_region_state(
    model: &WorkbenchViewModel,
    layout: &WorkbenchLayout,
    descriptors: &HashMap<ViewDescriptorId, &ViewDescriptor>,
    region: ShellRegionId,
    slots: &[ActivityDrawerSlot],
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
    metrics: &WorkbenchChromeMetrics,
) -> RegionState {
    let drawers_visible = model.drawer_ring.visible;
    let tab = active_tool_tab(model, slots);
    let has_tabs = drawers_visible
        && slots.iter().any(|slot| {
            model
                .tool_windows
                .get(slot)
                .is_some_and(|stack| stack.visible && !stack.tabs.is_empty())
        });
    let expanded = drawers_visible
        && slots.iter().any(|slot| {
            model.tool_windows.get(slot).is_some_and(|stack| {
                stack.visible
                    && !stack.tabs.is_empty()
                    && stack.mode != ActivityDrawerMode::Collapsed
            })
        });
    let extent = transient_region_preferred
        .and_then(|map| map.get(&region).copied())
        .unwrap_or_else(|| {
            slots
                .iter()
                .filter_map(|slot| layout.drawers.get(slot))
                .filter(|drawer| drawer.visible)
                .map(|drawer| drawer.extent)
                .fold(0.0_f32, f32::max)
        });

    if !has_tabs {
        return RegionState {
            visible: false,
            expanded: false,
            constraints: fixed_zero_constraints(),
        };
    }

    if !expanded {
        let collapsed_primary = match region {
            ShellRegionId::Bottom => metrics.panel_header_height,
            ShellRegionId::Left | ShellRegionId::Right => metrics.rail_width,
            ShellRegionId::Document => 0.0,
        };
        return RegionState {
            visible: true,
            expanded: false,
            constraints: match region {
                ShellRegionId::Bottom => PaneConstraints {
                    width: super::super::AxisConstraint {
                        min: 0.0,
                        max: -1.0,
                        preferred: 0.0,
                        priority: 50,
                        weight: 1.0,
                        stretch_mode: super::super::StretchMode::Stretch,
                    },
                    height: super::super::AxisConstraint {
                        min: collapsed_primary,
                        max: collapsed_primary,
                        preferred: collapsed_primary,
                        priority: 100,
                        weight: 1.0,
                        stretch_mode: super::super::StretchMode::Fixed,
                    },
                },
                ShellRegionId::Left | ShellRegionId::Right => PaneConstraints {
                    width: super::super::AxisConstraint {
                        min: collapsed_primary,
                        max: collapsed_primary,
                        preferred: collapsed_primary,
                        priority: 100,
                        weight: 1.0,
                        stretch_mode: super::super::StretchMode::Fixed,
                    },
                    height: super::super::AxisConstraint {
                        min: 0.0,
                        max: -1.0,
                        preferred: 0.0,
                        priority: 50,
                        weight: 1.0,
                        stretch_mode: super::super::StretchMode::Stretch,
                    },
                },
                ShellRegionId::Document => fixed_zero_constraints(),
            },
        };
    }

    let descriptor_constraints = tab
        .and_then(|tab| descriptors.get(&tab.descriptor_id).copied())
        .map(|descriptor| {
            if descriptor.default_constraints == PaneConstraints::default() {
                default_constraints_for_content(
                    tab.map(|tab| tab.content_kind)
                        .unwrap_or(ViewContentKind::Placeholder),
                )
            } else {
                descriptor.default_constraints
            }
        })
        .unwrap_or_else(|| default_region_constraints(region));
    let layout_override = layout.region_overrides.get(&region).copied();
    let view_override = tab.and_then(|tab| layout.view_overrides.get(&tab.instance_id).copied());
    let constraints = set_primary_preferred(
        region,
        merge_constraints(
            default_region_constraints(region),
            layout_override,
            descriptor_constraints,
            view_override,
        ),
        extent.max(0.0),
    );

    RegionState {
        visible: true,
        expanded: true,
        constraints,
    }
}
