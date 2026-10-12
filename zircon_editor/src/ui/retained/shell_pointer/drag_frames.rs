use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiFrame};

use super::node_ids::{
    floating_window_index_for_node, DOCUMENT_EDGE_BOTTOM_NODE_ID, DOCUMENT_EDGE_LEFT_NODE_ID,
    DOCUMENT_EDGE_RIGHT_NODE_ID, DOCUMENT_EDGE_TOP_NODE_ID, DRAG_TARGET_BOTTOM_NODE_ID,
    DRAG_TARGET_DOCUMENT_NODE_ID, DRAG_TARGET_LEFT_NODE_ID, DRAG_TARGET_RIGHT_NODE_ID,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct DragTargetFrames {
    pub(super) left: UiFrame,
    pub(super) right: UiFrame,
    pub(super) bottom: UiFrame,
    pub(super) document: UiFrame,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct DragHitGeometry {
    pub(super) targets: DragTargetFrames,
    document_drag_frame: Option<UiFrame>,
    left_drag_frame: Option<UiFrame>,
    right_drag_frame: Option<UiFrame>,
    bottom_drag_frame: Option<UiFrame>,
    document_edge_frame: Option<UiFrame>,
    floating_frames: Vec<Option<UiFrame>>,
}

impl DragHitGeometry {
    pub(super) fn new(
        document_drag_frame: Option<UiFrame>,
        left_drag_frame: Option<UiFrame>,
        right_drag_frame: Option<UiFrame>,
        bottom_drag_frame: Option<UiFrame>,
        document_edge_frame: Option<UiFrame>,
        floating_frames: Vec<Option<UiFrame>>,
    ) -> Self {
        Self {
            targets: DragTargetFrames {
                left: left_drag_frame.unwrap_or_default(),
                right: right_drag_frame.unwrap_or_default(),
                bottom: bottom_drag_frame.unwrap_or_default(),
                document: document_edge_frame.unwrap_or_default(),
            },
            document_drag_frame,
            left_drag_frame,
            right_drag_frame,
            bottom_drag_frame,
            document_edge_frame,
            floating_frames,
        }
    }

    pub(super) fn frame(&self, node_id: UiNodeId) -> Option<UiFrame> {
        match node_id {
            DRAG_TARGET_DOCUMENT_NODE_ID => self.document_drag_frame,
            DRAG_TARGET_LEFT_NODE_ID => self.left_drag_frame,
            DRAG_TARGET_RIGHT_NODE_ID => self.right_drag_frame,
            DRAG_TARGET_BOTTOM_NODE_ID => self.bottom_drag_frame,
            DOCUMENT_EDGE_LEFT_NODE_ID
            | DOCUMENT_EDGE_RIGHT_NODE_ID
            | DOCUMENT_EDGE_TOP_NODE_ID
            | DOCUMENT_EDGE_BOTTOM_NODE_ID => self.document_edge_frame,
            _ => floating_window_index_for_node(node_id)
                .and_then(|index| self.floating_frames.get(index).copied().flatten()),
        }
    }
}
