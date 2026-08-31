use std::collections::{HashMap, VecDeque};

use crate::core::math::UVec2;
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT, GlyphAtlasBitmapQueuedGlyph,
    GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapRetryBackpressurePolicy,
    GlyphAtlasBitmapRetryFrameDriverConfig, GlyphAtlasBitmapRetryFrameState,
    GlyphAtlasBitmapRetryFrameStateReport, GlyphAtlasBitmapRetryFrameSubmissionPlan,
    GlyphAtlasBitmapRetryFrameSubmissionReport, GlyphAtlasBitmapRetrySourceOrigin,
    GlyphAtlasBitmapSource, GlyphAtlasFormat, GlyphAtlasSet, GlyphRasterKey,
    glyph_atlas_bitmap_retry_frame_driver_submit_with_atlas_and_config,
};

use super::frame::{
    NATIVE_BITMAP_ATLAS_MAX_RASTER_COMPLETION_BYTES_PER_FRAME, NativeBitmapAtlasSourceImage,
    bitmap_atlas_page_size,
};

const NATIVE_BITMAP_ATLAS_MAX_RETRY_SOURCES_PER_FRAME: usize =
    super::source_cache::NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME / 2;
const NATIVE_BITMAP_ATLAS_MAX_RETRY_SOURCE_BYTES_PER_FRAME: usize =
    NATIVE_BITMAP_ATLAS_MAX_RASTER_COMPLETION_BYTES_PER_FRAME / 2;

pub(crate) struct NativeBitmapAtlasRetryFrame {
    pub(crate) submission: GlyphAtlasBitmapRenderSubmissionPlan,
    pub(crate) source_images: Vec<NativeBitmapAtlasSourceImage>,
    pub(crate) retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport,
    pub(crate) retry_state: GlyphAtlasBitmapRetryFrameStateReport,
    pub(crate) discarded_stale_retry_glyph_count: usize,
}

struct NativeBitmapAtlasRetrySourceSelection {
    retry_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    new_source_images: Vec<NativeBitmapAtlasSourceImage>,
    discarded_stale_retry_glyph_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NativeBitmapAtlasRetrySourceKey {
    raster_key: Option<GlyphRasterKey>,
    format: GlyphAtlasFormat,
    content_width: u32,
    content_height: u32,
    screen_rect: [u32; 4],
    foreground_color: [u32; 4],
    background_color: [u32; 4],
    source_byte_len: usize,
}

pub(crate) fn native_bitmap_atlas_retry_frame(
    retry_state: &mut GlyphAtlasBitmapRetryFrameState,
    atlas: GlyphAtlasSet,
    source_images: Vec<NativeBitmapAtlasSourceImage>,
    frame_index: u64,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> NativeBitmapAtlasRetryFrame {
    let selection =
        native_bitmap_atlas_select_visible_retry_sources(retry_state, source_images.as_slice());
    retry_state.replace_blocked_glyphs(selection.retry_glyphs.iter().copied());
    let output = glyph_atlas_bitmap_retry_frame_driver_submit_with_atlas_and_config(
        retry_state,
        atlas,
        selection.new_source_images.iter().map(|image| image.source),
        frame_index,
        native_bitmap_atlas_retry_frame_driver_config(viewport_size, clip_rect),
    );
    let retry_submission = output.retry_submission_report();
    let retry_state = output.state_report;
    let submission_source_images = native_bitmap_atlas_submission_source_images(
        source_images.as_slice(),
        &selection,
        &output.submission_plan,
    );

    NativeBitmapAtlasRetryFrame {
        submission: output.submission_plan.submission,
        source_images: submission_source_images,
        retry_submission,
        retry_state,
        discarded_stale_retry_glyph_count: selection.discarded_stale_retry_glyph_count,
    }
}

pub(crate) fn native_bitmap_atlas_retry_backpressure_policy()
-> GlyphAtlasBitmapRetryBackpressurePolicy {
    // Split the documented Text09 256 glyph / 2 MiB frame envelope evenly so retry pressure
    // cannot consume the entire new-visible-glyph budget. The retained queue is capped at the
    // full envelope; overflow fails closed to native degradation rather than retaining unbounded
    // work.
    GlyphAtlasBitmapRetryBackpressurePolicy {
        max_due_retry_sources_per_frame: Some(NATIVE_BITMAP_ATLAS_MAX_RETRY_SOURCES_PER_FRAME),
        max_due_retry_source_bytes_per_frame: Some(
            NATIVE_BITMAP_ATLAS_MAX_RETRY_SOURCE_BYTES_PER_FRAME,
        ),
        max_new_sources_per_frame: Some(NATIVE_BITMAP_ATLAS_MAX_RETRY_SOURCES_PER_FRAME),
        max_new_source_bytes_per_frame: Some(NATIVE_BITMAP_ATLAS_MAX_RETRY_SOURCE_BYTES_PER_FRAME),
        max_queued_blocked_glyphs: Some(
            super::source_cache::NATIVE_BITMAP_ATLAS_MAX_RASTER_REQUESTS_PER_FRAME,
        ),
        max_queued_blocked_source_bytes: Some(
            NATIVE_BITMAP_ATLAS_MAX_RASTER_COMPLETION_BYTES_PER_FRAME,
        ),
        defer_excess_by_frames: 1,
    }
}

fn native_bitmap_atlas_retry_frame_driver_config(
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRetryFrameDriverConfig {
    GlyphAtlasBitmapRetryFrameDriverConfig {
        backpressure_policy: native_bitmap_atlas_retry_backpressure_policy(),
        ..GlyphAtlasBitmapRetryFrameDriverConfig::with_defaults(
            bitmap_atlas_page_size(),
            GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
            viewport_size,
            clip_rect,
        )
    }
}

fn native_bitmap_atlas_select_visible_retry_sources(
    retry_state: &GlyphAtlasBitmapRetryFrameState,
    source_images: &[NativeBitmapAtlasSourceImage],
) -> NativeBitmapAtlasRetrySourceSelection {
    let queued_glyphs = retry_state.queued_blocked_glyphs();
    let mut source_indices_by_key =
        HashMap::<NativeBitmapAtlasRetrySourceKey, VecDeque<usize>>::new();
    for (source_index, source_image) in source_images.iter().enumerate() {
        let Some(key) = native_bitmap_atlas_retry_source_key(source_image.source) else {
            continue;
        };
        source_indices_by_key
            .entry(key)
            .or_default()
            .push_back(source_index);
    }

    let mut discarded_stale_retry_glyph_count: usize = 0;
    let mut matched_retry_source_indices = vec![false; source_images.len()];
    let mut retry_glyphs = Vec::with_capacity(queued_glyphs.len().min(source_images.len()));
    for queued in queued_glyphs.iter().copied() {
        let Some(key) = native_bitmap_atlas_retry_source_key(queued.source) else {
            discarded_stale_retry_glyph_count = discarded_stale_retry_glyph_count.saturating_add(1);
            continue;
        };
        let Some(source_index) = source_indices_by_key
            .get_mut(&key)
            .and_then(VecDeque::pop_front)
        else {
            discarded_stale_retry_glyph_count = discarded_stale_retry_glyph_count.saturating_add(1);
            continue;
        };
        let mut retry_glyph = queued;
        retry_glyph.source_index = source_index;
        matched_retry_source_indices[source_index] = true;
        retry_glyphs.push(retry_glyph);
    }
    let new_source_images = source_images
        .iter()
        .enumerate()
        .filter(|(source_index, _)| !matched_retry_source_indices[*source_index])
        .map(|(_, source_image)| source_image.clone())
        .collect();

    NativeBitmapAtlasRetrySourceSelection {
        retry_glyphs,
        new_source_images,
        discarded_stale_retry_glyph_count,
    }
}

fn native_bitmap_atlas_retry_source_key(
    source: GlyphAtlasBitmapSource,
) -> Option<NativeBitmapAtlasRetrySourceKey> {
    let source_floats = [
        source.screen_rect.x,
        source.screen_rect.y,
        source.screen_rect.width,
        source.screen_rect.height,
        source.foreground_color[0],
        source.foreground_color[1],
        source.foreground_color[2],
        source.foreground_color[3],
        source.background_color[0],
        source.background_color[1],
        source.background_color[2],
        source.background_color[3],
    ];
    // PartialEq never matches NaN sources, so they remain new/stale rather than hash-matching.
    if source_floats.iter().any(|value| value.is_nan()) {
        return None;
    }

    Some(NativeBitmapAtlasRetrySourceKey {
        raster_key: source.raster_key,
        format: source.format,
        content_width: source.content_size.x,
        content_height: source.content_size.y,
        screen_rect: [
            native_bitmap_atlas_retry_float_key(source.screen_rect.x),
            native_bitmap_atlas_retry_float_key(source.screen_rect.y),
            native_bitmap_atlas_retry_float_key(source.screen_rect.width),
            native_bitmap_atlas_retry_float_key(source.screen_rect.height),
        ],
        foreground_color: source
            .foreground_color
            .map(native_bitmap_atlas_retry_float_key),
        background_color: source
            .background_color
            .map(native_bitmap_atlas_retry_float_key),
        source_byte_len: source.source_byte_len,
    })
}

fn native_bitmap_atlas_retry_float_key(value: f32) -> u32 {
    // PartialEq treats both zero signs as equal.
    if value == 0.0 { 0 } else { value.to_bits() }
}

fn native_bitmap_atlas_submission_source_images(
    source_images: &[NativeBitmapAtlasSourceImage],
    selection: &NativeBitmapAtlasRetrySourceSelection,
    plan: &GlyphAtlasBitmapRetryFrameSubmissionPlan,
) -> Vec<NativeBitmapAtlasSourceImage> {
    plan.frame_input
        .source_origins
        .iter()
        .filter_map(|origin| match *origin {
            GlyphAtlasBitmapRetrySourceOrigin::Retried { source_index, .. } => {
                source_images.get(source_index).cloned()
            }
            GlyphAtlasBitmapRetrySourceOrigin::New { source_index } => {
                selection.new_source_images.get(source_index).cloned()
            }
        })
        .collect()
}
