use std::f32::consts::TAU;

pub(super) fn clamp_pixel_range(min: f32, max: f32, extent: u32) -> std::ops::Range<u32> {
    let start = min.floor().max(0.0).min(extent as f32) as u32;
    let end = max.ceil().max(0.0).min(extent as f32) as u32;
    start..end
}

pub(super) fn distance_to_segment(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let segment = (end.0 - start.0, end.1 - start.1);
    let length_sq = segment.0 * segment.0 + segment.1 * segment.1;
    if length_sq <= f32::EPSILON {
        let dx = point.0 - start.0;
        let dy = point.1 - start.1;
        return (dx * dx + dy * dy).sqrt();
    }
    let t = (((point.0 - start.0) * segment.0 + (point.1 - start.1) * segment.1) / length_sq)
        .clamp(0.0, 1.0);
    let projection = (start.0 + segment.0 * t, start.1 + segment.1 * t);
    let dx = point.0 - projection.0;
    let dy = point.1 - projection.1;
    (dx * dx + dy * dy).sqrt()
}

pub(super) fn normalized_angle(angle: f32) -> f32 {
    angle.rem_euclid(TAU)
}
