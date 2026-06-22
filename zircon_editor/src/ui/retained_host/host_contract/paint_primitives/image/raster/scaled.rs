use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::pixel::write_rgba_pixel;

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
        let source_y = (((y as f32 + 0.5 - rect.y) / rect_height) * image_height as f32)
            .floor()
            .max(0.0)
            .min((image_height - 1) as f32) as u32;
        let destination_row = y as usize * frame_width * 4;
        for x in target.x0..target.x1 {
            let source_x = (((x as f32 + 0.5 - rect.x) / rect_width) * image_width as f32)
                .floor()
                .max(0.0)
                .min((image_width - 1) as f32) as u32;
            let source_offset =
                ((source_y as usize * image_width as usize) + source_x as usize) * 4;
            let destination_offset = destination_row + x as usize * 4;
            write_rgba_pixel(bytes, destination_offset, rgba, source_offset);
        }
    }
}
