use crate::core::math::UVec2;

use super::super::render_batch::{glyph_atlas_draw_batch_plan, GlyphAtlasDrawBatchPlan};
use super::super::render_gpu_plan::{glyph_atlas_gpu_draw_plan, GlyphAtlasGpuDrawPlan};
use super::super::render_plan::GlyphAtlasScreenRect;
use super::super::{
    glyph_atlas_bitmap_prepared_upload_plan, glyph_atlas_bitmap_run_plan_with_atlas,
    glyph_atlas_bitmap_run_plan_with_atlas_and_padding, GlyphAtlasBitmapPlaceholderGlyph,
    GlyphAtlasBitmapPreparedUploadPlan, GlyphAtlasBitmapRunPlan, GlyphAtlasBitmapSource,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasSet, GlyphAtlasUploadCommand,
};
use super::placeholder::{
    glyph_atlas_bitmap_placeholder_draw_plan, GlyphAtlasBitmapPlaceholderDrawPlan,
};
use super::report::{
    glyph_atlas_bitmap_render_submission_report, GlyphAtlasBitmapRenderSubmissionReport,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlyphAtlasBitmapRenderSubmissionPlan {
    pub(crate) run: GlyphAtlasBitmapRunPlan,
    pub(crate) draw_batches: GlyphAtlasDrawBatchPlan,
    pub(crate) gpu_draw: GlyphAtlasGpuDrawPlan,
    pub(crate) placeholder_draws: GlyphAtlasBitmapPlaceholderDrawPlan,
}

impl GlyphAtlasBitmapRenderSubmissionPlan {
    pub(crate) fn upload_commands(&self) -> &[GlyphAtlasUploadCommand] {
        &self.run.upload_commands
    }

    pub(crate) fn rebuilt_page_count(&self) -> usize {
        self.run.rebuilt_pages.len()
    }

    pub(crate) fn allocation_failure_count(&self) -> usize {
        self.run.allocation_failures.len()
    }

    pub(crate) fn submission_report(&self) -> GlyphAtlasBitmapRenderSubmissionReport {
        glyph_atlas_bitmap_render_submission_report(self)
    }

    pub(crate) fn append_placeholder_glyphs<I>(
        &mut self,
        placeholders: I,
        clip_rect: GlyphAtlasScreenRect,
    ) where
        I: IntoIterator<Item = GlyphAtlasBitmapPlaceholderGlyph>,
    {
        let original_count = self.run.placeholder_glyphs.len();
        self.run.placeholder_glyphs.extend(placeholders);
        if self.run.placeholder_glyphs.len() == original_count {
            return;
        }

        self.placeholder_draws = glyph_atlas_bitmap_placeholder_draw_plan(
            self.run.placeholder_glyphs.iter().copied(),
            clip_rect,
        );
    }

    pub(crate) fn prepared_upload<'a, I>(
        &self,
        source_bytes: I,
    ) -> GlyphAtlasBitmapPreparedUploadPlan
    where
        I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
    {
        glyph_atlas_bitmap_prepared_upload_plan(&self.run, source_bytes)
    }
}

pub(crate) fn glyph_atlas_bitmap_render_submission_plan<I>(
    sources: I,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRenderSubmissionPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_render_submission_plan_with_atlas(
        GlyphAtlasSet::default(),
        sources,
        page_size,
        frame_index,
        max_pages_per_format,
        viewport_size,
        clip_rect,
    )
}

pub(crate) fn glyph_atlas_bitmap_render_submission_plan_with_padding<I>(
    sources: I,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRenderSubmissionPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    glyph_atlas_bitmap_render_submission_plan_with_atlas_and_padding(
        GlyphAtlasSet::default(),
        sources,
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
        viewport_size,
        clip_rect,
    )
}

pub(crate) fn glyph_atlas_bitmap_render_submission_plan_with_atlas<I>(
    atlas: GlyphAtlasSet,
    sources: I,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRenderSubmissionPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    let run = glyph_atlas_bitmap_run_plan_with_atlas(
        atlas,
        sources,
        page_size,
        frame_index,
        max_pages_per_format,
    );
    bitmap_render_submission_from_run(run, viewport_size, clip_rect)
}

pub(crate) fn glyph_atlas_bitmap_render_submission_plan_with_atlas_and_padding<I>(
    atlas: GlyphAtlasSet,
    sources: I,
    page_size: UVec2,
    frame_index: u64,
    max_pages_per_format: usize,
    padding_px: u32,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRenderSubmissionPlan
where
    I: IntoIterator<Item = GlyphAtlasBitmapSource>,
{
    let run = glyph_atlas_bitmap_run_plan_with_atlas_and_padding(
        atlas,
        sources,
        page_size,
        frame_index,
        max_pages_per_format,
        padding_px,
    );
    bitmap_render_submission_from_run(run, viewport_size, clip_rect)
}

fn bitmap_render_submission_from_run(
    run: GlyphAtlasBitmapRunPlan,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRenderSubmissionPlan {
    let draw_batches = glyph_atlas_draw_batch_plan(run.draw_glyphs.iter().copied(), clip_rect);
    let gpu_draw = glyph_atlas_gpu_draw_plan(&draw_batches, viewport_size);
    let placeholder_draws =
        glyph_atlas_bitmap_placeholder_draw_plan(run.placeholder_glyphs.iter().copied(), clip_rect);

    GlyphAtlasBitmapRenderSubmissionPlan {
        run,
        draw_batches,
        gpu_draw,
        placeholder_draws,
    }
}
