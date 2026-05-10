use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::constants::ITEM_NODE_ID_BASE;

pub(super) fn item_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(ITEM_NODE_ID_BASE + index as u64)
}
