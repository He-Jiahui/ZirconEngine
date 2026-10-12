use std::sync::OnceLock;

pub(in crate::ui::retained_host::host_contract) const LINEAR_ENCODE_LUT_MAX: usize = 4_096;

#[inline]
pub(in crate::ui::retained_host::host_contract) fn blend_srgb_pixel_linear(
    pixel: &mut [u8],
    color: [u8; 4],
    coverage: f32,
) {
    let source_alpha = f32::from(color[3]) / 255.0 * coverage.clamp(0.0, 1.0);
    if !source_alpha.is_finite() || source_alpha <= 0.0 {
        return;
    }
    if source_alpha >= 1.0 {
        pixel[..4].copy_from_slice(&color);
        return;
    }

    let destination_alpha = f32::from(pixel[3]) / 255.0;
    let output_alpha = source_alpha + destination_alpha * (1.0 - source_alpha);
    if output_alpha <= f32::EPSILON {
        return;
    }
    let inverse = 1.0 - source_alpha;
    for channel in 0..3 {
        let source = srgb_byte_to_linear(color[channel]);
        let destination = srgb_byte_to_linear(pixel[channel]);
        let output_premultiplied =
            source * source_alpha + destination * destination_alpha * inverse;
        pixel[channel] = linear_to_srgb_byte(output_premultiplied / output_alpha);
    }
    pixel[3] = (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}

#[inline]
pub(in crate::ui::retained_host::host_contract) fn blend_srgb_pixel_linear_channels(
    pixel: &mut [u8],
    color: [u8; 4],
    coverage: [u8; 3],
) {
    let color_alpha = f32::from(color[3]) / 255.0;
    if color_alpha <= 0.0 || coverage == [0, 0, 0] {
        return;
    }

    for channel in 0..3 {
        let source_alpha = color_alpha * f32::from(coverage[channel]) / 255.0;
        if source_alpha <= 0.0 {
            continue;
        }
        if source_alpha >= 1.0 {
            pixel[channel] = color[channel];
            continue;
        }
        let source = srgb_byte_to_linear(color[channel]);
        let destination = srgb_byte_to_linear(pixel[channel]);
        pixel[channel] =
            linear_to_srgb_byte(source * source_alpha + destination * (1.0 - source_alpha));
    }
    pixel[3] = 255;
}

#[inline]
pub(in crate::ui::retained_host::host_contract) fn blend_premultiplied_linear_srgb_pixel(
    pixel: &mut [u8],
    premultiplied_linear: [f32; 3],
    alpha: f32,
) {
    if !alpha.is_finite() || alpha <= 0.0 {
        return;
    }
    let source_alpha = alpha.clamp(0.0, 1.0);
    let destination_alpha = f32::from(pixel[3]) / 255.0;
    let inverse = 1.0 - source_alpha;
    let output_alpha = source_alpha + destination_alpha * inverse;
    if output_alpha <= f32::EPSILON {
        return;
    }
    for channel in 0..3 {
        let destination = srgb_byte_to_linear(pixel[channel]);
        let output_premultiplied =
            premultiplied_linear[channel] + destination * destination_alpha * inverse;
        pixel[channel] = linear_to_srgb_byte(output_premultiplied / output_alpha);
    }
    pixel[3] = (output_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}

#[inline]
pub(in crate::ui::retained_host::host_contract) fn srgb_byte_to_linear(value: u8) -> f32 {
    static DECODED_SRGB: OnceLock<[f32; 256]> = OnceLock::new();
    DECODED_SRGB.get_or_init(|| std::array::from_fn(|value| decode_srgb(value as f32 / 255.0)))
        [value as usize]
}

#[inline]
pub(in crate::ui::retained_host::host_contract) fn linear_to_srgb_byte(value: f32) -> u8 {
    static ENCODED_SRGB: OnceLock<[u8; LINEAR_ENCODE_LUT_MAX + 1]> = OnceLock::new();
    let index = (value.clamp(0.0, 1.0) * LINEAR_ENCODE_LUT_MAX as f32).round() as usize;
    ENCODED_SRGB.get_or_init(|| {
        std::array::from_fn(|index| {
            let linear = index as f32 / LINEAR_ENCODE_LUT_MAX as f32;
            (encode_linear_srgb(linear) * 255.0).round() as u8
        })
    })[index]
}

fn decode_srgb(value: f32) -> f32 {
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn encode_linear_srgb(value: f32) -> f32 {
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}
