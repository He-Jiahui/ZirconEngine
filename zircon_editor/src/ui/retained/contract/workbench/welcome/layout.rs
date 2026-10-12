use super::super::super::data::FrameRect;
use super::super::super::paint_geometry::{is_visible_frame, translated};

pub(in crate::ui::retained_host::host_contract) const WELCOME_COLUMN_INSET: f32 = 18.0;
pub(in crate::ui::retained_host::host_contract) const WELCOME_CONTENT_MAX_WIDTH: f32 = 680.0;

pub(in crate::ui::retained_host::host_contract) fn translated_welcome_frame(
    frame: Option<&FrameRect>,
    body: &FrameRect,
) -> Option<FrameRect> {
    frame
        .map(|frame| translated(frame, body.x, body.y))
        .filter(is_visible_frame)
}

pub(in crate::ui::retained_host::host_contract) fn inset_frame(
    rect: &FrameRect,
    x: f32,
    y: f32,
) -> FrameRect {
    FrameRect {
        x: rect.x + x,
        y: rect.y + y,
        width: (rect.width - x * 2.0).max(0.0),
        height: (rect.height - y * 2.0).max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract) fn constrain_welcome_content(
    mut rect: FrameRect,
    x: f32,
    width: f32,
) -> FrameRect {
    rect.x = x;
    rect.width = width;
    rect
}
