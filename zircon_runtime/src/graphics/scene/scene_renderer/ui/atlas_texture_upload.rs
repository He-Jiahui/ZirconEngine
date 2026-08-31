mod binding;
mod frame;
mod resource;
mod submission;
mod write;

pub(in crate::graphics::scene::scene_renderer::ui) use frame::{
    GlyphAtlasBitmapPreparedTextureUpload, GlyphAtlasBitmapTextureUploadFramePlan,
    GlyphAtlasBitmapTextureUploadFrameReport, glyph_atlas_bitmap_texture_upload_frame_plan,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity,
    prepare_glyph_atlas_bitmap_texture_upload_for_resources,
};
pub(in crate::graphics::scene::scene_renderer::ui) use resource::{
    GlyphAtlasTextureArrayResources, create_glyph_atlas_texture_array_resources,
    glyph_atlas_texture_array_spec,
};
pub(in crate::graphics::scene::scene_renderer::ui) use submission::glyph_atlas_bitmap_render_submission_texture_upload_frame_report;
pub(in crate::graphics::scene::scene_renderer::ui) use write::{
    glyph_atlas_texture_upload_region, glyph_atlas_texture_upload_source_range,
    glyph_atlas_texture_upload_write,
};

#[cfg(test)]
mod tests;
