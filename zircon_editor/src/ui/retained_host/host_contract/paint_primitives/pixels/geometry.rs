use super::super::super::data::FrameRect;
use super::super::super::paint_geometry::is_visible_frame;

pub(in crate::ui::retained_host::host_contract) fn clamped_corner_radius(
    rect: &FrameRect,
    corner_radius: f32,
) -> f32 {
    if !corner_radius.is_finite() {
        return 0.0;
    }
    corner_radius
        .max(0.0)
        .min(rect.width.min(rect.height).max(0.0) * 0.5)
}

pub(in crate::ui::retained_host::host_contract) fn rounded_rect_contains_pixel(
    x: u32,
    y: u32,
    rect: &FrameRect,
    corner_radius: f32,
) -> bool {
    if !is_visible_frame(rect) {
        return false;
    }
    let px = x as f32 + 0.5;
    let py = y as f32 + 0.5;
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    if px < left || px >= right || py < top || py >= bottom {
        return false;
    }
    let radius = clamped_corner_radius(rect, corner_radius);
    if radius <= 0.0 {
        return true;
    }
    let center_x = clamp_to_ordered_range(px, left + radius, right - radius);
    let center_y = clamp_to_ordered_range(py, top + radius, bottom - radius);
    let dx = px - center_x;
    let dy = py - center_y;
    dx * dx + dy * dy <= radius * radius
}

pub(in crate::ui::retained_host::host_contract) fn clamp_to_ordered_range(
    value: f32,
    min: f32,
    max: f32,
) -> f32 {
    if min <= max {
        value.clamp(min, max)
    } else {
        (min + max) * 0.5
    }
}

pub(in crate::ui::retained_host::host_contract) fn inset_frame(
    rect: &FrameRect,
    amount: f32,
) -> FrameRect {
    FrameRect {
        x: rect.x + amount,
        y: rect.y + amount,
        width: (rect.width - amount * 2.0).max(0.0),
        height: (rect.height - amount * 2.0).max(0.0),
    }
}
