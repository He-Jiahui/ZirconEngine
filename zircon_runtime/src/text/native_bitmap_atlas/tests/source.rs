use super::*;
use crate::text::atlas::{
    GlyphHintingMode, GlyphRasterKey, GlyphRasterRequest, GlyphSmoothingMode, SyntheticGlyphStyle,
};
use crate::text::InstancedFaceId;
use glyphon::TextBounds;
use std::sync::Arc;

#[test]
fn native_bitmap_atlas_source_uses_glyphon_bitmap_pixel_rect() {
    let image = NativeBitmapGlyphImage {
        x: 20,
        y: 7,
        line_y: 14.2,
        top: 11,
        left: -2,
        width: 9,
        height: 13,
        format: GlyphAtlasFormat::AlphaMask,
        scale_factor: 1.0,
        source_byte_len: 117,
        foreground_color: [0.9, 0.8, 0.7, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
    };
    let clipped = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
    let raster_key = test_raster_key(GlyphAtlasFormat::AlphaMask);
    let source_bytes = Arc::<[u8]>::from(vec![7; 117]);
    let clipped_source = native_bitmap_atlas_source_from_image(
        image,
        clipped,
        Arc::clone(&source_bytes),
        Some(raster_key),
    )
    .expect("alpha glyph image should produce an atlas source");
    let source = clipped_source.source;

    assert_eq!(source.format, GlyphAtlasFormat::AlphaMask);
    assert_eq!(source.content_size, UVec2::new(9, 13));
    assert_eq!(
        source.screen_rect,
        GlyphAtlasScreenRect::new(18.0, 10.0, 9.0, 13.0)
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
    let image = NativeBitmapGlyphImage {
        x: 20,
        y: 7,
        line_y: 14.2,
        top: 11,
        left: -2,
        width: 8,
        height: 8,
        format: GlyphAtlasFormat::SubpixelMask,
        scale_factor: 1.0,
        source_byte_len: 256,
        foreground_color: [0.9, 0.8, 0.7, 1.0],
        background_color: [0.1, 0.2, 0.3, 1.0],
    };
    let clipped = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
    let clipped_source =
        native_bitmap_atlas_source_from_image(image, clipped, Arc::from(vec![7; 256]), None)
            .expect("subpixel glyph image should preserve known background input");

    assert_eq!(clipped_source.source.format, GlyphAtlasFormat::SubpixelMask);
    assert_eq!(clipped_source.source.background_color, [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(clipped_source.bytes.len(), 256);
}

#[test]
fn native_bitmap_atlas_source_crops_alpha_rows_to_text_bounds() {
    let image = NativeBitmapGlyphImage {
        x: 20,
        y: 7,
        line_y: 14.2,
        top: 11,
        left: -2,
        width: 9,
        height: 4,
        format: GlyphAtlasFormat::AlphaMask,
        scale_factor: 1.0,
        source_byte_len: 36,
        foreground_color: [0.9, 0.8, 0.7, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
    };
    let source_bytes = Arc::<[u8]>::from((0..36).collect::<Vec<u8>>());
    let bounds = TextBounds {
        left: 20,
        top: 11,
        right: 25,
        bottom: 13,
    };
    let screen_rect = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
    let clipped = text_bounds_clipped_screen_rect(bounds, screen_rect)
        .expect("text bounds should keep the visible glyph slice");

    let clipped_source = native_bitmap_atlas_source_from_image(
        image,
        clipped,
        source_bytes,
        Some(test_raster_key(GlyphAtlasFormat::AlphaMask)),
    )
    .expect("text-bounds clipped alpha glyph should still produce an atlas source");

    assert_eq!(clipped_source.source.content_size, UVec2::new(5, 2));
    assert_eq!(
        clipped_source.source.screen_rect,
        GlyphAtlasScreenRect::new(20.0, 11.0, 5.0, 2.0)
    );
    assert_eq!(clipped_source.source.source_byte_len, 10);
    assert_eq!(clipped_source.source.raster_key, None);
    assert_eq!(
        clipped_source.bytes.as_ref(),
        &[11, 12, 13, 14, 15, 20, 21, 22, 23, 24]
    );
    assert!(clipped_source.was_clipped);
}

#[test]
fn native_bitmap_atlas_source_preserves_color_rgba_rows_and_untints_foreground() {
    let image = NativeBitmapGlyphImage {
        x: 10,
        y: 2,
        line_y: 8.0,
        top: 6,
        left: 1,
        width: 3,
        height: 2,
        format: GlyphAtlasFormat::Color,
        scale_factor: 1.0,
        source_byte_len: 24,
        foreground_color: native_bitmap_atlas_foreground_color(
            GlyphAtlasFormat::Color,
            [0.2, 0.4, 0.6, 0.8],
        ),
        background_color: [0.0, 0.0, 0.0, 1.0],
    };
    let source_bytes = Arc::<[u8]>::from((0..24).collect::<Vec<u8>>());
    let screen_rect = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
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

#[test]
fn native_bitmap_atlas_frame_replaces_glyphon_when_text_bounds_clip_alpha_source() {
    let inside = GlyphAtlasScreenRect::new(10.0, 8.0, 24.0, 12.0);
    let bounds = TextBounds {
        left: 8,
        top: 6,
        right: 40,
        bottom: 24,
    };

    assert_eq!(
        text_bounds_clipped_screen_rect(bounds, inside),
        Some(inside)
    );
    assert_eq!(
        text_bounds_clipped_screen_rect(bounds, GlyphAtlasScreenRect::new(10.0, 8.0, 40.0, 12.0)),
        Some(GlyphAtlasScreenRect::new(10.0, 8.0, 30.0, 12.0))
    );

    let clipped_source = GlyphAtlasBitmapSource {
        raster_key: None,
        format: GlyphAtlasFormat::AlphaMask,
        content_size: UVec2::new(30, 12),
        screen_rect: GlyphAtlasScreenRect::new(10.0, 8.0, 30.0, 12.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
        source_byte_len: 360,
    };
    let submission = test_submission([clipped_source]);
    let frame = test_frame(
        submission,
        vec![test_source_image(clipped_source, vec![255; 360])],
        1,
        0,
        1,
    );

    assert!(frame.replaces_glyphon());
}
