//! 核验位图 staging 与逐页上传请求，并准备统一资源上传批次；任一失败都拒绝整批及 shadow 提交。
mod binding;
mod frame;
mod resource;
mod submission;
mod write;

pub(in crate::graphics::scene::scene_renderer::ui) use frame::{
    glyph_atlas_bitmap_texture_upload_frame_plan,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity,
    prepare_glyph_atlas_bitmap_texture_upload_for_resources, GlyphAtlasBitmapPreparedTextureUpload,
    GlyphAtlasBitmapTextureUploadFramePlan, GlyphAtlasBitmapTextureUploadFrameReport,
};
pub(in crate::graphics::scene::scene_renderer::ui) use resource::{
    create_glyph_atlas_texture_array_resources, glyph_atlas_texture_array_spec,
    GlyphAtlasTextureArrayResources,
};
pub(in crate::graphics::scene::scene_renderer::ui) use submission::glyph_atlas_bitmap_render_submission_texture_upload_frame_report;
pub(in crate::graphics::scene::scene_renderer::ui) use write::{
    glyph_atlas_texture_upload_region, glyph_atlas_texture_upload_source_range,
    glyph_atlas_texture_upload_write,
};

#[cfg(test)]
#[path = "atlas_texture_upload/tests/cases.rs"]
mod tests;
