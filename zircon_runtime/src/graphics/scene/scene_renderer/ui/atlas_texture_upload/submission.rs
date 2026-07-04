use crate::graphics::text::atlas::{
    GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapUploadSourceBytes,
};

use super::frame::{
    glyph_atlas_bitmap_texture_upload_frame_plan,
    write_glyph_atlas_bitmap_texture_upload_frame_resources,
    GlyphAtlasBitmapTextureUploadFrameReport,
};
use super::resource::GlyphAtlasTextureArrayResources;

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_bitmap_render_submission_texture_upload_frame_report<
    'a,
    I,
>(
    submission: &GlyphAtlasBitmapRenderSubmissionPlan,
    source_bytes: I,
) -> GlyphAtlasBitmapTextureUploadFrameReport
where
    I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
{
    let prepared_upload = submission.prepared_upload(source_bytes);
    let frame_plan = glyph_atlas_bitmap_texture_upload_frame_plan(&prepared_upload);
    frame_plan.report()
}

pub(in crate::graphics::scene::scene_renderer::ui) fn write_glyph_atlas_bitmap_render_submission_texture_upload_resources<
    'a,
    I,
>(
    queue: &wgpu::Queue,
    resources: &GlyphAtlasTextureArrayResources,
    submission: &GlyphAtlasBitmapRenderSubmissionPlan,
    source_bytes: I,
) -> GlyphAtlasBitmapTextureUploadFrameReport
where
    I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
{
    let prepared_upload = submission.prepared_upload(source_bytes);
    let frame_plan = glyph_atlas_bitmap_texture_upload_frame_plan(&prepared_upload);
    write_glyph_atlas_bitmap_texture_upload_frame_resources(queue, resources, &frame_plan)
}
