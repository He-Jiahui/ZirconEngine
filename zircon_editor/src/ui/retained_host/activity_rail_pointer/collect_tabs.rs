use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::host_activity_rail_pointer_item::HostActivityRailPointerItem;

pub(super) fn collect_tabs(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Vec<HostActivityRailPointerItem> {
    let tabs = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack
                .tabs
                .iter()
                .map(move |tab| HostActivityRailPointerItem {
                    slot: stack.slot,
                    instance_id: tab.instance_id.clone(),
                })
        })
        .collect::<Vec<_>>();
    zircon_runtime::profile_counter!("editor", "ui.activity_rail.projection_batch_count", 1);
    zircon_runtime::profile_counter!(
        "editor",
        "ui.activity_rail.projection_visit_count",
        tabs.len()
    );
    tabs
}
