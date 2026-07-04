use crate::graphics::text::atlas::{
    glyph_atlas_bitmap_texture_upload_request_plan, GlyphAtlasBitmapPreparedUploadPlan,
    GlyphAtlasBitmapTextureUploadRequestPlan,
};

use super::binding::{
    glyph_atlas_bitmap_texture_upload_binding_plan,
    write_glyph_atlas_bitmap_texture_upload_bindings, GlyphAtlasBitmapTextureUploadBindingPlan,
};
use super::resource::GlyphAtlasTextureArrayResources;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapTextureUploadFrameReport {
    pub(in crate::graphics::scene::scene_renderer::ui) request_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) binding_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) binding_failure_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) staging_failure_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) staged_upload_failure_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) skipped_staged_upload_failure_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) upload_byte_len: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) ready_to_write_texture: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapTextureUploadFramePlan<'a>
{
    request_plan: GlyphAtlasBitmapTextureUploadRequestPlan,
    binding_plan: GlyphAtlasBitmapTextureUploadBindingPlan<'a>,
    report: GlyphAtlasBitmapTextureUploadFrameReport,
}

impl GlyphAtlasBitmapTextureUploadFrameReport {
    pub(in crate::graphics::scene::scene_renderer::ui) fn has_failures(self) -> bool {
        self.binding_failure_count > 0
            || self.staging_failure_count > 0
            || self.staged_upload_failure_count > 0
    }
}

impl GlyphAtlasBitmapTextureUploadFramePlan<'_> {
    pub(in crate::graphics::scene::scene_renderer::ui) fn ready_to_write_texture(&self) -> bool {
        self.report.ready_to_write_texture
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn report(
        &self,
    ) -> GlyphAtlasBitmapTextureUploadFrameReport {
        self.report
    }
}

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_bitmap_texture_upload_frame_plan(
    prepared_upload: &GlyphAtlasBitmapPreparedUploadPlan,
) -> GlyphAtlasBitmapTextureUploadFramePlan<'_> {
    let request_plan =
        glyph_atlas_bitmap_texture_upload_request_plan(&prepared_upload.staged_uploads);
    let binding_plan = glyph_atlas_bitmap_texture_upload_binding_plan(
        &prepared_upload.staging.pages,
        &request_plan.requests,
    );
    let report = glyph_atlas_bitmap_texture_upload_frame_report(
        prepared_upload,
        &request_plan,
        &binding_plan,
    );

    GlyphAtlasBitmapTextureUploadFramePlan {
        request_plan,
        binding_plan,
        report,
    }
}

pub(in crate::graphics::scene::scene_renderer::ui) fn write_glyph_atlas_bitmap_texture_upload_frame_plan(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    plan: &GlyphAtlasBitmapTextureUploadFramePlan<'_>,
) -> GlyphAtlasBitmapTextureUploadFrameReport {
    if plan.ready_to_write_texture() {
        write_glyph_atlas_bitmap_texture_upload_bindings(
            queue,
            texture,
            &plan.binding_plan.bindings,
        );
    }
    plan.report
}

pub(in crate::graphics::scene::scene_renderer::ui) fn write_glyph_atlas_bitmap_texture_upload_frame_resources(
    queue: &wgpu::Queue,
    resources: &GlyphAtlasTextureArrayResources,
    plan: &GlyphAtlasBitmapTextureUploadFramePlan<'_>,
) -> GlyphAtlasBitmapTextureUploadFrameReport {
    write_glyph_atlas_bitmap_texture_upload_frame_plan(queue, resources.texture(), plan)
}

fn glyph_atlas_bitmap_texture_upload_frame_report(
    prepared_upload: &GlyphAtlasBitmapPreparedUploadPlan,
    request_plan: &GlyphAtlasBitmapTextureUploadRequestPlan,
    binding_plan: &GlyphAtlasBitmapTextureUploadBindingPlan<'_>,
) -> GlyphAtlasBitmapTextureUploadFrameReport {
    let upload_byte_len = binding_plan
        .bindings
        .iter()
        .filter_map(|binding| request_plan.requests.get(binding.request_index))
        .map(|request| request.upload_byte_len)
        .sum();
    let report = GlyphAtlasBitmapTextureUploadFrameReport {
        request_count: request_plan.requests.len(),
        binding_count: binding_plan.bindings.len(),
        binding_failure_count: binding_plan.failures.len(),
        staging_failure_count: prepared_upload.staging.failures.len(),
        staged_upload_failure_count: prepared_upload.staged_uploads.failures.len(),
        skipped_staged_upload_failure_count: request_plan.skipped_failure_count,
        upload_byte_len,
        ready_to_write_texture: false,
    };

    GlyphAtlasBitmapTextureUploadFrameReport {
        ready_to_write_texture: report.binding_count > 0 && !report.has_failures(),
        ..report
    }
}
