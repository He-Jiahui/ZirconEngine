use super::super::super::data::FrameRect;
use super::metrics::{tree_line_height, tree_metrics};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_label_rect(
    rect: &FrameRect,
    icon: &FrameRect,
) -> FrameRect {
    let metrics = tree_metrics();
    let line_height = tree_line_height();
    let text_x = icon.x + icon.width + metrics.tree_text_gap;
    let right_reserve =
        metrics.tree_right_inset + metrics.tree_action_size * 2.0 + metrics.tree_action_gap;
    FrameRect {
        x: text_x,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.x + rect.width - text_x - right_reserve).max(0.0),
        height: line_height,
    }
}
