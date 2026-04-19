use crate::DockEdge;
use zircon_runtime::ui::event_ui::UiNodeId;

pub(super) const DRAG_POINTER_ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const DRAG_TARGET_DOCUMENT_NODE_ID: UiNodeId = UiNodeId::new(2);
pub(super) const DRAG_TARGET_LEFT_NODE_ID: UiNodeId = UiNodeId::new(3);
pub(super) const DRAG_TARGET_RIGHT_NODE_ID: UiNodeId = UiNodeId::new(4);
pub(super) const DRAG_TARGET_BOTTOM_NODE_ID: UiNodeId = UiNodeId::new(5);
pub(super) const DOCUMENT_EDGE_LEFT_NODE_ID: UiNodeId = UiNodeId::new(6);
pub(super) const DOCUMENT_EDGE_RIGHT_NODE_ID: UiNodeId = UiNodeId::new(7);
pub(super) const DOCUMENT_EDGE_TOP_NODE_ID: UiNodeId = UiNodeId::new(8);
pub(super) const DOCUMENT_EDGE_BOTTOM_NODE_ID: UiNodeId = UiNodeId::new(9);

pub(super) const RESIZE_POINTER_ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const RESIZE_TARGET_LEFT_NODE_ID: UiNodeId = UiNodeId::new(10);
pub(super) const RESIZE_TARGET_RIGHT_NODE_ID: UiNodeId = UiNodeId::new(11);
pub(super) const RESIZE_TARGET_BOTTOM_NODE_ID: UiNodeId = UiNodeId::new(12);

const FLOATING_NODE_ID_BASE: u64 = 1_000;
const FLOATING_NODE_STRIDE: u64 = 10;

pub(super) fn floating_window_attach_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(FLOATING_NODE_ID_BASE + index as u64 * FLOATING_NODE_STRIDE)
}

pub(super) fn floating_window_projection_exclusion_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(FLOATING_NODE_ID_BASE + index as u64 * FLOATING_NODE_STRIDE + 5)
}

pub(super) fn floating_window_edge_node_id(index: usize, edge: DockEdge) -> UiNodeId {
    let offset = match edge {
        DockEdge::Left => 1,
        DockEdge::Right => 2,
        DockEdge::Top => 3,
        DockEdge::Bottom => 4,
    };
    UiNodeId::new(FLOATING_NODE_ID_BASE + index as u64 * FLOATING_NODE_STRIDE + offset)
}
