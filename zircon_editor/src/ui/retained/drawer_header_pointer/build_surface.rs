use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::host_drawer_header_pointer_item::HostDrawerHeaderPointerItem;
use super::host_drawer_header_pointer_surface::HostDrawerHeaderPointerSurface;

pub(super) fn build_surface(
    key: &'static str,
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Option<HostDrawerHeaderPointerSurface> {
    let items = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack
                .tabs
                .iter()
                .map(move |tab| HostDrawerHeaderPointerItem {
                    slot: stack.slot,
                    instance_id: tab.instance_id.clone(),
                })
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        return None;
    }

    Some(HostDrawerHeaderPointerSurface { key, items })
}
