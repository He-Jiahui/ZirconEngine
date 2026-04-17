use zircon_ui::UiNodeId;

use super::constants::{LEFT_BUTTON_NODE_ID_BASE, RIGHT_BUTTON_NODE_ID_BASE};
use super::workbench_activity_rail_pointer_side::WorkbenchActivityRailPointerSide;

pub(super) fn strip_button_node_id(
    side: WorkbenchActivityRailPointerSide,
    index: usize,
) -> UiNodeId {
    let base = match side {
        WorkbenchActivityRailPointerSide::Left => LEFT_BUTTON_NODE_ID_BASE,
        WorkbenchActivityRailPointerSide::Right => RIGHT_BUTTON_NODE_ID_BASE,
    };
    UiNodeId::new(base + index as u64)
}
