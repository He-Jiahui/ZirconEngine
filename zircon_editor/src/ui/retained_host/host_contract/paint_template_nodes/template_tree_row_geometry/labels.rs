use super::super::super::data::FrameRect;
use super::metrics::{
    tree_line_height, TREE_ACTION_GAP, TREE_ACTION_SIZE, TREE_RIGHT_INSET, TREE_TEXT_GAP,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_label_rect(
    rect: &FrameRect,
    icon: &FrameRect,
) -> FrameRect {
    let line_height = tree_line_height();
    let text_x = icon.x + icon.width + TREE_TEXT_GAP;
    let right_reserve = TREE_RIGHT_INSET + TREE_ACTION_SIZE * 2.0 + TREE_ACTION_GAP;
    FrameRect {
        x: text_x,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.x + rect.width - text_x - right_reserve).max(1.0),
        height: line_height,
    }
}
