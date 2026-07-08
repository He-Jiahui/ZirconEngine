use crate::core::math::UVec2;
use crate::graphics::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::graphics::text::atlas::{
    glyph_atlas_bitmap_retry_frame_driver_submit_with_atlas_and_config,
    GlyphAtlasBitmapQueuedGlyph, GlyphAtlasBitmapRenderSubmissionPlan,
    GlyphAtlasBitmapRetryFrameDriverConfig, GlyphAtlasBitmapRetryFrameState,
    GlyphAtlasBitmapRetryFrameStateReport, GlyphAtlasBitmapRetryFrameSubmissionPlan,
    GlyphAtlasBitmapRetryFrameSubmissionReport, GlyphAtlasBitmapRetrySourceOrigin, GlyphAtlasSet,
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
};

use super::{bitmap_atlas_page_size, NativeBitmapAtlasSourceImage};

pub(super) struct NativeBitmapAtlasRetryFrame {
    pub(super) submission: GlyphAtlasBitmapRenderSubmissionPlan,
    pub(super) source_images: Vec<NativeBitmapAtlasSourceImage>,
    pub(super) retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport,
    pub(super) retry_state: GlyphAtlasBitmapRetryFrameStateReport,
    pub(super) discarded_stale_retry_glyph_count: usize,
}

struct NativeBitmapAtlasRetrySourceSelection {
    retry_glyphs: Vec<GlyphAtlasBitmapQueuedGlyph>,
    new_source_images: Vec<NativeBitmapAtlasSourceImage>,
    discarded_stale_retry_glyph_count: usize,
}

pub(super) fn native_bitmap_atlas_retry_frame(
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
        GlyphAtlasBitmapRetryFrameDriverConfig::with_defaults(
            bitmap_atlas_page_size(),
            GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
            viewport_size,
            clip_rect,
        ),
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

fn native_bitmap_atlas_select_visible_retry_sources(
    retry_state: &GlyphAtlasBitmapRetryFrameState,
    source_images: &[NativeBitmapAtlasSourceImage],
) -> NativeBitmapAtlasRetrySourceSelection {
    let mut retry_glyphs = Vec::new();
    let mut retry_source_indices = Vec::new();
    let queued_glyphs = retry_state.queued_blocked_glyphs();
    let mut matched_retry_indices = vec![false; queued_glyphs.len()];

    for (source_index, source_image) in source_images.iter().enumerate() {
        if let Some((queued_index, queued)) =
            queued_glyphs
                .iter()
                .enumerate()
                .find(|(queued_index, queued)| {
                    !matched_retry_indices[*queued_index] && queued.source == source_image.source
                })
        {
            let mut retry_glyph = *queued;
            retry_glyph.source_index = source_index;
            retry_glyphs.push(retry_glyph);
            retry_source_indices.push(source_index);
            matched_retry_indices[queued_index] = true;
        }
    }
    let discarded_stale_retry_glyph_count = matched_retry_indices
        .iter()
        .filter(|matched| !**matched)
        .count();

    let new_source_images = source_images
        .iter()
        .enumerate()
        .filter(|(source_index, _)| !retry_source_indices.contains(source_index))
        .map(|(_, source_image)| source_image.clone())
        .collect();

    NativeBitmapAtlasRetrySourceSelection {
        retry_glyphs,
        new_source_images,
        discarded_stale_retry_glyph_count,
    }
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
