use super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const STATUS_ITEM_ICON_SIZE: f32 =
    14.0;
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn warning_mark_segments(
    rect: &FrameRect,
    mark_width: f32,
) -> [FrameRect; 2] {
    let mark_width = normalized_status_mark_width(mark_width);
    let x = 7.0 - mark_width * 0.5;
    [
        local_rect_scaled(rect, x, 6.0, mark_width, 4.0, STATUS_ITEM_ICON_SIZE),
        local_rect_scaled(rect, x, 11.0, mark_width, mark_width, STATUS_ITEM_ICON_SIZE),
    ]
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn local_rect_scaled(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    base_size: f32,
) -> FrameRect {
    let scale_x = origin.width / base_size;
    let scale_y = origin.height / base_size;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: width * scale_x,
        height: height * scale_y,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn normalized_status_mark_width(
    width: f32,
) -> f32 {
    if width.is_finite() && width > 0.0 {
        width
    } else {
        2.0
    }
}
