use std::sync::Arc;

use zr_rhi_wgpu::{WgpuTextureUpload, WgpuTextureUploadBatch};

use crate::text::atlas::{
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapPageShadowCommit,
    GlyphAtlasBitmapPreparedUploadPlan, GlyphAtlasBitmapRequeueReason, GlyphAtlasBitmapRunPlan,
    GlyphAtlasBitmapTextureUploadRequestPlan, GlyphAtlasFormat, GlyphAtlasSet,
    glyph_atlas_bitmap_page_shadow_commit, glyph_atlas_bitmap_texture_upload_request_plan,
    glyph_atlas_bitmap_texture_upload_request_plan_with_atlas,
    glyph_atlas_bitmap_texture_upload_request_plan_with_atlas_and_face_validity,
};

use super::binding::{
    GlyphAtlasBitmapTextureUploadBindingPlan, glyph_atlas_bitmap_texture_upload_binding_plan,
};
use super::write::{glyph_atlas_texture_upload_region, glyph_atlas_texture_upload_source_range};

const GLYPH_ATLAS_BITMAP_RESOURCE_FORMAT_COUNT: usize = 5;

/// Fixed lookup for the finite atlas format set. Duplicate resources are unavailable so a frame
/// cannot silently choose a target texture by iteration order.
pub(super) struct GlyphAtlasBitmapTextureUploadResourceTable<T> {
    resources: [Option<T>; GLYPH_ATLAS_BITMAP_RESOURCE_FORMAT_COUNT],
    duplicate_formats: [bool; GLYPH_ATLAS_BITMAP_RESOURCE_FORMAT_COUNT],
}

impl<T> GlyphAtlasBitmapTextureUploadResourceTable<T> {
    pub(super) fn from_resources(
        resources: impl IntoIterator<Item = (GlyphAtlasFormat, T)>,
    ) -> Self {
        let mut table = Self {
            resources: std::array::from_fn(|_| None),
            duplicate_formats: [false; GLYPH_ATLAS_BITMAP_RESOURCE_FORMAT_COUNT],
        };
        for (format, resource) in resources {
            let index = glyph_atlas_bitmap_resource_format_index(format);
            if table.resources[index].replace(resource).is_some() {
                table.duplicate_formats[index] = true;
            }
        }
        table
    }

    pub(super) fn resource(&self, format: GlyphAtlasFormat) -> Option<&T> {
        let index = glyph_atlas_bitmap_resource_format_index(format);
        (!self.duplicate_formats[index])
            .then(|| self.resources[index].as_ref())
            .flatten()
    }
}

fn glyph_atlas_bitmap_resource_format_index(format: GlyphAtlasFormat) -> usize {
    match format {
        GlyphAtlasFormat::AlphaMask => 0,
        GlyphAtlasFormat::SubpixelMask => 1,
        GlyphAtlasFormat::Sdf => 2,
        GlyphAtlasFormat::Msdf => 3,
        GlyphAtlasFormat::Color => 4,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapTextureUploadFrameReport {
    pub(in crate::graphics::scene::scene_renderer::ui) request_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) binding_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) binding_failure_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) staging_failure_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) staged_upload_failure_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) skipped_staged_upload_failure_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) requeued_upload_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) missing_page_requeue_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) page_generation_mismatch_requeue_count:
        usize,
    pub(in crate::graphics::scene::scene_renderer::ui) stale_page_generation_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui) face_invalidated_count: usize,
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
            || self.stale_page_generation_count > 0
            || self.face_invalidated_count > 0
            || self.requeued_upload_count > 0
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

pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapPreparedTextureUpload {
    texture_uploads: WgpuTextureUploadBatch,
    shadow_commit: GlyphAtlasBitmapPageShadowCommit,
    report: GlyphAtlasBitmapTextureUploadFrameReport,
}

impl GlyphAtlasBitmapPreparedTextureUpload {
    pub(in crate::graphics::scene::scene_renderer::ui) fn into_parts(
        self,
    ) -> (
        WgpuTextureUploadBatch,
        GlyphAtlasBitmapPageShadowCommit,
        GlyphAtlasBitmapTextureUploadFrameReport,
    ) {
        (self.texture_uploads, self.shadow_commit, self.report)
    }
}

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_bitmap_texture_upload_frame_plan<
    'a,
>(
    prepared_upload: &'a GlyphAtlasBitmapPreparedUploadPlan,
) -> GlyphAtlasBitmapTextureUploadFramePlan<'a> {
    let request_plan =
        glyph_atlas_bitmap_texture_upload_request_plan(&prepared_upload.staged_uploads);
    glyph_atlas_bitmap_texture_upload_frame_plan_from_requests(prepared_upload, request_plan)
}

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas<
    'a,
>(
    prepared_upload: &'a GlyphAtlasBitmapPreparedUploadPlan,
    atlas: &GlyphAtlasSet,
) -> GlyphAtlasBitmapTextureUploadFramePlan<'a> {
    let request_plan = glyph_atlas_bitmap_texture_upload_request_plan_with_atlas(
        &prepared_upload.staged_uploads,
        atlas,
    );
    glyph_atlas_bitmap_texture_upload_frame_plan_from_requests(prepared_upload, request_plan)
}

pub(in crate::graphics::scene::scene_renderer::ui) fn glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity<
    'a,
>(
    prepared_upload: &'a GlyphAtlasBitmapPreparedUploadPlan,
    atlas: &GlyphAtlasSet,
    face_validity: GlyphAtlasBitmapFaceValidity,
) -> GlyphAtlasBitmapTextureUploadFramePlan<'a> {
    let request_plan = glyph_atlas_bitmap_texture_upload_request_plan_with_atlas_and_face_validity(
        &prepared_upload.staged_uploads,
        atlas,
        face_validity,
    );
    glyph_atlas_bitmap_texture_upload_frame_plan_from_requests(prepared_upload, request_plan)
}

fn glyph_atlas_bitmap_texture_upload_frame_plan_from_requests<'a>(
    prepared_upload: &'a GlyphAtlasBitmapPreparedUploadPlan,
    request_plan: GlyphAtlasBitmapTextureUploadRequestPlan,
) -> GlyphAtlasBitmapTextureUploadFramePlan<'a> {
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

pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_glyph_atlas_bitmap_texture_upload_for_resources(
    resources: impl IntoIterator<Item = (GlyphAtlasFormat, wgpu::Texture)>,
    run: &GlyphAtlasBitmapRunPlan,
    prepared_upload: GlyphAtlasBitmapPreparedUploadPlan,
    face_validity: GlyphAtlasBitmapFaceValidity,
) -> GlyphAtlasBitmapPreparedTextureUpload {
    let plan = glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity(
        &prepared_upload,
        &run.atlas,
        face_validity,
    );
    let mut report = plan.report;
    if !plan.ready_to_write_texture() {
        drop(plan);
        return GlyphAtlasBitmapPreparedTextureUpload {
            texture_uploads: WgpuTextureUploadBatch::new(),
            shadow_commit: glyph_atlas_bitmap_page_shadow_commit(run, prepared_upload, false),
            report,
        };
    }

    let resources = GlyphAtlasBitmapTextureUploadResourceTable::from_resources(resources);
    let mut missing_resource_count = 0_usize;
    for binding in &plan.binding_plan.bindings {
        let Some(request) = plan.request_plan.requests.get(binding.request_index) else {
            missing_resource_count = missing_resource_count.saturating_add(1);
            continue;
        };
        if resources.resource(request.page_key.format).is_none() {
            missing_resource_count = missing_resource_count.saturating_add(1);
        }
    }
    if missing_resource_count > 0 {
        report.binding_failure_count = report
            .binding_failure_count
            .saturating_add(missing_resource_count);
        report.ready_to_write_texture = false;
        drop(plan);
        return GlyphAtlasBitmapPreparedTextureUpload {
            texture_uploads: WgpuTextureUploadBatch::new(),
            shadow_commit: glyph_atlas_bitmap_page_shadow_commit(run, prepared_upload, false),
            report,
        };
    }

    let mut texture_uploads = WgpuTextureUploadBatch::new();
    for binding in &plan.binding_plan.bindings {
        let Some(request) = plan.request_plan.requests.get(binding.request_index) else {
            continue;
        };
        let Some(texture) = resources.resource(request.page_key.format) else {
            continue;
        };
        let Some(staging_page) = prepared_upload
            .staging
            .pages
            .get(request.staging_page_index)
        else {
            continue;
        };
        let Some(source_range) =
            glyph_atlas_texture_upload_source_range(binding.write, request.upload_byte_len)
        else {
            report.binding_failure_count = report.binding_failure_count.saturating_add(1);
            report.ready_to_write_texture = false;
            break;
        };
        let Some(upload) = WgpuTextureUpload::new(
            texture.clone(),
            glyph_atlas_texture_upload_region(binding.write),
            binding.write.bytes_per_row,
            binding.write.rows_per_image,
            Arc::clone(&staging_page.bytes),
            source_range,
        ) else {
            report.binding_failure_count = report.binding_failure_count.saturating_add(1);
            report.ready_to_write_texture = false;
            break;
        };
        texture_uploads.push(upload);
    }
    drop(plan);
    if !report.ready_to_write_texture {
        texture_uploads = WgpuTextureUploadBatch::new();
    }
    let shadow_commit =
        glyph_atlas_bitmap_page_shadow_commit(run, prepared_upload, report.ready_to_write_texture);
    GlyphAtlasBitmapPreparedTextureUpload {
        texture_uploads,
        shadow_commit,
        report,
    }
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
        requeued_upload_count: request_plan.requeued_uploads.len(),
        missing_page_requeue_count: requeue_reason_count(
            request_plan,
            GlyphAtlasBitmapRequeueReason::MissingPage,
        ),
        page_generation_mismatch_requeue_count: requeue_reason_count(
            request_plan,
            GlyphAtlasBitmapRequeueReason::PageGenerationMismatch,
        ),
        stale_page_generation_count: request_plan.stale_page_generation_count,
        face_invalidated_count: request_plan.face_invalidated_count,
        upload_byte_len,
        ready_to_write_texture: false,
    };

    GlyphAtlasBitmapTextureUploadFrameReport {
        ready_to_write_texture: report.binding_count > 0 && !report.has_failures(),
        ..report
    }
}

fn requeue_reason_count(
    request_plan: &GlyphAtlasBitmapTextureUploadRequestPlan,
    reason: GlyphAtlasBitmapRequeueReason,
) -> usize {
    request_plan
        .requeued_uploads
        .iter()
        .filter(|requeue| requeue.reason == reason)
        .count()
}
