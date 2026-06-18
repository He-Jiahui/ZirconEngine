use super::super::super::data::{FrameRect, PaneData};
use super::super::super::paint_geometry::{frame_from_template, is_visible_frame, translated};

pub(super) const WELCOME_COLUMN_INSET: f32 = 18.0;
pub(super) const WELCOME_CONTENT_MAX_WIDTH: f32 = 680.0;
pub(super) const WELCOME_ROW_HEIGHT: f32 = 54.0;
pub(super) const WELCOME_ROW_GAP: f32 = 8.0;

pub(super) fn welcome_node_frame(
    pane: &PaneData,
    body: &FrameRect,
    control_id: &str,
) -> Option<FrameRect> {
    (0..pane.welcome.nodes.row_count())
        .filter_map(|row| pane.welcome.nodes.row_data(row))
        .find_map(|node| {
            (node.control_id.as_str() == control_id)
                .then(|| translated(&frame_from_template(&node.frame), body.x, body.y))
                .filter(is_visible_frame)
        })
}

pub(super) fn inset_frame(rect: &FrameRect, x: f32, y: f32) -> FrameRect {
    FrameRect {
        x: rect.x + x,
        y: rect.y + y,
        width: (rect.width - x * 2.0).max(0.0),
        height: (rect.height - y * 2.0).max(0.0),
    }
}

pub(super) fn constrain_welcome_content(mut rect: FrameRect, x: f32, width: f32) -> FrameRect {
    rect.x = x;
    rect.width = width;
    rect
}
