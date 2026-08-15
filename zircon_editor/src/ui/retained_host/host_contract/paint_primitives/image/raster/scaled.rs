use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::pixel::write_bilinear_rgba_pixel;

pub(in crate::ui::retained_host::host_contract) fn draw_scaled_rgba_image_pixels(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    target: &PixelRect,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) {
    let rect_width = rect.width.max(1.0);
    let rect_height = rect.height.max(1.0);
    let frame_width = frame.width() as usize;
    let bytes = frame.as_bytes_mut();

    for y in target.y0..target.y1 {
        let source_y = source_sample_coordinate(y, rect.y, rect_height, image_height);
        let source_y0 = source_y.floor() as u32;
        let source_y1 = source_y0.saturating_add(1).min(image_height - 1);
        let y_mix = source_y - source_y0 as f32;
        let destination_row = y as usize * frame_width * 4;
        for x in target.x0..target.x1 {
            let source_x = source_sample_coordinate(x, rect.x, rect_width, image_width);
            let source_x0 = source_x.floor() as u32;
            let source_x1 = source_x0.saturating_add(1).min(image_width - 1);
            let x_mix = source_x - source_x0 as f32;
            let destination_offset = destination_row + x as usize * 4;
            write_bilinear_rgba_pixel(
                bytes,
                destination_offset,
                rgba,
                image_width,
                [source_x0, source_x1],
                [source_y0, source_y1],
                [x_mix, y_mix],
            );
        }
    }
}

fn source_sample_coordinate(
    destination: u32,
    destination_origin: f32,
    destination_extent: f32,
    source_extent: u32,
) -> f32 {
    ((((destination as f32 + 0.5 - destination_origin) / destination_extent)
        * source_extent as f32)
        - 0.5)
        .clamp(0.0, source_extent.saturating_sub(1) as f32)
}
