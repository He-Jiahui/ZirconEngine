use super::super::super::super::data::FrameRect;
use super::super::super::UiProfileFrame;

pub(in crate::ui::retained_host::host_contract) fn intersect_profile_frame(
    left: &FrameRect,
    right: &UiProfileFrame,
) -> Option<UiProfileFrame> {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    (x1 > x0 && y1 > y0).then(|| UiProfileFrame {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}

pub(in crate::ui::retained_host::host_contract) fn intersect_frames(
    left: &FrameRect,
    right: &FrameRect,
) -> Option<FrameRect> {
    let x0 = left.x.max(right.x);
    let y0 = left.y.max(right.y);
    let x1 = (left.x + left.width).min(right.x + right.width);
    let y1 = (left.y + left.height).min(right.y + right.height);
    (x1 > x0 && y1 > y0).then(|| FrameRect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    })
}
