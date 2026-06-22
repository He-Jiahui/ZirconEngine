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
