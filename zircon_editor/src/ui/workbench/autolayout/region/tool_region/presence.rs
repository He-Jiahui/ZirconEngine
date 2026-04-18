use std::collections::BTreeMap;

use crate::layout::{ActivityDrawerMode, ActivityDrawerSlot, WorkbenchLayout};
use crate::ui::workbench::model::WorkbenchViewModel;

use super::super::super::ShellRegionId;

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
    layout: &WorkbenchLayout,
    region: ShellRegionId,
    slots: &[ActivityDrawerSlot],
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> f32 {
    transient_region_preferred
        .and_then(|map| map.get(&region).copied())
        .unwrap_or_else(|| {
            slots
                .iter()
                .filter_map(|slot| layout.drawers.get(slot))
                .filter(|drawer| drawer.visible)
                .map(|drawer| drawer.extent)
                .fold(0.0_f32, f32::max)
        })
}
