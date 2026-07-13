/// Median channel selection used by the MSDF reference implementation and WGSL.
pub(crate) fn median3(red: f32, green: f32, blue: f32) -> f32 {
    red.max(green.min(blue)).min(green.max(blue))
}

pub(crate) fn msdf_sample_distance(sample: [f32; 4]) -> f32 {
    median3(sample[0], sample[1], sample[2])
}

pub(crate) fn mtsdf_sample_true_distance(sample: [f32; 4]) -> f32 {
    sample[3]
}

/// CPU reference for the shader coverage equation; derivative is in sample units per pixel.
pub(crate) fn distance_field_coverage(
    distance: f32,
    screen_px_range: f32,
    derivative_width: f32,
) -> f32 {
    let px_range = screen_px_range.max(1.0);
    let signed_distance = (distance - 0.5) * px_range;
    let aa_width = (derivative_width.abs() * px_range).max(1.0);
    (signed_distance / aa_width + 0.5).clamp(0.0, 1.0)
}
