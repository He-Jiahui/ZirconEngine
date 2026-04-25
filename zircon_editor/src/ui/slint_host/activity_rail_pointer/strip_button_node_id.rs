use zircon_runtime::ui::event_ui::UiNodeId;

use super::constants::{LEFT_BUTTON_NODE_ID_BASE, RIGHT_BUTTON_NODE_ID_BASE};
use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;

pub(super) fn strip_button_node_id(side: HostActivityRailPointerSide, index: usize) -> UiNodeId {
    let base = match side {
        HostActivityRailPointerSide::Left => LEFT_BUTTON_NODE_ID_BASE,
        HostActivityRailPointerSide::Right => RIGHT_BUTTON_NODE_ID_BASE,
    };
    UiNodeId::new(base + index as u64)
}
