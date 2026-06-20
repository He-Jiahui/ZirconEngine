use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::cells::{table_content_offset, TABLE_ACTION_WIDTH};

pub(super) fn table_action_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    FrameRect {
        x: rect.x + rect.width - TABLE_ACTION_WIDTH + 7.0 + content_offset_x,
        y: rect.y + (rect.height - 14.0).max(0.0) * 0.5 + content_offset_y,
        width: 14.0,
        height: 14.0,
    }
}
