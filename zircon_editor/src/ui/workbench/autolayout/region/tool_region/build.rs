use std::collections::{BTreeMap, HashMap};

use crate::ui::workbench::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{ViewDescriptor, ViewDescriptorId};

use super::super::super::active_tab::active_tool_tab;
use super::super::super::constraints::{
    default_constraints_for_content, default_region_constraints, fixed_zero_constraints,
    merge_constraints, set_primary_preferred,
};
use super::super::super::region_state::RegionState;
use super::super::super::{PaneConstraints, ShellRegionId, WorkbenchChromeMetrics};
use super::collapsed_constraints::collapsed_region_constraints;
use super::presence::{tool_region_extent, tool_region_has_tabs, tool_region_is_expanded};

pub(crate) fn build_tool_region_state(
    model: &WorkbenchViewModel,
    layout: &WorkbenchLayout,
    descriptors: &HashMap<ViewDescriptorId, &ViewDescriptor>,
    region: ShellRegionId,
    slots: &[ActivityDrawerSlot],
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
    metrics: &WorkbenchChromeMetrics,
) -> RegionState {
    let tab = active_tool_tab(model, slots);
    let has_tabs = tool_region_has_tabs(model, slots);
    let expanded = tool_region_is_expanded(model, slots);
    let extent = tool_region_extent(model, region, slots, transient_region_preferred);

    if !has_tabs {
        return RegionState {
            visible: false,
            expanded: false,
            constraints: fixed_zero_constraints(),
        };
    }

    if !expanded {
        return RegionState {
            visible: true,
            expanded: false,
            constraints: collapsed_region_constraints(region, metrics),
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
