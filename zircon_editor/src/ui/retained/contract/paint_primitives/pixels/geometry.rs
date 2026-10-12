use super::super::super::data::FrameRect;
use super::super::super::paint_geometry::is_visible_frame;

const COVERAGE_SAMPLE_AXIS: u32 = 8;
const PIXEL_HALF_DIAGONAL: f32 = std::f32::consts::FRAC_1_SQRT_2;

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
    rounded_rect_pixel_coverage(x, y, rect, corner_radius) >= 0.5
}

pub(in crate::ui::retained_host::host_contract) fn rounded_rect_pixel_coverage(
    x: u32,
    y: u32,
    rect: &FrameRect,
    corner_radius: f32,
) -> f32 {
    if !is_visible_frame(rect) {
        return 0.0;
    }
    let radius = clamped_corner_radius(rect, corner_radius);
    let center_distance =
        rounded_rect_signed_distance(x as f32 + 0.5, y as f32 + 0.5, rect, radius);
    if center_distance <= -PIXEL_HALF_DIAGONAL {
        return 1.0;
    }
    if center_distance >= PIXEL_HALF_DIAGONAL {
        return 0.0;
    }

    let mut covered = 0_u32;
    for sample_y in 0..COVERAGE_SAMPLE_AXIS {
        let py = y as f32 + (sample_y as f32 + 0.5) / COVERAGE_SAMPLE_AXIS as f32;
        for sample_x in 0..COVERAGE_SAMPLE_AXIS {
            let px = x as f32 + (sample_x as f32 + 0.5) / COVERAGE_SAMPLE_AXIS as f32;
            covered += u32::from(rounded_rect_signed_distance(px, py, rect, radius) <= 0.0);
        }
    }
    covered as f32 / (COVERAGE_SAMPLE_AXIS * COVERAGE_SAMPLE_AXIS) as f32
}

pub(in crate::ui::retained_host::host_contract) fn rect_pixel_coverage(
    x: u32,
    y: u32,
    rect: &FrameRect,
) -> f32 {
    if !is_visible_frame(rect) {
        return 0.0;
    }
    interval_pixel_coverage(x, rect.x, rect.x + rect.width)
        * interval_pixel_coverage(y, rect.y, rect.y + rect.height)
}

pub(super) fn interval_pixel_coverage(pixel: u32, start: f32, end: f32) -> f32 {
    let pixel_start = pixel as f32;
    (end.min(pixel_start + 1.0) - start.max(pixel_start)).clamp(0.0, 1.0)
}

fn rounded_rect_signed_distance(x: f32, y: f32, rect: &FrameRect, corner_radius: f32) -> f32 {
    let half_width = rect.width * 0.5;
    let half_height = rect.height * 0.5;
    let center_x = rect.x + half_width;
    let center_y = rect.y + half_height;
    let qx = (x - center_x).abs() - (half_width - corner_radius);
    let qy = (y - center_y).abs() - (half_height - corner_radius);
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    let inside = qx.max(qy).min(0.0);
    outside + inside - corner_radius
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
