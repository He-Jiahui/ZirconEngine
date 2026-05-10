use zircon_runtime_interface::ui::layout::UiFrame;

use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::drawer_slot_key::drawer_slot_key;
use super::host_drawer_header_pointer_item::HostDrawerHeaderPointerItem;
use super::host_drawer_header_pointer_surface::HostDrawerHeaderPointerSurface;

pub(super) fn build_surface(
    key: &str,
    region_frame: UiFrame,
    model: &WorkbenchViewModel,
    slots: &[crate::ui::workbench::layout::ActivityDrawerSlot],
    metrics: &WorkbenchChromeMetrics,
    side_with_rail: bool,
) -> Option<HostDrawerHeaderPointerSurface> {
    let items = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack
                .tabs
                .iter()
                .map(move |tab| HostDrawerHeaderPointerItem {
                    slot: drawer_slot_key(stack.slot).to_string(),
                    instance_id: tab.instance_id.0.clone(),
                })
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        return None;
    }

    let strip_frame = if side_with_rail {
        if region_frame.width <= metrics.rail_width + metrics.separator_thickness {
            return None;
        }
        UiFrame::new(
            region_frame.x + metrics.rail_width + metrics.separator_thickness,
            region_frame.y,
            (region_frame.width - metrics.rail_width - metrics.separator_thickness).max(0.0),
            metrics.panel_header_height,
        )
    } else if key == "right" {
        if region_frame.width <= metrics.rail_width + metrics.separator_thickness {
            return None;
        }
        UiFrame::new(
            region_frame.x,
            region_frame.y,
            (region_frame.width - metrics.rail_width - metrics.separator_thickness).max(0.0),
            metrics.panel_header_height,
        )
    } else {
        UiFrame::new(
            region_frame.x,
            region_frame.y,
            region_frame.width.max(0.0),
            metrics.panel_header_height,
        )
    };

    Some(HostDrawerHeaderPointerSurface {
        key: key.to_string(),
        strip_frame,
        items,
    })
}
