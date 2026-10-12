use super::super::data::{FrameRect, TemplateNodeFrameData};

pub(in crate::ui::retained_host::host_contract) fn frame_or(
    frame: &FrameRect,
    fallback: FrameRect,
) -> FrameRect {
    if is_visible_frame(frame) {
        frame.clone()
    } else {
        fallback
    }
}

pub(in crate::ui::retained_host::host_contract) fn is_visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.5
        && frame.height > 0.5
}

pub(in crate::ui::retained_host::host_contract) fn frame_from_template(
    frame: &TemplateNodeFrameData,
) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
