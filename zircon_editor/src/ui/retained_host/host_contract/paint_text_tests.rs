use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::draw::{draw_text_with_size_and_style, DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT};
use super::raster::rasterize_cached_glyph;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;

#[test]
fn glyph_raster_cache_reuses_bitmap_for_same_glyph_and_size() {
    let first = rasterize_cached_glyph(1, DEFAULT_FONT_SIZE);
    let second = rasterize_cached_glyph(1, DEFAULT_FONT_SIZE);

    assert_eq!(first.metrics.width, second.metrics.width);
    assert!(Arc::ptr_eq(&first.bitmap, &second.bitmap));
}

#[test]
fn text_draw_skips_disjoint_active_and_explicit_clips() {
    let mut frame = HostRgbaFrame::filled(64, 32, [0, 0, 0, 255]);
    frame.replace_paint_clip(Some(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 8.0,
        height: 8.0,
    }));

    draw_text_with_size_and_style(
        &mut frame,
        FrameRect {
            x: 16.0,
            y: 16.0,
            width: 40.0,
            height: 12.0,
        },
        "Ready",
        Some(&FrameRect {
            x: 16.0,
            y: 16.0,
            width: 40.0,
            height: 12.0,
        }),
        [255, 255, 255, 255],
        DEFAULT_FONT_SIZE,
        DEFAULT_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
    );

    assert!(frame
        .as_bytes()
        .chunks_exact(4)
        .all(|pixel| pixel == [0, 0, 0, 255]));
}
