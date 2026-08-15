mod binding;
mod frame;
mod resource;
mod submission;
mod write;

pub(in crate::graphics::scene::scene_renderer::ui) use frame::{
    glyph_atlas_bitmap_texture_upload_frame_plan,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity,
    write_glyph_atlas_bitmap_texture_upload_frame_plan,
    write_glyph_atlas_bitmap_texture_upload_frame_resources,
    GlyphAtlasBitmapTextureUploadFramePlan, GlyphAtlasBitmapTextureUploadFrameReport,
};
pub(in crate::graphics::scene::scene_renderer::ui) use resource::{
    create_glyph_atlas_texture_array_resources, glyph_atlas_texture_array_spec,
    GlyphAtlasTextureArrayResources,
};
pub(in crate::graphics::scene::scene_renderer::ui) use submission::{
    glyph_atlas_bitmap_render_submission_texture_upload_frame_report,
    write_glyph_atlas_bitmap_render_submission_texture_upload_resources,
};
pub(in crate::graphics::scene::scene_renderer::ui) use write::write_glyph_atlas_texture_upload_command;

#[cfg(test)]
mod tests;
