use super::super::super::data::FrameRect;

pub(super) fn inset_rect(frame: &FrameRect, x: f32, y: f32) -> FrameRect {
    FrameRect {
        x: frame.x + x,
        y: frame.y + y,
        width: (frame.width - x * 2.0).max(1.0),
        height: (frame.height - y * 2.0).max(1.0),
    }
}
