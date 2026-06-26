use super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const STATUS_ICON_GLYPH_SIZE: f32 =
    16.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn centered_rect(
    rect: &FrameRect,
    size: f32,
) -> FrameRect {
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size.min(rect.width.max(1.0)).max(1.0),
        height: size.min(rect.height.max(1.0)).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn local_rect(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
}
