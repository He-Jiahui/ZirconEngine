use super::super::super::data::{FrameRect, TemplateNodeFrameData};
use super::super::{UiProfileFrame, UiProfileNamedFrame, UiProfilePoint};

pub(super) fn push_named_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: &str,
    kind: &str,
    surface: &str,
    frame: FrameRect,
    clip: Option<FrameRect>,
) {
    if !is_visible_frame(&frame) {
        return;
    }
    push_named_profile_frame(out, id, kind, surface, frame.into(), clip.map(Into::into));
}

pub(super) fn push_named_profile_frame(
    out: &mut Vec<UiProfileNamedFrame>,
    id: &str,
    kind: &str,
    surface: &str,
    frame: UiProfileFrame,
    clip: Option<UiProfileFrame>,
) {
    if !is_visible_profile_frame(&frame) {
        return;
    }
    out.push(UiProfileNamedFrame {
        id: id.to_string(),
        kind: kind.to_string(),
        surface: surface.to_string(),
        frame,
        clip,
    });
}

pub(super) fn visible_profile_frame(frame: &FrameRect) -> Option<UiProfileFrame> {
    is_visible_frame(frame).then(|| frame.into())
}

pub(super) fn is_visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(super) fn is_visible_profile_frame(frame: &UiProfileFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

pub(super) fn intersect_profile_frame(
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

pub(super) fn intersect_frames(left: &FrameRect, right: &FrameRect) -> Option<FrameRect> {
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

pub(super) fn profile_frame_center(frame: &UiProfileFrame) -> UiProfilePoint {
    UiProfilePoint {
        x: frame.x + frame.width * 0.5,
        y: frame.y + frame.height * 0.5,
    }
}

pub(super) fn frame_rect_center_point(frame: &FrameRect) -> UiProfilePoint {
    UiProfilePoint {
        x: frame.x + frame.width * 0.5,
        y: frame.y + frame.height * 0.5,
    }
}

pub(super) fn translated(frame: &FrameRect, origin_x: f32, origin_y: f32) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

pub(super) fn translated_template_frame(
    frame: &TemplateNodeFrameData,
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
