use crate::text::atlas::{GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapUploadSourceBytes};

use super::frame::{
    GlyphAtlasBitmapTextureUploadFrameReport,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas,
};

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
    let frame_plan = glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas(
        &prepared_upload,
        &submission.run.atlas,
    );
    frame_plan.report()
}
