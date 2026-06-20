use super::super::paint_frame::HostRgbaFrame;

pub(in crate::ui::retained_host::host_contract) fn blend_pixel(
    frame: &mut HostRgbaFrame,
    x: u32,
    y: u32,
    color: [u8; 4],
) {
    if color[3] == 0 {
        return;
    }
    let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
    let bytes = frame.as_bytes_mut();
    if color[3] == 255 {
        write_pixel_channels(&mut bytes[offset..offset + 4], color);
        return;
    }

    let alpha = color[3] as u32;
    let inverse = 255 - alpha;
    for channel in 0..3 {
        let source = color[channel] as u32;
        let destination = bytes[offset + channel] as u32;
        bytes[offset + channel] = ((source * alpha + destination * inverse) / 255) as u8;
    }
    bytes[offset + 3] = 255;
}

#[inline]
fn write_pixel_channels(pixel: &mut [u8], color: [u8; 4]) {
    pixel[0] = color[0];
    pixel[1] = color[1];
    pixel[2] = color[2];
    pixel[3] = color[3];
}
