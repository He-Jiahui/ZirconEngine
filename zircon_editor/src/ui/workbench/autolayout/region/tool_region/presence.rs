use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::WorkbenchViewModel;

use super::super::super::{LogicalRegionPreferredExtents, ShellRegionId};

pub(super) fn tool_region_has_tabs(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> bool {
    let drawers_visible = model.drawer_ring.visible;
    drawers_visible
        && slots.iter().any(|slot| {
            model
                .tool_windows
                .get(slot)
                .is_some_and(|stack| stack.visible && !stack.tabs.is_empty())
        })
}

pub(super) fn tool_region_is_expanded(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> bool {
    let drawers_visible = model.drawer_ring.visible;
    drawers_visible
        && slots.iter().any(|slot| {
            model.tool_windows.get(slot).is_some_and(|stack| {
                stack.visible
                    && !stack.tabs.is_empty()
                    && stack.mode != ActivityDrawerMode::Collapsed
            })
        })
}

pub(super) fn tool_region_extent(
    model: &WorkbenchViewModel,
    region: ShellRegionId,
    slots: &[ActivityDrawerSlot],
    transient_region_preferred: LogicalRegionPreferredExtents<'_>,
    token_region_preferred: LogicalRegionPreferredExtents<'_>,
) -> f32 {
    transient_region_preferred
        .get(region)
        .or_else(|| persisted_tool_region_extent(model, slots))
        .or_else(|| token_region_preferred.get(region))
        .unwrap_or(0.0)
}

fn persisted_tool_region_extent(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Option<f32> {
    slots
        .iter()
        .filter_map(|slot| model.drawer_ring.drawers.get(slot))
        .filter(|drawer| drawer.visible)
        .map(|drawer| drawer.extent)
        .fold(None, |maximum, extent| {
            Some(maximum.map_or(extent, |current: f32| current.max(extent)))
        })
}
