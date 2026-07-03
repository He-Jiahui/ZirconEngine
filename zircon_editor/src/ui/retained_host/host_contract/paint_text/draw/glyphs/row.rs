use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::super::super::blend::{blend_pixel, blend_pixel_channel_coverage};
use super::super::super::raster::CachedGlyphRasterFormat;

const THIN_STROKE_HIGH_SAMPLE: u8 = 220;
const THIN_STROKE_MAX_AVERAGE: u8 = 96;
const THIN_STROKE_MIN_COVERAGE: u8 = 128;

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
    if raster_width == 0 || raster_height == 0 || !raster_scale.is_finite() || raster_scale <= 1.0 {
        return bitmap
            .get(logical_row * raster_width + logical_column)
            .copied()
            .unwrap_or(0);
    }

    let sample_offset_x = normalized_sample_offset(sample_offset_x);
    let x0 = (((logical_column as f32) - sample_offset_x) * raster_scale).floor() as isize;
    let x1 = ((((logical_column + 1) as f32) - sample_offset_x) * raster_scale).ceil() as isize;
    let y0 = ((logical_row as f32) * raster_scale).floor() as usize;
    let y1 = (((logical_row + 1) as f32) * raster_scale).ceil() as usize;

    let mut sum = 0_u32;
    let mut count = 0_u32;
    let mut max_coverage = 0_u8;
    let x_start = x0.max(0) as usize;
    let x_end = x1.max(0).min(raster_width as isize) as usize;
    for sample_y in y0..y1.min(raster_height) {
        let row_start = sample_y * raster_width;
        for sample_x in x_start..x_end {
            let coverage = bitmap[row_start + sample_x];
            sum += coverage as u32;
            max_coverage = max_coverage.max(coverage);
            count += 1;
        }
    }

    if count == 0 {
        0
    } else {
        let average = ((sum as f32 / count as f32).round()).min(255.0) as u8;
        thin_stroke_preserved_coverage(average, max_coverage)
    }
}

fn sampled_subpixel_coverage(
    bitmap: &[u8],
    raster_width: usize,
    raster_height: usize,
    logical_column: usize,
    logical_row: usize,
    raster_scale: f32,
    _sample_offset_x: f32,
) -> [u8; 3] {
    if raster_width == 0 || raster_height == 0 || !raster_scale.is_finite() || raster_scale <= 1.0 {
        let offset = (logical_row * raster_width + logical_column) * 4;
        return bitmap
            .get(offset..offset + 3)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .unwrap_or([0, 0, 0]);
    }

    let x0 = ((logical_column as f32) * raster_scale).floor() as usize;
    let x1 = (((logical_column + 1) as f32) * raster_scale).ceil() as usize;
    let y0 = ((logical_row as f32) * raster_scale).floor() as usize;
    let y1 = (((logical_row + 1) as f32) * raster_scale).ceil() as usize;

    let mut sums = [0_u32; 3];
    let mut maxes = [0_u8; 3];
    let mut count = 0_u32;
    for sample_y in y0..y1.min(raster_height) {
        let row_start = sample_y * raster_width * 4;
        for sample_x in x0..x1.min(raster_width) {
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

fn normalized_sample_offset(offset: f32) -> f32 {
    if offset.is_finite() {
        offset.clamp(0.0, 0.999)
    } else {
        0.0
    }
}

fn averaged_channel_coverage(sum: u32, count: u32, max_coverage: u8) -> u8 {
    let average = ((sum as f32 / count as f32).round()).min(255.0) as u8;
    thin_stroke_preserved_coverage(average, max_coverage)
}

fn thin_stroke_preserved_coverage(average: u8, max_coverage: u8) -> u8 {
    if average > 0 && average < THIN_STROKE_MAX_AVERAGE && max_coverage >= THIN_STROKE_HIGH_SAMPLE {
        THIN_STROKE_MIN_COVERAGE.max(average)
    } else {
        average
    }
}

fn italic_pixel_offset(style: UiTextRunPaintStyle, row: usize, height: usize) -> i32 {
    if !style.emphasis || height == 0 {
        return 0;
    }
    let top_bias = height.saturating_sub(row) as f32 / height.max(1) as f32;
    (top_bias * 2.0).round() as i32
}

fn glyph_draw_pass_count(_style: UiTextRunPaintStyle) -> i32 {
    1
}

#[cfg(test)]
mod tests;
