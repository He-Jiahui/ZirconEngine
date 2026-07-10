use crate::core::math::UVec2;
use crate::graphics::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::graphics::text::atlas::{
    glyph_atlas_bitmap_render_submission_plan_with_atlas, GlyphAtlasBitmapFaceValidity,
    GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasSet,
    GlyphAtlasStorageFormat, GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
};

use super::{
    bitmap_atlas_page_size, glyph_atlas_bitmap_face_validity_for_epoch,
    NativeBitmapAtlasSourceImage,
};

pub(super) fn native_bitmap_atlas_storage_submissions(
    atlas: &GlyphAtlasSet,
    source_images: &[NativeBitmapAtlasSourceImage],
    frame_index: u64,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
    face_epoch: u64,
) -> Vec<NativeBitmapAtlasStorageSubmission> {
    let mut runs = Vec::<(GlyphAtlasStorageFormat, Vec<NativeBitmapAtlasSourceImage>)>::new();
    for image in source_images {
        let storage_format = image.source.format.storage_format();
        match runs.last_mut() {
            Some((run_format, run_images)) if *run_format == storage_format => {
                run_images.push(image.clone());
            }
            _ => runs.push((storage_format, vec![image.clone()])),
        }
    }

    runs.into_iter()
        .map(|(storage_format, run_images)| {
            NativeBitmapAtlasStorageSubmission::new(
                storage_format,
                atlas.clone(),
                run_images,
                frame_index,
                viewport_size,
                clip_rect,
                face_epoch,
            )
        })
        .collect()
}

pub(in crate::graphics::scene::scene_renderer::ui::text) struct NativeBitmapAtlasStorageSubmission {
    pub(in crate::graphics::scene::scene_renderer::ui::text) storage_format:
        GlyphAtlasStorageFormat,
    pub(in crate::graphics::scene::scene_renderer::ui::text) submission:
        GlyphAtlasBitmapRenderSubmissionPlan,
    source_images: Vec<NativeBitmapAtlasSourceImage>,
    face_epoch: u64,
}

impl NativeBitmapAtlasStorageSubmission {
    pub(super) fn new(
        storage_format: GlyphAtlasStorageFormat,
        atlas: GlyphAtlasSet,
        source_images: Vec<NativeBitmapAtlasSourceImage>,
        frame_index: u64,
        viewport_size: UVec2,
        clip_rect: GlyphAtlasScreenRect,
        face_epoch: u64,
    ) -> Self {
        let submission = glyph_atlas_bitmap_render_submission_plan_with_atlas(
            atlas,
            source_images.iter().map(|image| image.source),
            bitmap_atlas_page_size(),
            frame_index,
            GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
            viewport_size,
            clip_rect,
        );

        Self {
            storage_format,
            submission,
            source_images,
            face_epoch,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn source_bytes(
        &self,
    ) -> Vec<GlyphAtlasBitmapUploadSourceBytes<'_>> {
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

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn face_validity(
        &self,
    ) -> GlyphAtlasBitmapFaceValidity {
        glyph_atlas_bitmap_face_validity_for_epoch(
            self.source_images.iter().map(|image| image.face_epoch),
            self.face_epoch,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn atlas_layer_count(&self) -> u32 {
        self.submission
            .gpu_draw
            .vertices
            .iter()
            .map(|vertex| vertex.page_index)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(1)
    }

    pub(super) fn source_image_count(&self) -> usize {
        self.source_images.len()
    }

    pub(super) fn visible_glyph_count(&self) -> usize {
        self.submission.gpu_draw.visible_glyph_count
    }

    pub(super) fn has_allocation_failures(&self) -> bool {
        !self.submission.run.allocation_failures.is_empty()
    }
}

pub(super) fn single_native_bitmap_atlas_storage_format<I>(
    formats: I,
) -> Option<GlyphAtlasStorageFormat>
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut formats = formats.into_iter();
    let first = formats.next()?;
    formats.all(|format| format == first).then_some(first)
}

pub(super) fn native_bitmap_atlas_has_mixed_storage_formats<I>(formats: I) -> bool
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut formats = formats.into_iter();
    let Some(first) = formats.next() else {
        return false;
    };
    formats.any(|format| format != first)
}
