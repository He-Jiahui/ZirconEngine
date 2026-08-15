#[inline]
pub(super) fn write_rgba_pixel(
    bytes: &mut [u8],
    destination_offset: usize,
    rgba: &[u8],
    source_offset: usize,
) {
    let alpha = rgba[source_offset + 3];
    if alpha == 0 {
        return;
    }
    if alpha == 255 {
        bytes[destination_offset] = rgba[source_offset];
        bytes[destination_offset + 1] = rgba[source_offset + 1];
        bytes[destination_offset + 2] = rgba[source_offset + 2];
        bytes[destination_offset + 3] = 255;
        return;
    }

    let alpha = alpha as u32;
    let inverse = 255 - alpha;
    for channel in 0..3 {
        let source = rgba[source_offset + channel] as u32;
        let destination = bytes[destination_offset + channel] as u32;
        bytes[destination_offset + channel] =
            ((source * alpha + destination * inverse) / 255) as u8;
    }
    bytes[destination_offset + 3] = 255;
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
    let mut premultiplied = [0.0_f32; 3];
    for (weight, source_offset) in weights.into_iter().zip(offsets) {
        let sample_alpha = rgba[source_offset + 3] as f32;
        alpha += sample_alpha * weight;
        for channel in 0..3 {
            premultiplied[channel] +=
                rgba[source_offset + channel] as f32 * (sample_alpha / 255.0) * weight;
        }
    }
    blend_premultiplied_pixel(bytes, destination_offset, premultiplied, alpha);
}

fn source_offset(image_width: u32, x: u32, y: u32) -> usize {
    ((y as usize * image_width as usize) + x as usize) * 4
}

fn blend_premultiplied_pixel(
    bytes: &mut [u8],
    destination_offset: usize,
    premultiplied: [f32; 3],
    alpha: f32,
) {
    if alpha <= 0.0 {
        return;
    }
    let inverse = 1.0 - (alpha / 255.0).clamp(0.0, 1.0);
    for channel in 0..3 {
        let destination = bytes[destination_offset + channel] as f32;
        bytes[destination_offset + channel] = (premultiplied[channel] + destination * inverse)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    bytes[destination_offset + 3] = 255;
}
