use std::ops::Range;

use crate::core::math::UVec2;
use crate::text::atlas::render_batch::glyph_atlas_draw_batch_plan;
use crate::text::atlas::render_gpu_plan::glyph_atlas_gpu_draw_plan;
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapRunPlan,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasDirtyPage, GlyphAtlasFormat,
    GlyphAtlasStorageFormat, GlyphAtlasUploadMode, glyph_atlas_upload_command,
};

use super::{
    NativeBitmapAtlasSourceImage, bitmap_atlas_page_size,
    glyph_atlas_bitmap_face_validity_for_epoch,
};

pub(crate) fn native_bitmap_atlas_storage_submissions(
    frame_submission: &GlyphAtlasBitmapRenderSubmissionPlan,
    source_images: &[NativeBitmapAtlasSourceImage],
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
    face_epoch: u64,
) -> Vec<NativeBitmapAtlasStorageSubmission> {
    let mut runs = Vec::<(GlyphAtlasFormat, Range<usize>)>::new();
    for (source_index, image) in source_images.iter().enumerate() {
        let atlas_format = image.source.format;
        match runs.last_mut() {
            Some((run_format, source_range)) if *run_format == atlas_format => {
                source_range.end = source_index.saturating_add(1);
            }
            _ => runs.push((atlas_format, source_index..source_index.saturating_add(1))),
        }
    }

    runs.into_iter()
        .map(|(atlas_format, source_range)| {
            NativeBitmapAtlasStorageSubmission::from_frame_submission(
                atlas_format,
                frame_submission,
                source_images[source_range.clone()].to_vec(),
                source_range,
                viewport_size,
                clip_rect,
                face_epoch,
            )
        })
        .collect()
}

pub(crate) struct NativeBitmapAtlasStorageSubmission {
    pub(crate) atlas_format: GlyphAtlasFormat,
    pub(crate) storage_format: GlyphAtlasStorageFormat,
    pub(crate) submission: GlyphAtlasBitmapRenderSubmissionPlan,
    source_images: Vec<NativeBitmapAtlasSourceImage>,
    face_epoch: u64,
}

impl NativeBitmapAtlasStorageSubmission {
    fn from_frame_submission(
        atlas_format: GlyphAtlasFormat,
        frame_submission: &GlyphAtlasBitmapRenderSubmissionPlan,
        source_images: Vec<NativeBitmapAtlasSourceImage>,
        source_range: Range<usize>,
        viewport_size: UVec2,
        clip_rect: GlyphAtlasScreenRect,
        face_epoch: u64,
    ) -> Self {
        let submission = native_bitmap_atlas_storage_submission_plan(
            frame_submission,
            source_range,
            viewport_size,
            clip_rect,
        );

        Self {
            atlas_format,
            storage_format: atlas_format.storage_format(),
            submission,
            source_images,
            face_epoch,
        }
    }

    pub(crate) fn source_bytes(&self) -> Vec<GlyphAtlasBitmapUploadSourceBytes<'_>> {
        self.source_images
            .iter()
            .enumerate()
            .map(|(source_index, image)| {
                GlyphAtlasBitmapUploadSourceBytes::with_face_epoch(
                    source_index,
                    &image.bytes,
                    image.face_epoch,
                )
            })
            .collect()
    }

    pub(crate) fn face_validity(&self) -> GlyphAtlasBitmapFaceValidity {
        glyph_atlas_bitmap_face_validity_for_epoch(
            self.source_images.iter().map(|image| image.face_epoch),
            self.face_epoch,
        )
    }

    pub(crate) fn atlas_layer_count(&self) -> u32 {
        self.submission
            .gpu_draw
            .instances
            .iter()
            .map(|instance| instance.page_index)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(1)
    }

    pub(crate) fn source_image_count(&self) -> usize {
        self.source_images.len()
    }

    pub(crate) fn visible_glyph_count(&self) -> usize {
        self.submission.gpu_draw.visible_glyph_count
    }

    pub(crate) fn has_allocation_failures(&self) -> bool {
        !self.submission.run.allocation_failures.is_empty()
    }
}

fn native_bitmap_atlas_storage_submission_plan(
    frame_submission: &GlyphAtlasBitmapRenderSubmissionPlan,
    source_range: Range<usize>,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasBitmapRenderSubmissionPlan {
    let source_start = source_range.start;
    let contains_source = |source_index| source_range.contains(&source_index);
    let mut run = GlyphAtlasBitmapRunPlan {
        atlas: frame_submission.run.atlas.clone(),
        zero_initialize_shadow_pages: frame_submission.run.zero_initialize_shadow_pages.clone(),
        glyphs: frame_submission
            .run
            .glyphs
            .iter()
            .filter(|glyph| contains_source(glyph.source_index))
            .map(|glyph| {
                let mut glyph = *glyph;
                glyph.source_index = glyph.source_index.saturating_sub(source_start);
                glyph
            })
            .collect(),
        allocation_failures: frame_submission
            .run
            .allocation_failures
            .iter()
            .filter(|failure| contains_source(failure.source_index))
            .map(|failure| {
                let mut failure = *failure;
                failure.source_index = failure.source_index.saturating_sub(source_start);
                failure
            })
            .collect(),
        blocked_glyphs: frame_submission
            .run
            .blocked_glyphs
            .iter()
            .filter(|glyph| contains_source(glyph.source_index))
            .map(|glyph| {
                let mut glyph = *glyph;
                glyph.source_index = glyph.source_index.saturating_sub(source_start);
                glyph
            })
            .collect(),
        placeholder_glyphs: frame_submission
            .run
            .placeholder_glyphs
            .iter()
            .filter(|glyph| contains_source(glyph.source_index))
            .map(|glyph| {
                let mut glyph = *glyph;
                glyph.source_index = glyph.source_index.saturating_sub(source_start);
                glyph
            })
            .collect(),
        ..GlyphAtlasBitmapRunPlan::default()
    };
    run.draw_glyphs = frame_submission
        .run
        .glyphs
        .iter()
        .zip(&frame_submission.run.draw_glyphs)
        .filter_map(|(glyph, draw_glyph)| contains_source(glyph.source_index).then_some(*draw_glyph))
        .collect();
    run.upload_copies = frame_submission
        .run
        .upload_copies
        .iter()
        .filter(|copy| contains_source(copy.source_index))
        .map(|copy| {
            let mut copy = *copy;
            copy.source_index = copy.source_index.saturating_sub(source_start);
            copy
        })
        .collect();
    let mut retained_slot_rects_by_page = run.atlas.persistent_bitmap_slot_rects_by_page();
    // This storage split may write only a subset of the frame's new slots. Slots
    // owned by another split remain protected until that split has staged them.
    for copy in &run.upload_copies {
        if let Some(retained_regions) = retained_slot_rects_by_page.get_mut(&copy.page_key) {
            retained_regions.retain(|retained| *retained != copy.atlas_rect);
        }
    }
    for copy in &run.upload_copies {
        let dirty_page = run
            .dirty_pages
            .iter_mut()
            .find(|dirty_page| dirty_page.page_key() == copy.page_key);
        if let Some(dirty_page) = dirty_page {
            dirty_page.mark_dirty(copy.page_key, copy.atlas_rect);
        } else {
            let has_unstaged_split_copy = frame_submission.run.upload_copies.iter().any(|other| {
                other.page_key == copy.page_key && !contains_source(other.source_index)
            });
            // A pending zero-init commit, or a committed shadow that predates a
            // sibling split's new copy, cannot replay the whole current page.
            // Full-page upload would otherwise erase the sibling pixels before
            // the combined shadow commit becomes visible next frame.
            let has_replayable_shadow =
                !has_unstaged_split_copy && run.atlas.has_bitmap_page_shadow(copy.page_key);
            let mut dirty_page = retained_slot_rects_by_page
                .get(&copy.page_key)
                .cloned()
                .map_or_else(
                    || GlyphAtlasDirtyPage::new(copy.page_key),
                    |retained_regions| {
                        if has_replayable_shadow {
                            GlyphAtlasDirtyPage::new_with_replayable_shadow(
                                copy.page_key,
                                retained_regions,
                            )
                        } else {
                            GlyphAtlasDirtyPage::new_with_retained_regions(
                                copy.page_key,
                                retained_regions,
                            )
                        }
                    },
                );
            dirty_page.mark_dirty(copy.page_key, copy.atlas_rect);
            run.dirty_pages.push(dirty_page);
        }
    }
    run.upload_commands = native_bitmap_atlas_storage_upload_commands(&run);

    let draw_batches = glyph_atlas_draw_batch_plan(run.draw_glyphs.iter().copied(), clip_rect);
    let gpu_draw = glyph_atlas_gpu_draw_plan(&draw_batches, viewport_size);
    GlyphAtlasBitmapRenderSubmissionPlan {
        run,
        draw_batches,
        gpu_draw,
        placeholder_draws: frame_submission.placeholder_draws.clone(),
    }
}

fn native_bitmap_atlas_storage_upload_commands(
    run: &GlyphAtlasBitmapRunPlan,
) -> Vec<crate::text::atlas::GlyphAtlasUploadCommand> {
    let mut commands = Vec::new();
    for dirty_page in &run.dirty_pages {
        let page_key = dirty_page.page_key();
        let Some(page) = run.atlas.page(page_key.format, page_key.page_index) else {
            continue;
        };
        let page_rect = crate::text::atlas::GlyphAtlasRect {
            x: 0,
            y: 0,
            width: page.size.x.max(1),
            height: page.size.y.max(1),
        };
        let source_byte_len = (page.size.x as usize)
            .saturating_mul(page.size.y as usize)
            .saturating_mul(page.storage_format.bytes_per_pixel() as usize);
        for dirty_rect in dirty_page.regions_for_page(page_rect) {
            let mode = if dirty_rect == page_rect {
                GlyphAtlasUploadMode::FullPage
            } else {
                GlyphAtlasUploadMode::PartialRect
            };
            if let Some(command) =
                glyph_atlas_upload_command(page, mode, Some(dirty_rect), source_byte_len)
            {
                commands.push(command);
            }
        }
    }
    commands
}

pub(crate) fn single_native_bitmap_atlas_format<I>(formats: I) -> Option<GlyphAtlasFormat>
where
    I: IntoIterator<Item = GlyphAtlasFormat>,
{
    let mut formats = formats.into_iter();
    let first = formats.next()?;
    formats.all(|format| format == first).then_some(first)
}

pub(crate) fn single_native_bitmap_atlas_storage_format<I>(
    formats: I,
) -> Option<GlyphAtlasStorageFormat>
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut formats = formats.into_iter();
    let first = formats.next()?;
    formats.all(|format| format == first).then_some(first)
}

pub(crate) fn native_bitmap_atlas_has_mixed_storage_formats<I>(formats: I) -> bool
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut formats = formats.into_iter();
    let Some(first) = formats.next() else {
        return false;
    };
    formats.any(|format| format != first)
}
