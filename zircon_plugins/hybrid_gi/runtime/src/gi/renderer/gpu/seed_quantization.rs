pub(super) fn quantized_signed(value: f32) -> u32 {
    ((value * 64.0).round() as i32).wrapping_add(2048) as u32
}

pub(super) fn quantized_positive(value: f32, scale: f32) -> u32 {
    (value.max(0.0) * scale).round() as u32
}
