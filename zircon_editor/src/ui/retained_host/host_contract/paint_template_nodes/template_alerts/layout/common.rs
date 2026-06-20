use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

pub(super) fn centered_rect(rect: &FrameRect, left: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: rect.x + left,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width,
        height,
    }
}
