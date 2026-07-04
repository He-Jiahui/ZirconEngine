use super::super::super::data::FrameRect;
use super::metrics::tree_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_rect(
    rect: &FrameRect,
    index_from_right: usize,
) -> FrameRect {
    let metrics = tree_metrics();
    let stride = metrics.tree_action_size + metrics.tree_action_gap;
    FrameRect {
        x: rect.x + rect.width
            - metrics.tree_right_inset
            - metrics.tree_action_size
            - index_from_right as f32 * stride,
        y: rect.y + (rect.height - metrics.tree_action_size).max(0.0) * 0.5,
        width: metrics.tree_action_size,
        height: metrics.tree_action_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_button_rect(
    rect: &FrameRect,
    index_from_right: usize,
) -> FrameRect {
    let icon = tree_action_rect(rect, index_from_right);
    let metrics = tree_metrics();
    FrameRect {
        x: icon.x + (icon.width - metrics.tree_action_button_size) * 0.5,
        y: icon.y + (icon.height - metrics.tree_action_button_size) * 0.5,
        width: metrics.tree_action_button_size,
        height: metrics.tree_action_button_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_action_icon_rect(
    button_rect: &FrameRect,
) -> FrameRect {
    let metrics = tree_metrics();
    FrameRect {
        x: button_rect.x + (button_rect.width - metrics.tree_action_size).max(0.0) * 0.5,
        y: button_rect.y + (button_rect.height - metrics.tree_action_size).max(0.0) * 0.5,
        width: metrics.tree_action_size,
        height: metrics.tree_action_size,
    }
}
