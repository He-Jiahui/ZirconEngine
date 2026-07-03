use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::METRICS;
use super::super::cells::{table_content_offset, TABLE_ACTION_WIDTH};

pub(super) fn table_action_button_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    let button_size = table_action_button_size();
    FrameRect {
        x: rect.x + rect.width - TABLE_ACTION_WIDTH
            + (TABLE_ACTION_WIDTH - button_size) * 0.5
            + content_offset_x,
        y: rect.y + (rect.height - button_size).max(0.0) * 0.5 + content_offset_y,
        width: button_size,
        height: button_size,
    }
}

pub(super) fn table_action_icon_rect(button_rect: &FrameRect) -> FrameRect {
    let icon_size = table_action_icon_size();
    FrameRect {
        x: button_rect.x + (button_rect.width - icon_size).max(0.0) * 0.5,
        y: button_rect.y + (button_rect.height - icon_size).max(0.0) * 0.5,
        width: icon_size,
        height: icon_size,
    }
}

fn table_action_button_size() -> f32 {
    TABLE_ACTION_WIDTH - METRICS.gap_s
}

fn table_action_icon_size() -> f32 {
    METRICS.gap_m * 2.0
}
