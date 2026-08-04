use std::sync::Arc;

use glyphon::SwashContent;
use glyphon::cosmic_text::{CacheKey, CacheKeyFlags, SubpixelBin, Weight, fontdb};
use glyphon::{Attrs, Buffer, Metrics, Shaping};

use crate::text::atlas::{
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT, GlyphAtlasBitmapPlaceholderGlyph,
    GlyphAtlasBitmapPlaceholderMode, GlyphAtlasBitmapQueuedGlyph,
    GlyphAtlasBitmapRenderSubmissionReport, GlyphAtlasPageKey, GlyphAtlasPageSpec,
    glyph_atlas_bitmap_render_submission_plan,
    glyph_atlas_bitmap_render_submission_plan_with_atlas,
};
use crate::text::font::FontDatabase;
use crate::text::parallel::raster_pool::{TextRasterWorkerPool, TextRasterWorkerPoolOptions};

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

fn test_cache_key(glyph_id: u16) -> CacheKey {
    CacheKey {
        font_id: fontdb::ID::dummy(),
        glyph_id,
        font_size_bits: 16.0f32.to_bits(),
        x_bin: SubpixelBin::Zero,
        y_bin: SubpixelBin::Zero,
        font_weight: Weight(400),
        flags: CacheKeyFlags::empty(),
    }
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
