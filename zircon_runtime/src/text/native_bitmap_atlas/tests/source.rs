use super::*;
use crate::text::InstancedFaceId;
use crate::text::atlas::{
    GlyphHintingMode, GlyphRasterKey, GlyphRasterRequest, GlyphSmoothingMode, SyntheticGlyphStyle,
};
use std::sync::Arc;

#[test]
fn native_bitmap_atlas_source_uses_prepared_glyph_baseline() {
    let image = test_image(GlyphAtlasFormat::AlphaMask, 9, 13, 117);
    let screen_rect = screen_rect_for(image);
    let raster_key = test_raster_key(GlyphAtlasFormat::AlphaMask);
    let source_bytes = Arc::<[u8]>::from(vec![7; 117]);
    let clipped_source = native_bitmap_atlas_source_from_image(
        image,
        screen_rect,
        Arc::clone(&source_bytes),
        Some(raster_key),
    )
    .expect("alpha glyph image should produce an atlas source");
    let source = clipped_source.source;

    assert_eq!(source.format, GlyphAtlasFormat::AlphaMask);
    assert_eq!(source.content_size, UVec2::new(9, 13));
    assert_eq!(
        source.screen_rect,
        GlyphAtlasScreenRect::new(20.0, 11.0, 9.0, 13.0)
    );
    assert_eq!(source.source_byte_len, 117);
    assert_eq!(source.raster_key, Some(raster_key));
    assert_eq!(source.foreground_color, [0.9, 0.8, 0.7, 1.0]);
    assert_eq!(clipped_source.bytes.len(), 117);
    assert!(Arc::ptr_eq(&source_bytes, &clipped_source.bytes));
    assert!(!clipped_source.was_clipped);
}

#[test]
fn native_bitmap_atlas_source_preserves_known_subpixel_background_color() {
    let mut image = test_image(GlyphAtlasFormat::SubpixelMask, 8, 8, 256);
    image.background_color = [0.1, 0.2, 0.3, 1.0];
    let screen_rect = screen_rect_for(image);
    let clipped_source =
        native_bitmap_atlas_source_from_image(image, screen_rect, Arc::from(vec![7; 256]), None)
            .expect("subpixel glyph image should preserve known background input");

    assert_eq!(clipped_source.source.format, GlyphAtlasFormat::SubpixelMask);
    assert_eq!(clipped_source.source.background_color, [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(clipped_source.bytes.len(), 256);
}

#[test]
fn native_bitmap_atlas_source_crops_alpha_rows_to_run_bounds() {
    let image = test_image(GlyphAtlasFormat::AlphaMask, 9, 4, 36);
    let source_bytes = Arc::<[u8]>::from((0..36).collect::<Vec<u8>>());
    let clipped = GlyphAtlasScreenRect::new(20.0, 11.0, 5.0, 2.0);

    let clipped_source = native_bitmap_atlas_source_from_image(
        image,
        clipped,
        source_bytes,
        Some(test_raster_key(GlyphAtlasFormat::AlphaMask)),
    )
    .expect("run-bounds clipped alpha glyph should still produce an atlas source");

    assert_eq!(clipped_source.source.content_size, UVec2::new(5, 2));
    assert_eq!(
        clipped_source.source.screen_rect,
        GlyphAtlasScreenRect::new(20.0, 11.0, 5.0, 2.0)
    );
    assert_eq!(clipped_source.source.source_byte_len, 10);
    assert_eq!(clipped_source.source.raster_key, None);
    assert_eq!(
        clipped_source.bytes.as_ref(),
        &[0, 1, 2, 3, 4, 9, 10, 11, 12, 13]
    );
    assert!(clipped_source.was_clipped);
}

#[test]
fn native_bitmap_atlas_source_preserves_color_rgba_rows_and_untints_foreground() {
    let mut image = test_image(GlyphAtlasFormat::Color, 3, 2, 24);
    image.screen_x = 10.0;
    image.baseline_y = 10.0;
    image.top = 6;
    image.left = 1;
    image.foreground_color =
        native_bitmap_atlas_foreground_color(GlyphAtlasFormat::Color, [0.2, 0.4, 0.6, 0.8]);
    let screen_rect = screen_rect_for(image);
    let source_bytes = Arc::<[u8]>::from((0..24).collect::<Vec<u8>>());
    let clipped = GlyphAtlasScreenRect::new(screen_rect.x + 1.0, screen_rect.y, 2.0, 2.0);

    let clipped_source = native_bitmap_atlas_source_from_image(image, clipped, source_bytes, None)
        .expect("color glyph should produce RGBA atlas source bytes");

    assert_eq!(clipped_source.source.format, GlyphAtlasFormat::Color);
    assert_eq!(clipped_source.source.content_size, UVec2::new(2, 2));
    assert_eq!(clipped_source.source.source_byte_len, 16);
    assert_eq!(clipped_source.source.foreground_color, [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(
        clipped_source.bytes.as_ref(),
        &[4, 5, 6, 7, 8, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23]
    );
    assert!(clipped_source.was_clipped);
}

fn test_image(
    format: GlyphAtlasFormat,
    width: u16,
    height: u16,
    source_byte_len: usize,
) -> NativeBitmapGlyphImage {
    NativeBitmapGlyphImage {
        screen_x: 20.0,
        baseline_y: 21.0,
        top: 10,
        left: 0,
        width,
        height,
        format,
        source_byte_len,
        foreground_color: [0.9, 0.8, 0.7, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
    }
}

fn screen_rect_for(image: NativeBitmapGlyphImage) -> GlyphAtlasScreenRect {
    native_bitmap_atlas_screen_rect(
        image.screen_x,
        image.baseline_y,
        image.top,
        image.left,
        image.width,
        image.height,
    )
}

fn test_raster_key(format: GlyphAtlasFormat) -> GlyphRasterKey {
    GlyphRasterKey::from_request(GlyphRasterRequest {
        face: InstancedFaceId(7),
        glyph_id: 42,
        logical_px: 16.0,
        scale_factor: 1.0,
        screen_x: 0.0,
        snap_to_pixel: false,
        format,
        hinting: GlyphHintingMode::Full,
        smoothing: GlyphSmoothingMode::Grayscale,
        synthetic: SyntheticGlyphStyle::default(),
    })
}

#[test]
fn native_bitmap_atlas_format_maps_rgba_contents_to_atlas_formats() {
    assert_eq!(
        native_bitmap_atlas_format(SwashContent::Mask),
        Some(GlyphAtlasFormat::AlphaMask)
    );
    assert_eq!(
        native_bitmap_atlas_format(SwashContent::Color),
        Some(GlyphAtlasFormat::Color)
    );
    assert_eq!(
        native_bitmap_atlas_format(SwashContent::SubpixelMask),
        Some(GlyphAtlasFormat::SubpixelMask)
    );
}
