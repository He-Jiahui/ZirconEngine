use super::super::data::FrameRect;
use super::frame::is_visible_frame;

pub(in crate::ui::retained_host::host_contract) fn translated(
    frame: &FrameRect,
    origin_x: f32,
    origin_y: f32,
) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

pub(in crate::ui::retained_host::host_contract) fn intersect(
    left: &FrameRect,
    right: &FrameRect,
) -> Option<FrameRect> {
    if !is_visible_frame(left) || !is_visible_frame(right) {
        return None;
    }

    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    let width = x1 - x0;
    let height = y1 - y0;
    (width > 0.5 && height > 0.5).then_some(FrameRect {
        x: x0,
        y: y0,
        width,
        height,
    })
}

pub(in crate::ui::retained_host::host_contract) fn inset(
    frame: &FrameRect,
    amount: f32,
) -> FrameRect {
    let amount = amount.max(0.0);
    FrameRect {
        x: frame.x + amount,
        y: frame.y + amount,
        width: (frame.width - amount * 2.0).max(0.0),
        height: (frame.height - amount * 2.0).max(0.0),
    }
}
