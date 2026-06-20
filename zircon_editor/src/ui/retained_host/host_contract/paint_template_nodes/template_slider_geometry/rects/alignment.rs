use super::super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_rect(
    center_x: f32,
    center_y: f32,
    size: f32,
) -> FrameRect {
    FrameRect {
        x: center_x - size * 0.5,
        y: center_y - size * 0.5,
        width: size,
        height: size,
    }
}

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
