use super::bitmap::{ALPHA_MASK_CHANNELS, COLOR_BITMAP_CHANNELS};
use super::rasterizer::glyph_bitmap_from_swash_image;
use super::*;
use crate::core::math::{UVec2, Vec2};
use crate::graphics::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};
use ::swash::scale::image::{Content as SwashImageContent, Image as SwashImage};
use ::swash::FontRef;

const TEST_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraSans-Regular.ttf"
));

#[test]
fn text_raster_swash_glyph_bitmap_tracks_storage_format() {
    let alpha = GlyphBitmap::alpha_mask(UVec2::new(2, 2), Vec2::new(1.0, 3.0), 14.0, vec![255; 4])
        .expect("alpha mask bitmap should be valid");
    let subpixel =
        GlyphBitmap::subpixel_mask(UVec2::new(2, 2), Vec2::new(1.0, 3.0), 14.0, vec![255; 16])
            .expect("subpixel mask bitmap should be valid");
    let color = GlyphBitmap::color(UVec2::new(2, 2), Vec2::new(1.0, 3.0), 14.0, vec![255; 16])
        .expect("color bitmap should be valid");

    assert_eq!(alpha.atlas_format(), Some(GlyphAtlasFormat::AlphaMask));
    assert_eq!(
        alpha.storage_format(),
        Some(GlyphAtlasStorageFormat::R8Unorm)
    );
    assert!(alpha.has_expected_data_len());

    assert_eq!(
        subpixel.atlas_format(),
        Some(GlyphAtlasFormat::SubpixelMask)
    );
    assert_eq!(
        subpixel.storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert_eq!(subpixel.content, GlyphBitmapContent::SubpixelMask);
    assert!(subpixel.has_expected_data_len());

    assert_eq!(color.atlas_format(), Some(GlyphAtlasFormat::Color));
    assert_eq!(
        color.storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(color.has_expected_data_len());
}

#[test]
fn text_raster_swash_glyph_bitmap_rejects_data_length_mismatch() {
    let error = GlyphBitmap::color(UVec2::new(2, 2), Vec2::new(1.0, 3.0), 14.0, vec![255; 15])
        .expect_err("color bitmap should reject incomplete rgba data");

    assert_eq!(
        error,
        GlyphBitmapError::DataLengthMismatch {
            expected: 16,
            actual: 15,
        }
    );
}

#[test]
fn text_raster_swash_glyph_bitmap_rejects_invalid_metrics() {
    let empty_size =
        GlyphBitmap::alpha_mask(UVec2::new(0, 2), Vec2::new(1.0, 3.0), 14.0, vec![255; 2])
            .expect_err("zero-width glyph bitmap should be rejected");
    assert_eq!(
        empty_size,
        GlyphBitmapError::EmptySize {
            size: UVec2::new(0, 2),
        }
    );

    let invalid_bearing = GlyphBitmap::alpha_mask(
        UVec2::new(1, 1),
        Vec2::new(f32::NAN, 3.0),
        14.0,
        vec![255; 1],
    )
    .expect_err("non-finite bearing should be rejected");
    assert_eq!(invalid_bearing, GlyphBitmapError::InvalidBearing);

    let invalid_px_size =
        GlyphBitmap::alpha_mask(UVec2::new(1, 1), Vec2::new(1.0, 3.0), 0.0, vec![255; 1])
            .expect_err("non-positive px size should be rejected");
    assert_eq!(invalid_px_size, GlyphBitmapError::InvalidPxSize);
}

#[test]
fn text_raster_swash_normalizes_mask_image_to_alpha_bitmap() {
    let bitmap = glyph_bitmap_from_swash_image(
        swash_image(SwashImageContent::Mask, -2, 11, 2, 3, vec![128; 6]),
        16.0,
    )
    .expect("mask image should normalize to alpha bitmap");

    assert_eq!(bitmap.size, UVec2::new(2, 3));
    assert_eq!(bitmap.bearing, Vec2::new(-2.0, 11.0));
    assert_eq!(bitmap.channels, ALPHA_MASK_CHANNELS);
    assert_eq!(
        bitmap.storage_format(),
        Some(GlyphAtlasStorageFormat::R8Unorm)
    );
}

#[test]
fn text_raster_swash_normalizes_color_image_to_rgba_bitmap() {
    let bitmap = glyph_bitmap_from_swash_image(
        swash_image(SwashImageContent::Color, 1, 9, 2, 2, vec![255; 16]),
        16.0,
    )
    .expect("color image should normalize to rgba bitmap");

    assert_eq!(bitmap.size, UVec2::new(2, 2));
    assert_eq!(bitmap.channels, COLOR_BITMAP_CHANNELS);
    assert_eq!(
        bitmap.storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
}

#[test]
fn text_raster_swash_normalizes_subpixel_mask_image_to_rgba_bitmap() {
    let bitmap = glyph_bitmap_from_swash_image(
        swash_image(SwashImageContent::SubpixelMask, 1, 9, 2, 2, vec![255; 16]),
        16.0,
    )
    .expect("subpixel masks should normalize to a dedicated rgba atlas format");

    assert_eq!(bitmap.size, UVec2::new(2, 2));
    assert_eq!(bitmap.channels, COLOR_BITMAP_CHANNELS);
    assert_eq!(bitmap.content, GlyphBitmapContent::SubpixelMask);
    assert_eq!(bitmap.atlas_format(), Some(GlyphAtlasFormat::SubpixelMask));
    assert_eq!(
        bitmap.storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
}

#[test]
fn text_raster_swash_rasterizer_rejects_invalid_font_face() {
    let mut rasterizer = SwashRasterizer::new();
    let error = rasterizer
        .rasterize_alpha_outline(&[], 0, 1, 16.0, true)
        .expect_err("empty font data should not create a swash face");

    assert_eq!(error, SwashRasterError::InvalidFontFace { face_index: 0 });
}

#[test]
fn text_raster_swash_request_rejects_invalid_px_size_before_scaling() {
    let mut rasterizer = SwashRasterizer::new();
    let error = rasterizer
        .rasterize_alpha_outline(&[], 0, 1, 0.0, true)
        .expect_err("zero px size should be rejected before parsing font data");

    assert_eq!(error, SwashRasterError::InvalidPxSize);
}

#[test]
fn text_raster_swash_subpixel_outline_request_uses_subpixel_render_format() {
    let request = SwashRasterRequest::subpixel_outline(0, 1, 16.0, true);

    assert_eq!(
        request.source.render_format(),
        ::swash::zeno::Format::Subpixel
    );
    assert_eq!(
        SwashRasterRequest::alpha_outline(0, 1, 16.0, true)
            .source
            .render_format(),
        ::swash::zeno::Format::Alpha
    );
}

#[test]
fn text_raster_swash_rasterizer_renders_real_alpha_outline_glyph() {
    let font = FontRef::from_index(TEST_FONT_BYTES, 0).expect("test font should parse as face 0");
    let glyph_id = font.charmap().map('P');
    assert_ne!(glyph_id, 0, "test glyph should be present in FiraSans");

    let mut rasterizer = SwashRasterizer::new();
    let bitmap = rasterizer
        .rasterize_alpha_outline(TEST_FONT_BYTES, 0, glyph_id, 18.0, true)
        .expect("real alpha outline glyph should rasterize");

    assert_eq!(bitmap.channels, ALPHA_MASK_CHANNELS);
    assert_eq!(bitmap.atlas_format(), Some(GlyphAtlasFormat::AlphaMask));
    assert!(bitmap.size.x > 0);
    assert!(bitmap.size.y > 0);
    assert!(bitmap.has_expected_data_len());
    assert!(
        bitmap.data.iter().any(|coverage| *coverage > 0),
        "alpha glyph should contain non-empty coverage"
    );
}

#[test]
fn text_raster_swash_rasterizer_renders_real_subpixel_outline_glyph() {
    let font = FontRef::from_index(TEST_FONT_BYTES, 0).expect("test font should parse as face 0");
    let glyph_id = font.charmap().map('P');
    assert_ne!(glyph_id, 0, "test glyph should be present in FiraSans");

    let mut rasterizer = SwashRasterizer::new();
    let bitmap = rasterizer
        .rasterize_subpixel_outline(TEST_FONT_BYTES, 0, glyph_id, 18.0, true)
        .expect("real subpixel outline glyph should rasterize");

    assert_eq!(bitmap.channels, COLOR_BITMAP_CHANNELS);
    assert_eq!(bitmap.content, GlyphBitmapContent::SubpixelMask);
    assert_eq!(bitmap.atlas_format(), Some(GlyphAtlasFormat::SubpixelMask));
    assert_eq!(
        bitmap.storage_format(),
        Some(GlyphAtlasStorageFormat::Rgba8Unorm)
    );
    assert!(bitmap.has_expected_data_len());
    assert!(
        bitmap.data.iter().any(|coverage| *coverage > 0),
        "subpixel glyph should contain non-empty coverage"
    );
}

#[test]
fn text_raster_emoji_strike_selection_prefers_nearest_larger_strike() {
    let selection = select_color_bitmap_strike(
        20.0,
        &[
            strike(16, 16, 1.0, 13.0, 18.0),
            strike(32, 32, 2.0, 26.0, 34.0),
            strike(24, 24, 1.5, 19.0, 26.0),
        ],
    )
    .expect("larger strike should be selected");

    assert_eq!(selection.strike.ppem, 24);
    assert_eq!(selection.fit, ColorGlyphBitmapStrikeFit::Downsample);
    assert_eq!(selection.scaled_size(), UVec2::new(20, 20));
    assert_near(selection.scaled_bearing().x, 1.25);
    assert_near(selection.scaled_bearing().y, 15.833);
    assert_near(selection.scaled_advance_px(), 21.667);
}

#[test]
fn text_raster_emoji_strike_selection_falls_back_to_largest_smaller_strike() {
    let selection = select_color_bitmap_strike(
        40.0,
        &[
            strike(16, 16, 1.0, 13.0, 18.0),
            strike(32, 32, 2.0, 26.0, 34.0),
        ],
    )
    .expect("largest smaller strike should be selected when no larger strike exists");

    assert_eq!(selection.strike.ppem, 32);
    assert_eq!(selection.fit, ColorGlyphBitmapStrikeFit::UpscaleFallback);
    assert_eq!(selection.scaled_size(), UVec2::new(40, 40));
    assert_near(selection.scaled_bearing().x, 2.5);
    assert_near(selection.scaled_bearing().y, 32.5);
    assert_near(selection.scaled_advance_px(), 42.5);
}

#[test]
fn text_raster_emoji_strike_selection_ignores_invalid_strikes() {
    let selection = select_color_bitmap_strike(
        20.0,
        &[
            strike(0, 24, 1.0, 19.0, 26.0),
            strike(24, 0, 1.0, 19.0, 26.0),
            strike(24, 24, f32::NAN, 19.0, 26.0),
            strike(24, 24, 1.0, 19.0, -1.0),
            strike(32, 32, 2.0, 26.0, 34.0),
        ],
    )
    .expect("valid strike should be selected after invalid strikes are ignored");

    assert_eq!(selection.strike.ppem, 32);
    assert_eq!(selection.fit, ColorGlyphBitmapStrikeFit::Downsample);
}

#[test]
fn text_raster_emoji_colr_source_preempts_bitmap_strikes() {
    let plan = color_glyph_raster_plan(true, 20.0, &[strike(24, 24, 1.5, 19.0, 26.0)]);

    assert_eq!(plan, ColorGlyphRasterPlan::ColrCpalVector);
}

#[test]
fn text_raster_emoji_missing_bitmap_strike_is_explicit() {
    let plan = color_glyph_raster_plan(false, 20.0, &[]);

    assert_eq!(plan, ColorGlyphRasterPlan::Missing);
}

fn strike(
    ppem: u16,
    bitmap_size: u32,
    bearing_x: f32,
    bearing_y: f32,
    advance_px: f32,
) -> ColorGlyphBitmapStrike {
    ColorGlyphBitmapStrike::new(
        ppem,
        UVec2::new(bitmap_size, bitmap_size),
        Vec2::new(bearing_x, bearing_y),
        advance_px,
    )
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected {actual} to be near {expected}"
    );
}

fn swash_image(
    content: SwashImageContent,
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    data: Vec<u8>,
) -> SwashImage {
    let mut image = SwashImage::new();
    image.content = content;
    image.placement = ::swash::zeno::Placement {
        left,
        top,
        width,
        height,
    };
    image.data = data;
    image
}
