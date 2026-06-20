use super::super::super::super::data::FrameRect;

pub(super) fn centered_square(rect: &FrameRect) -> FrameRect {
    let size = rect.width.min(rect.height).max(0.0);
    FrameRect {
        x: (rect.x + (rect.width - size).max(0.0) * 0.5).round(),
        y: (rect.y + (rect.height - size).max(0.0) * 0.5).round(),
        width: size.round().max(1.0),
        height: size.round().max(1.0),
    }
}
