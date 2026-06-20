use super::super::super::data::FrameRect;
use super::metrics::{TREE_ACTION_GAP, TREE_ACTION_SIZE, TREE_RIGHT_INSET};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_rect(
    rect: &FrameRect,
    index_from_right: usize,
) -> FrameRect {
    let stride = TREE_ACTION_SIZE + TREE_ACTION_GAP;
    FrameRect {
        x: rect.x + rect.width
            - TREE_RIGHT_INSET
            - TREE_ACTION_SIZE
            - index_from_right as f32 * stride,
        y: rect.y + (rect.height - TREE_ACTION_SIZE).max(0.0) * 0.5,
        width: TREE_ACTION_SIZE,
        height: TREE_ACTION_SIZE,
    }
}
