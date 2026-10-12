use zircon_runtime::core::framework::text::TextGlyphBitmapFormat;

use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::super::super::blend::{blend_pixel, blend_pixel_channel_coverage};

pub(super) fn draw_glyph_row(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    bitmap: &[u8],
    raster_width: usize,
    raster_height: usize,
    raster_format: TextGlyphBitmapFormat,
    row: usize,
    glyph_x: i32,
    y: i32,
    color: [u8; 4],
) {
    if row >= raster_height {
        return;
    }
    for column in 0..raster_width {
        let coverage = glyph_pixel(bitmap, raster_width, raster_format, column, row);
        if coverage.is_empty() {
            continue;
        }
        let x = glyph_x + column as i32;
        if x < clip.x0 as i32 || x >= clip.x1 as i32 {
            continue;
        }
        coverage.blend(frame, x as u32, y as u32, color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GlyphPixel {
    Empty,
    Alpha(u8),
    Subpixel([u8; 3]),
    Color([u8; 4]),
}

impl GlyphPixel {
    fn is_empty(self) -> bool {
        matches!(self, Self::Empty)
    }

    fn blend(self, frame: &mut HostRgbaFrame, x: u32, y: u32, color: [u8; 4]) {
        match self {
            Self::Empty => {}
            Self::Alpha(coverage) => {
                let mut pixel = color;
                pixel[3] = ((pixel[3] as u16 * coverage as u16) / 255) as u8;
                blend_pixel(frame, x, y, pixel);
            }
            Self::Subpixel(coverage) => {
                blend_pixel_channel_coverage(frame, x, y, color, coverage);
            }
            Self::Color(mut pixel) => {
                pixel[3] = ((pixel[3] as u16 * color[3] as u16) / 255) as u8;
                blend_pixel(frame, x, y, pixel);
            }
        }
    }
}

fn glyph_pixel(
    bitmap: &[u8],
    raster_width: usize,
    format: TextGlyphBitmapFormat,
    column: usize,
    row: usize,
) -> GlyphPixel {
    let pixel_index = row.saturating_mul(raster_width).saturating_add(column);
    match format {
        TextGlyphBitmapFormat::AlphaMask => bitmap
            .get(pixel_index)
            .copied()
            .filter(|coverage| *coverage != 0)
            .map(GlyphPixel::Alpha)
            .unwrap_or(GlyphPixel::Empty),
        TextGlyphBitmapFormat::SubpixelMask => rgba_pixel(bitmap, pixel_index)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .filter(|coverage| *coverage != [0, 0, 0])
            .map(GlyphPixel::Subpixel)
            .unwrap_or(GlyphPixel::Empty),
        TextGlyphBitmapFormat::ColorRgba => rgba_pixel(bitmap, pixel_index)
            .filter(|pixel| pixel[3] != 0)
            .map(GlyphPixel::Color)
            .unwrap_or(GlyphPixel::Empty),
    }
}

fn rgba_pixel(bitmap: &[u8], pixel_index: usize) -> Option<[u8; 4]> {
    let offset = pixel_index.checked_mul(4)?;
    let end = offset.checked_add(4)?;
    bitmap.get(offset..end)?.try_into().ok()
}

#[cfg(test)]
#[path = "row/tests/cases.rs"]
mod tests;
