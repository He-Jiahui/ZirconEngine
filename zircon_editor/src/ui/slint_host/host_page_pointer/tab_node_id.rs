use zircon_runtime::ui::event_ui::UiNodeId;

use super::constants::TAB_NODE_ID_BASE;

pub(super) fn tab_node_id(item_index: usize) -> UiNodeId {
    UiNodeId::new(TAB_NODE_ID_BASE + item_index as u64)
}
