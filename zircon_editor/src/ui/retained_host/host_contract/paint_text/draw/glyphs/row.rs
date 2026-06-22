use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::super::super::blend::blend_pixel;

pub(super) fn draw_glyph_row(
    frame: &mut HostRgbaFrame,
    clip: &PixelRect,
    bitmap: &[u8],
    glyph_width: usize,
    glyph_height: usize,
    row: usize,
    glyph_x: i32,
    y: i32,
    color: [u8; 4],
    style: UiTextRunPaintStyle,
) {
    for column in 0..glyph_width {
        let coverage = bitmap[row * glyph_width + column];
        if coverage == 0 {
            continue;
        }
        let italic_offset = italic_pixel_offset(style, row, glyph_height);
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

fn italic_pixel_offset(style: UiTextRunPaintStyle, row: usize, height: usize) -> i32 {
    if !style.emphasis || height == 0 {
        return 0;
    }
    let top_bias = height.saturating_sub(row) as f32 / height.max(1) as f32;
    (top_bias * 2.0).round() as i32
}
