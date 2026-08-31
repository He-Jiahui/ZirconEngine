use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;
use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;
use crate::ui::retained_host::route_intent::EditorRouteIntentMap;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Default)]
pub(crate) struct HostActivityRailPointerBridge {
    pub(super) layout: HostActivityRailPointerLayout,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) route_intents: EditorRouteIntentMap,
}

impl HostActivityRailPointerBridge {
    pub(crate) fn target_for_button(
        &self,
        side: HostActivityRailPointerSide,
        item_index: usize,
    ) -> Option<(ActivityDrawerSlot, &ViewInstanceId)> {
        let tabs = match side {
            HostActivityRailPointerSide::Left => &self.layout.left_tabs,
            HostActivityRailPointerSide::Right => &self.layout.right_tabs,
        };
        let tab = tabs.get(item_index)?;
        Some((tab.slot, &tab.instance_id))
    }
}
