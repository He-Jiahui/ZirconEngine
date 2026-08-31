use crate::ui::retained_host::host_contract::paint_color::{
    blend_premultiplied_linear_srgb_pixel, blend_srgb_pixel_linear, srgb_byte_to_linear,
};

#[inline]
pub(super) fn write_rgba_pixel(
    bytes: &mut [u8],
    destination_offset: usize,
    rgba: &[u8],
    source_offset: usize,
) {
    let color = [
        rgba[source_offset],
        rgba[source_offset + 1],
        rgba[source_offset + 2],
        rgba[source_offset + 3],
    ];
    blend_srgb_pixel_linear(
        &mut bytes[destination_offset..destination_offset + 4],
        color,
        1.0,
    );
}

pub(super) fn write_bilinear_rgba_pixel(
    bytes: &mut [u8],
    destination_offset: usize,
    rgba: &[u8],
    image_width: u32,
    source_x: [u32; 2],
    source_y: [u32; 2],
    mix: [f32; 2],
) {
    let weights = [
        (1.0 - mix[0]) * (1.0 - mix[1]),
        mix[0] * (1.0 - mix[1]),
        (1.0 - mix[0]) * mix[1],
        mix[0] * mix[1],
    ];
    let offsets = [
        source_offset(image_width, source_x[0], source_y[0]),
        source_offset(image_width, source_x[1], source_y[0]),
        source_offset(image_width, source_x[0], source_y[1]),
        source_offset(image_width, source_x[1], source_y[1]),
    ];
    let mut alpha = 0.0_f32;
    let mut premultiplied_linear = [0.0_f32; 3];
    for (weight, source_offset) in weights.into_iter().zip(offsets) {
        let sample_alpha = f32::from(rgba[source_offset + 3]) / 255.0;
        alpha += sample_alpha * weight;
        for channel in 0..3 {
            premultiplied_linear[channel] +=
                srgb_byte_to_linear(rgba[source_offset + channel]) * sample_alpha * weight;
        }
    }
    blend_premultiplied_linear_srgb_pixel(
        &mut bytes[destination_offset..destination_offset + 4],
        premultiplied_linear,
        alpha,
    );
}

fn source_offset(image_width: u32, x: u32, y: u32) -> usize {
    ((y as usize * image_width as usize) + x as usize) * 4
}
