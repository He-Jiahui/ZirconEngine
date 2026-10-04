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
    let source_x_samples = (target.x0..target.x1)
        .map(|x| source_axis_sample(x, rect.x, rect_width, image_width))
        .collect::<Vec<_>>();

    for y in target.y0..target.y1 {
        let source_y = source_axis_sample(y, rect.y, rect_height, image_height);
        let mut destination_offset = (y as usize * frame_width + target.x0 as usize) * 4;
        for sample in &source_x_samples {
            write_bilinear_rgba_pixel(
                bytes,
                destination_offset,
                rgba,
                image_width,
                [sample.lower, sample.upper],
                [source_y.lower, source_y.upper],
                [sample.mix, source_y.mix],
            );
            destination_offset += 4;
        }
    }
}

#[derive(Clone, Copy)]
struct SourceAxisSample {
    lower: u32,
    upper: u32,
    mix: f32,
}

fn source_axis_sample(
    destination: u32,
    destination_origin: f32,
    destination_extent: f32,
    source_extent: u32,
) -> SourceAxisSample {
    let coordinate = source_sample_coordinate(
        destination,
        destination_origin,
        destination_extent,
        source_extent,
    );
    let lower = coordinate.floor() as u32;
    SourceAxisSample {
        lower,
        upper: lower.saturating_add(1).min(source_extent - 1),
        mix: coordinate - lower as f32,
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

#[cfg(test)]
#[path = "tests/scaled.rs"]
mod tests;
