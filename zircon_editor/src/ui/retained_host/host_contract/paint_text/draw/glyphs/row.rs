use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::super::super::blend::blend_pixel;

pub(super) fn draw_glyph_row(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    bitmap: &[u8],
    raster_width: usize,
    raster_height: usize,
    logical_width: usize,
    logical_height: usize,
    row: usize,
    glyph_x: i32,
    y: i32,
    color: [u8; 4],
    style: UiTextRunPaintStyle,
    raster_scale: f32,
) {
    for column in 0..logical_width {
        let coverage = sampled_coverage(
            bitmap,
            raster_width,
            raster_height,
            column,
            row,
            raster_scale,
        );
        if coverage == 0 {
            continue;
        }
        let italic_offset = italic_pixel_offset(style, row, logical_height);
        let draw_count = if style.strong { 2 } else { 1 };
        for pass in 0..draw_count {
            let x = glyph_x + column as i32 + italic_offset + pass;
            if x < clip.x0 as i32 || x >= clip.x1 as i32 {
                continue;
            }
            let mut pixel = color;
            pixel[3] = ((pixel[3] as u16 * coverage as u16) / 255) as u8;
            blend_pixel(frame, x as u32, y as u32, pixel);
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
) -> u8 {
    if raster_width == 0 || raster_height == 0 || !raster_scale.is_finite() || raster_scale <= 1.0 {
        return bitmap
            .get(logical_row * raster_width + logical_column)
            .copied()
            .unwrap_or(0);
    }

    let x0 = ((logical_column as f32) * raster_scale).floor() as usize;
    let x1 = (((logical_column + 1) as f32) * raster_scale).ceil() as usize;
    let y0 = ((logical_row as f32) * raster_scale).floor() as usize;
    let y1 = (((logical_row + 1) as f32) * raster_scale).ceil() as usize;

    let mut sum = 0_u32;
    let mut count = 0_u32;
    for sample_y in y0..y1.min(raster_height) {
        let row_start = sample_y * raster_width;
        for sample_x in x0..x1.min(raster_width) {
            sum += bitmap[row_start + sample_x] as u32;
            count += 1;
        }
    }

    if count == 0 {
        0
    } else {
        (sum / count) as u8
    }
}

fn italic_pixel_offset(style: UiTextRunPaintStyle, row: usize, height: usize) -> i32 {
    if !style.emphasis || height == 0 {
        return 0;
    }
    let top_bias = height.saturating_sub(row) as f32 / height.max(1) as f32;
    (top_bias * 2.0).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampled_coverage_averages_supersampled_pixels() {
        let bitmap = [0, 255, 255, 0];

        assert_eq!(sampled_coverage(&bitmap, 2, 2, 0, 0, 2.0), 127);
    }

    #[test]
    fn sampled_coverage_clamps_to_bitmap_edge() {
        let bitmap = [64, 128, 255];

        assert_eq!(sampled_coverage(&bitmap, 3, 1, 1, 0, 2.0), 255);
    }
}
