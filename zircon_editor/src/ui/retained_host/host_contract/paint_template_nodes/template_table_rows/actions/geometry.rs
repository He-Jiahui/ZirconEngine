use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::cells::{table_content_offset, TABLE_ACTION_WIDTH};

const TABLE_ACTION_ICON_SIZE: f32 = 16.0;

pub(super) fn table_action_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let (content_offset_x, content_offset_y) = table_content_offset(node);
    FrameRect {
        x: rect.x + rect.width - TABLE_ACTION_WIDTH
            + (TABLE_ACTION_WIDTH - TABLE_ACTION_ICON_SIZE) * 0.5
            + content_offset_x,
        y: rect.y + (rect.height - TABLE_ACTION_ICON_SIZE).max(0.0) * 0.5 + content_offset_y,
        width: TABLE_ACTION_ICON_SIZE,
        height: TABLE_ACTION_ICON_SIZE,
    }
}
