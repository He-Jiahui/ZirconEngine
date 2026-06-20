use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::PixelRect;
use super::blend::blend_pixel;
use super::clip::effective_clip;
use super::font::fallback_font;
use super::raster::rasterize_cached_glyph;

pub(in crate::ui::retained_host::host_contract) const DEFAULT_FONT_SIZE: f32 = 12.0;
pub(in crate::ui::retained_host::host_contract) const DEFAULT_LINE_HEIGHT: f32 = 14.0;

pub(in crate::ui::retained_host::host_contract) fn draw_text(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    draw_text_with_size(
        frame,
        rect,
        text,
        clip,
        color,
        DEFAULT_FONT_SIZE,
        DEFAULT_LINE_HEIGHT,
    );
}

pub(in crate::ui::retained_host::host_contract) fn draw_text_with_size(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
) {
    draw_text_with_size_and_style(
        frame,
        rect,
        text,
        clip,
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
    );
}

pub(in crate::ui::retained_host::host_contract) fn draw_text_with_size_and_style(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
    style: UiTextRunPaintStyle,
) {
    if text.trim().is_empty() || color[3] == 0 {
        return;
    }
    let Some(effective_clip) = effective_clip(frame, clip) else {
        return;
    };
    let Some(clip) = PixelRect::from_frame(
        &rect,
        effective_clip.as_ref(),
        frame.width(),
        frame.height(),
    ) else {
        return;
    };

    let max_text_height = rect.height.max(1.0);
    let font_size = font_size.max(1.0).min(max_text_height);
    let line_height = line_height.max(font_size).max(1.0).min(max_text_height);
    if frame.is_recording() {
        frame.record_text(
            FrameRect {
                x: clip.x0 as f32,
                y: clip.y0 as f32,
                width: clip.x1.saturating_sub(clip.x0) as f32,
                height: clip.y1.saturating_sub(clip.y0) as f32,
            },
            effective_clip,
            text,
            color,
            font_size,
            line_height,
            style,
        );
        if frame.record_only() {
            return;
        }
    }
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: rect.x,
        y: rect.y + ((rect.height - line_height).max(0.0) * 0.5),
        max_width: Some(rect.width.max(1.0)),
        max_height: Some(rect.height.max(1.0)),
        ..LayoutSettings::default()
    });
    layout.append(&[fallback_font()], &TextStyle::new(text, font_size, 0));

    for glyph in layout.glyphs() {
        let raster = rasterize_cached_glyph(glyph.key.glyph_index, glyph.key.px);
        let metrics = &raster.metrics;
        let bitmap = raster.bitmap.as_ref();
        if metrics.width == 0 || metrics.height == 0 {
            continue;
        }
        let glyph_x = glyph.x.round() as i32;
        let glyph_y = glyph.y.round() as i32;
        for row in 0..metrics.height {
            let y = glyph_y + row as i32;
            if y < clip.y0 as i32 || y >= clip.y1 as i32 {
                continue;
            }
            for column in 0..metrics.width {
                let coverage = bitmap[row * metrics.width + column];
                if coverage == 0 {
                    continue;
                }
                let italic_offset = italic_pixel_offset(style, row, metrics.height);
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
    }
}

fn italic_pixel_offset(style: UiTextRunPaintStyle, row: usize, height: usize) -> i32 {
    if !style.emphasis || height == 0 {
        return 0;
    }
    let top_bias = height.saturating_sub(row) as f32 / height.max(1) as f32;
    (top_bias * 2.0).round() as i32
}
