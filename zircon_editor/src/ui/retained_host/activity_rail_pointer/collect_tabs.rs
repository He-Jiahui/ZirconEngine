use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::drawer_slot_key::drawer_slot_key;
use super::host_activity_rail_pointer_item::HostActivityRailPointerItem;

pub(super) fn collect_tabs(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Vec<HostActivityRailPointerItem> {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack
                .tabs
                .iter()
                .map(move |tab| HostActivityRailPointerItem {
                    slot: drawer_slot_key(stack.slot).to_string(),
                    instance_id: tab.instance_id.0.clone(),
                })
        })
        .collect()
}
