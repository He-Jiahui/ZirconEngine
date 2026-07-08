use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::super::super::blend::{blend_pixel, blend_pixel_channel_coverage};
use super::super::super::raster::CachedGlyphRasterFormat;
use super::metrics::{
    averaged_channel_coverage, glyph_draw_pass_count, italic_pixel_offset, raster_sample_x_range,
    raster_sample_y_range, uses_native_pixel_sampling,
};

pub(super) fn draw_glyph_row(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    bitmap: &[u8],
    raster_width: usize,
    raster_height: usize,
    raster_format: CachedGlyphRasterFormat,
    logical_width: usize,
    logical_height: usize,
    row: usize,
    glyph_x: i32,
    y: i32,
    color: [u8; 4],
    style: UiTextRunPaintStyle,
    raster_scale: f32,
    sample_offset_x: f32,
) {
    for column in 0..logical_width {
        let draw_pixel = sampled_pixel_coverage(
            bitmap,
            raster_width,
            raster_height,
            raster_format,
            column,
            row,
            raster_scale,
            sample_offset_x,
        );
        if draw_pixel.is_empty() {
            continue;
        }
        let italic_offset = italic_pixel_offset(style, row, logical_height);
        for pass in 0..glyph_draw_pass_count(style) {
            let x = glyph_x + column as i32 + italic_offset + pass;
            if x < clip.x0 as i32 || x >= clip.x1 as i32 {
                continue;
            }
            draw_pixel.blend(frame, x as u32, y as u32, color);
        }
    }
}

#[derive(Clone, Copy)]
enum SampledPixelCoverage {
    Empty,
    Alpha(u8),
    Subpixel([u8; 3]),
}

impl SampledPixelCoverage {
    fn is_empty(self) -> bool {
        matches!(self, SampledPixelCoverage::Empty)
    }

    fn blend(self, frame: &mut HostRgbaFrame, x: u32, y: u32, color: [u8; 4]) {
        match self {
            SampledPixelCoverage::Empty => {}
            SampledPixelCoverage::Alpha(coverage) => {
                let mut pixel = color;
                pixel[3] = ((pixel[3] as u16 * coverage as u16) / 255) as u8;
                blend_pixel(frame, x, y, pixel);
            }
            SampledPixelCoverage::Subpixel(coverage) => {
                blend_pixel_channel_coverage(frame, x, y, color, coverage);
            }
        }
    }
}

fn sampled_pixel_coverage(
    bitmap: &[u8],
    raster_width: usize,
    raster_height: usize,
    raster_format: CachedGlyphRasterFormat,
    logical_column: usize,
    logical_row: usize,
    raster_scale: f32,
    sample_offset_x: f32,
) -> SampledPixelCoverage {
    match raster_format {
        CachedGlyphRasterFormat::AlphaMask => {
            let coverage = sampled_coverage(
                bitmap,
                raster_width,
                raster_height,
                logical_column,
                logical_row,
                raster_scale,
                sample_offset_x,
            );
            if coverage == 0 {
                SampledPixelCoverage::Empty
            } else {
                SampledPixelCoverage::Alpha(coverage)
            }
        }
        CachedGlyphRasterFormat::SubpixelMask => {
            let coverage = sampled_subpixel_coverage(
                bitmap,
                raster_width,
                raster_height,
                logical_column,
                logical_row,
                raster_scale,
                sample_offset_x,
            );
            if coverage == [0, 0, 0] {
                SampledPixelCoverage::Empty
            } else {
                SampledPixelCoverage::Subpixel(coverage)
            }
        }
    }
}

fn sampled_coverage(
    bitmap: &[u8],
    raster_width: usize,
    raster_height: usize,
    logical_column: usize,
    logical_row: usize,
    raster_scale: f32,
    sample_offset_x: f32,
) -> u8 {
    if uses_native_pixel_sampling(raster_width, raster_height, raster_scale) {
        return bitmap
            .get(logical_row * raster_width + logical_column)
            .copied()
            .unwrap_or(0);
    }

    let mut sum = 0_u32;
    let mut count = 0_u32;
    let mut max_coverage = 0_u8;
    let sample_x_range =
        raster_sample_x_range(logical_column, raster_scale, sample_offset_x, raster_width);
    for sample_y in raster_sample_y_range(logical_row, raster_scale, raster_height) {
        let row_start = sample_y * raster_width;
        for sample_x in sample_x_range.clone() {
            let coverage = bitmap[row_start + sample_x];
            sum += coverage as u32;
            max_coverage = max_coverage.max(coverage);
            count += 1;
        }
    }

    if count == 0 {
        0
    } else {
        averaged_channel_coverage(sum, count, max_coverage)
    }
}

fn sampled_subpixel_coverage(
    bitmap: &[u8],
    raster_width: usize,
    raster_height: usize,
    logical_column: usize,
    logical_row: usize,
    raster_scale: f32,
    sample_offset_x: f32,
) -> [u8; 3] {
    if uses_native_pixel_sampling(raster_width, raster_height, raster_scale) {
        let offset = (logical_row * raster_width + logical_column) * 4;
        return bitmap
            .get(offset..offset + 3)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .unwrap_or([0, 0, 0]);
    }

    let mut sums = [0_u32; 3];
    let mut maxes = [0_u8; 3];
    let mut count = 0_u32;
    let sample_x_range =
        raster_sample_x_range(logical_column, raster_scale, sample_offset_x, raster_width);
    for sample_y in raster_sample_y_range(logical_row, raster_scale, raster_height) {
        let row_start = sample_y * raster_width * 4;
        for sample_x in sample_x_range.clone() {
            let offset = row_start + sample_x * 4;
            for channel in 0..3 {
                let coverage = bitmap[offset + channel];
                sums[channel] += coverage as u32;
                maxes[channel] = maxes[channel].max(coverage);
            }
            count += 1;
        }
    }

    if count == 0 {
        [0, 0, 0]
    } else {
        [
            averaged_channel_coverage(sums[0], count, maxes[0]),
            averaged_channel_coverage(sums[1], count, maxes[1]),
            averaged_channel_coverage(sums[2], count, maxes[2]),
        ]
    }
}

#[cfg(test)]
mod tests;
