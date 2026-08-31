use std::sync::Arc;

use glyphon::SwashContent;

use crate::core::math::UVec2;
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT, GlyphAtlasBitmapFaceValidity,
    GlyphAtlasBitmapPlaceholderGlyph, GlyphAtlasBitmapPlaceholderMode, GlyphAtlasBitmapQueuedGlyph,
    GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapRenderSubmissionReport,
    GlyphAtlasBitmapRetryFrameState, GlyphAtlasBitmapRetryFrameStateReport,
    GlyphAtlasBitmapRetryFrameSubmissionReport, GlyphAtlasBitmapSource, GlyphAtlasFormat,
    GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasSet, GlyphAtlasStorageFormat,
    GlyphHintingMode, GlyphRasterKey, GlyphSmoothingMode, SyntheticGlyphStyle,
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas,
};
use crate::text::font::FontDatabase;
use crate::text::parallel::raster_pool::{TextRasterWorkerPool, TextRasterWorkerPoolOptions};

use super::source_image::{
    NativeBitmapGlyphImage, native_bitmap_atlas_foreground_color, native_bitmap_atlas_format,
    native_bitmap_atlas_screen_rect, native_bitmap_atlas_source_from_image,
};
use super::*;

const TEST_BITMAP_ATLAS_FRAME_INDEX: u64 = 17;

#[path = "tests/retry_frame.rs"]
mod retry_frame_tests;

#[path = "tests/source_cache.rs"]
mod source_cache_tests;

#[path = "tests/handoff.rs"]
mod handoff_tests;

#[path = "tests/source.rs"]
mod source_tests;

#[path = "tests/frame.rs"]
mod frame_tests;

#[path = "tests/storage.rs"]
mod storage_tests;

fn test_viewport_size() -> UVec2 {
    UVec2::new(128, 64)
}

fn test_clip_rect() -> GlyphAtlasScreenRect {
    GlyphAtlasScreenRect::new(0.0, 0.0, 128.0, 64.0)
}

fn test_submission<I>(sources: I) -> GlyphAtlasBitmapRenderSubmissionPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_render_submission_plan(
        sources,
        UVec2::new(64, 64),
        TEST_BITMAP_ATLAS_FRAME_INDEX,
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
        test_viewport_size(),
        test_clip_rect(),
    )
}

fn test_source_image(
    source: GlyphAtlasBitmapSource,
    bytes: Vec<u8>,
) -> NativeBitmapAtlasSourceImage {
    NativeBitmapAtlasSourceImage {
        source,
        bytes: Arc::from(bytes),
        face_epoch: 0,
    }
}

fn test_cache_key(glyph_id: u16) -> GlyphRasterKey {
    GlyphRasterKey {
        face: crate::text::InstancedFaceId(17),
        glyph_id: u32::from(glyph_id),
        px_size_bucket: 16,
        subpixel_bin: 0,
        vertical_subpixel_bin: 0,
        format: GlyphAtlasFormat::AlphaMask,
        hinting: GlyphHintingMode::Full,
        smoothing: GlyphSmoothingMode::Grayscale,
        synthetic: SyntheticGlyphStyle::default(),
    }
}

fn test_glyph_run(glyph_id: u16) -> NativeBitmapAtlasGlyphRun {
    test_glyph_run_with_key(test_cache_key(glyph_id), test_clip_rect())
}

fn test_glyph_run_with_key(
    raster_key: GlyphRasterKey,
    bounds: GlyphAtlasScreenRect,
) -> NativeBitmapAtlasGlyphRun {
    NativeBitmapAtlasGlyphRun::new(
        bounds,
        vec![NativeBitmapAtlasGlyph {
            raster_key,
            screen_x: 12.0,
            baseline_y: 24.0,
            placeholder_rect: GlyphAtlasScreenRect::new(12.0, 8.0, 12.0, 20.0),
            foreground_color: [1.0; 4],
            background_color: None,
        }],
    )
}

fn test_font_database_with_fira() -> (FontDatabase, crate::text::InstancedFaceId) {
    let source_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut font_database = FontDatabase::default();
    let face = font_database
        .register_font_file(&source_path, Some("Zircon Native Atlas Test"), 0)
        .expect("test font should register with the text font database");
    let instance = font_database
        .effective_instance_id(face, 400)
        .expect("registered test face should resolve an exact instance");
    (font_database, instance)
}

fn test_cached_image(byte: u8) -> super::source_cache::NativeBitmapAtlasCachedGlyphImage {
    super::source_cache::NativeBitmapAtlasCachedGlyphImage {
        content: SwashContent::Mask,
        top: 0,
        left: 0,
        width: 2,
        height: 2,
        bytes: Arc::from(vec![byte; 4]),
    }
}

fn test_frame(
    submission: GlyphAtlasBitmapRenderSubmissionPlan,
    source_images: Vec<NativeBitmapAtlasSourceImage>,
    visible_raster_glyph_count: usize,
    unsupported_glyph_count: usize,
    clipped_glyph_count: usize,
) -> NativeBitmapAtlasFrame {
    NativeBitmapAtlasFrame {
        submission,
        source_images,
        frame_index: TEST_BITMAP_ATLAS_FRAME_INDEX,
        viewport_size: test_viewport_size(),
        clip_rect: test_clip_rect(),
        visible_raster_glyph_count,
        missing_raster_image_count: 0,
        visible_missing_raster_image_count: 0,
        approximate_raster_image_count: 0,
        unsupported_glyph_count,
        clipped_glyph_count,
        background_composite_glyph_count: 0,
        missing_background_composite_glyph_count: 0,
        source_cache: NativeBitmapAtlasSourceCacheFrameReport::default(),
        retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport::default(),
        retry_state: GlyphAtlasBitmapRetryFrameStateReport::default(),
        discarded_stale_retry_glyph_count: 0,
        face_epoch: 0,
    }
}
