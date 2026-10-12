use super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(in crate::ui::retained_host::host_contract) fn clipped_frame(
    frame: &FrameRect,
    clip: Option<&FrameRect>,
) -> Option<FrameRect> {
    match clip {
        Some(clip) => intersect_frames(frame, clip),
        None if visible_frame(frame) => Some(frame.clone()),
        None => None,
    }
}

fn intersect_frames(left: &FrameRect, right: &FrameRect) -> Option<FrameRect> {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    let frame = FrameRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    };
    visible_frame(&frame).then_some(frame)
}
