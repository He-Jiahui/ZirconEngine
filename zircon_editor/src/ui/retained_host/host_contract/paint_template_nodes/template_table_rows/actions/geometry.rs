use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::cells::table_content_offset;
use super::metrics::table_action_metrics;

pub(super) fn table_action_button_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let metrics = table_action_metrics();
    FrameRect {
        x: rect.x + rect.width - metrics.action_column_width
            + (metrics.action_column_width - metrics.button_size) * 0.5
            + content_offset_x,
        y: rect.y + (rect.height - metrics.button_size).max(0.0) * 0.5 + content_offset_y,
        width: metrics.button_size,
        height: metrics.button_size,
    }
}

pub(super) fn table_action_icon_rect(button_rect: &FrameRect) -> FrameRect {
    let metrics = table_action_metrics();
    FrameRect {
        x: button_rect.x + (button_rect.width - metrics.icon_size).max(0.0) * 0.5,
        y: button_rect.y + (button_rect.height - metrics.icon_size).max(0.0) * 0.5,
        width: metrics.icon_size,
        height: metrics.icon_size,
    }
}
