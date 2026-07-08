use wgpu::util::DeviceExt;

use crate::core::math::UVec2;
use crate::graphics::text::atlas::render_gpu_plan::{
    glyph_atlas_gpu_bind_group_layout, GlyphAtlasGpuDrawCommand, GlyphAtlasGpuDrawPlan,
    GlyphAtlasGpuPipelineContract, GlyphAtlasGpuPipelineKey,
};
use crate::graphics::text::atlas::{
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapPreparedUploadPlan,
    GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasSet,
    GlyphAtlasStorageFormat,
};

use super::super::atlas_texture_upload::{
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity,
    write_glyph_atlas_bitmap_texture_upload_frame_plan, GlyphAtlasBitmapTextureUploadFramePlan,
    GlyphAtlasBitmapTextureUploadFrameReport,
};
use super::pipeline::{
    create_glyph_atlas_bitmap_pipeline, create_glyph_atlas_bitmap_pipeline_layout,
    create_glyph_atlas_bitmap_shader,
};
use super::resources::{
    create_glyph_atlas_bitmap_atlas_resources, create_glyph_atlas_bitmap_bind_group_layout,
    create_glyph_atlas_bitmap_sampler, GlyphAtlasBitmapAtlasResources,
};

pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapRenderer {
    target_format: wgpu::TextureFormat,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    storage_passes: Vec<GlyphAtlasBitmapRendererStoragePass>,
    pipeline_resources: Vec<GlyphAtlasBitmapPipelineResource>,
    pending_invalidated_storage_pass_count: usize,
    last_report: GlyphAtlasBitmapRendererPrepareReport,
}

pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapRendererStorageSubmission<
    'a,
> {
    submission: &'a GlyphAtlasBitmapRenderSubmissionPlan,
    source_bytes: Vec<GlyphAtlasBitmapUploadSourceBytes<'a>>,
    atlas_layer_count: u32,
    atlas_storage_format: GlyphAtlasStorageFormat,
    face_validity: GlyphAtlasBitmapFaceValidity,
}

struct GlyphAtlasBitmapRendererStoragePass {
    atlas: GlyphAtlasBitmapAtlasResources,
    vertex_buffer: Option<wgpu::Buffer>,
    draw_commands: Vec<GlyphAtlasGpuDrawCommand>,
}

struct GlyphAtlasBitmapPipelineResource {
    key: GlyphAtlasGpuPipelineKey,
    pipeline: wgpu::RenderPipeline,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapRendererPrepareReport {
    pub(super) atlas_size: UVec2,
    pub(super) atlas_layer_count: u32,
    pub(super) atlas_storage_format: GlyphAtlasStorageFormat,
    pub(super) storage_pass_count: usize,
    pub(super) storage_pass_visible_glyph_count: usize,
    pub(super) mixed_atlas_storage_format: bool,
    pub(super) atlas_resized: bool,
    pub(super) vertex_count: usize,
    pub(super) vertex_buffer_byte_len: usize,
    pub(super) draw_command_count: usize,
    pub(super) pipeline_count: usize,
    pub(super) requires_background_composite: bool,
    pub(super) upload_request_count: usize,
    pub(super) upload_requeued_count: usize,
    pub(super) upload_missing_page_requeue_count: usize,
    pub(super) upload_page_generation_mismatch_requeue_count: usize,
    pub(super) upload_face_invalidated_count: usize,
    pub(super) upload_byte_len: usize,
    pub(super) upload_ready_to_write_texture: bool,
    pub(super) upload_failure_count: usize,
    pub(super) invalidated_storage_pass_count: usize,
}

impl Default for GlyphAtlasBitmapRendererPrepareReport {
    fn default() -> Self {
        Self {
            atlas_size: UVec2::new(1, 1),
            atlas_layer_count: 1,
            atlas_storage_format: GlyphAtlasStorageFormat::R8Unorm,
            storage_pass_count: 0,
            storage_pass_visible_glyph_count: 0,
            mixed_atlas_storage_format: false,
            atlas_resized: false,
            vertex_count: 0,
            vertex_buffer_byte_len: 0,
            draw_command_count: 0,
            pipeline_count: 0,
            requires_background_composite: false,
            upload_request_count: 0,
            upload_requeued_count: 0,
            upload_missing_page_requeue_count: 0,
            upload_page_generation_mismatch_requeue_count: 0,
            upload_face_invalidated_count: 0,
            upload_byte_len: 0,
            upload_ready_to_write_texture: false,
            upload_failure_count: 0,
            invalidated_storage_pass_count: 0,
        }
    }
}

impl<'a> GlyphAtlasBitmapRendererStorageSubmission<'a> {
    pub(in crate::graphics::scene::scene_renderer::ui) fn new(
        submission: &'a GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: Vec<GlyphAtlasBitmapUploadSourceBytes<'a>>,
        atlas_layer_count: u32,
        atlas_storage_format: GlyphAtlasStorageFormat,
    ) -> Self {
        Self::new_with_face_validity(
            submission,
            source_bytes,
            atlas_layer_count,
            atlas_storage_format,
            GlyphAtlasBitmapFaceValidity::Valid,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn new_with_face_validity(
        submission: &'a GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: Vec<GlyphAtlasBitmapUploadSourceBytes<'a>>,
        atlas_layer_count: u32,
        atlas_storage_format: GlyphAtlasStorageFormat,
        face_validity: GlyphAtlasBitmapFaceValidity,
    ) -> Self {
        Self {
            submission,
            source_bytes,
            atlas_layer_count,
            atlas_storage_format,
            face_validity,
        }
    }
}

impl GlyphAtlasBitmapRendererStoragePass {
    fn new(atlas: GlyphAtlasBitmapAtlasResources) -> Self {
        Self {
            atlas,
            vertex_buffer: None,
            draw_commands: Vec::new(),
        }
    }
}

impl GlyphAtlasBitmapRenderer {
    pub(in crate::graphics::scene::scene_renderer::ui) fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let bind_group_layout = create_glyph_atlas_bitmap_bind_group_layout(
            device,
            glyph_atlas_gpu_bind_group_layout(),
        );
        let sampler = create_glyph_atlas_bitmap_sampler(device);
        let pipeline_layout = create_glyph_atlas_bitmap_pipeline_layout(device, &bind_group_layout);
        let shader = create_glyph_atlas_bitmap_shader(device);
        let atlas = create_glyph_atlas_bitmap_atlas_resources(
            device,
            &bind_group_layout,
            &sampler,
            UVec2::new(1, 1),
            1,
            GlyphAtlasStorageFormat::R8Unorm,
        );

        Self {
            target_format,
            shader,
            pipeline_layout,
            sampler,
            bind_group_layout,
            storage_passes: vec![GlyphAtlasBitmapRendererStoragePass::new(atlas)],
            pipeline_resources: Vec::new(),
            pending_invalidated_storage_pass_count: 0,
            last_report: GlyphAtlasBitmapRendererPrepareReport::default(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn discard_all_for_face_invalidation(
        &mut self,
    ) {
        let invalidated_storage_pass_count = self.clear_active_storage_passes();
        self.pending_invalidated_storage_pass_count = self
            .pending_invalidated_storage_pass_count
            .saturating_add(invalidated_storage_pass_count);
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_plan(
        &mut self,
        device: &wgpu::Device,
        plan: &GlyphAtlasGpuDrawPlan,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_storage_format: GlyphAtlasStorageFormat,
    ) {
        let mut report = self.prepare_storage_pass(
            device,
            plan,
            atlas_size,
            atlas_layer_count,
            atlas_storage_format,
            0,
        );
        report = self.report_with_pending_face_invalidation(report);
        self.storage_passes.truncate(1);
        self.last_report = report;
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_submission<'a, I>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        submission: &GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: I,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_storage_format: GlyphAtlasStorageFormat,
    ) where
        I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
    {
        self.prepare_submission_with_face_validity(
            device,
            queue,
            submission,
            source_bytes,
            atlas_size,
            atlas_layer_count,
            atlas_storage_format,
            GlyphAtlasBitmapFaceValidity::Valid,
        );
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_submission_with_face_validity<
        'a,
        I,
    >(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        submission: &GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: I,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_storage_format: GlyphAtlasStorageFormat,
        face_validity: GlyphAtlasBitmapFaceValidity,
    ) where
        I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
    {
        let mut report = self.prepare_storage_pass(
            device,
            &submission.gpu_draw,
            atlas_size,
            atlas_layer_count,
            atlas_storage_format,
            0,
        );
        self.storage_passes.truncate(1);
        let prepared_upload = submission.prepared_upload(source_bytes);
        let upload_frame = glyph_atlas_bitmap_renderer_texture_upload_frame_plan(
            &prepared_upload,
            &submission.run.atlas,
            face_validity,
        );
        let upload_report = write_glyph_atlas_bitmap_texture_upload_frame_plan(
            queue,
            self.storage_passes[0].atlas.texture(),
            &upload_frame,
        );
        report = report.with_upload_report(upload_report);
        report = self.report_with_pending_face_invalidation(report);
        self.last_report = report;
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_storage_submissions<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        submissions: &[GlyphAtlasBitmapRendererStorageSubmission<'a>],
        atlas_size: UVec2,
    ) {
        if submissions.is_empty() {
            self.prepare_plan(
                device,
                &GlyphAtlasGpuDrawPlan::default(),
                UVec2::new(1, 1),
                1,
                GlyphAtlasStorageFormat::R8Unorm,
            );
            return;
        }

        let mut pass_reports = Vec::with_capacity(submissions.len());
        for (pass_index, submission) in submissions.iter().enumerate() {
            let mut report = self.prepare_storage_pass(
                device,
                &submission.submission.gpu_draw,
                atlas_size,
                submission.atlas_layer_count,
                submission.atlas_storage_format,
                pass_index,
            );
            let prepared_upload = submission
                .submission
                .prepared_upload(submission.source_bytes.iter().copied());
            let upload_frame = glyph_atlas_bitmap_renderer_texture_upload_frame_plan(
                &prepared_upload,
                &submission.submission.run.atlas,
                submission.face_validity,
            );
            let upload_report = write_glyph_atlas_bitmap_texture_upload_frame_plan(
                queue,
                self.storage_passes[pass_index].atlas.texture(),
                &upload_frame,
            );
            report = report.with_upload_report(upload_report);
            pass_reports.push(report);
        }

        self.storage_passes.truncate(submissions.len());
        let report = glyph_atlas_bitmap_renderer_prepare_report_for_storage_passes(
            &pass_reports,
            self.pipeline_resources.len(),
        );
        self.last_report = self.report_with_pending_face_invalidation(report);
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_report(
        &self,
    ) -> GlyphAtlasBitmapRendererPrepareReport {
        self.last_report.clone()
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn render<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) {
        for storage_pass in &self.storage_passes {
            let Some(vertex_buffer) = storage_pass.vertex_buffer.as_ref() else {
                continue;
            };
            if storage_pass.draw_commands.is_empty() {
                continue;
            }

            pass.set_bind_group(0, storage_pass.atlas.bind_group(), &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            for command in &storage_pass.draw_commands {
                let Some(pipeline) = self.pipeline(command.pipeline_key) else {
                    continue;
                };
                pass.set_pipeline(pipeline);
                pass.draw(
                    command.vertex_start..command.vertex_start + command.vertex_count,
                    0..1,
                );
            }
        }
    }

    fn prepare_storage_pass(
        &mut self,
        device: &wgpu::Device,
        plan: &GlyphAtlasGpuDrawPlan,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_storage_format: GlyphAtlasStorageFormat,
        pass_index: usize,
    ) -> GlyphAtlasBitmapRendererPrepareReport {
        self.ensure_pipelines(device, &plan.pipeline_contracts);
        let atlas_resized = self.ensure_storage_pass_resources(
            device,
            pass_index,
            atlas_size,
            atlas_layer_count,
            atlas_storage_format,
        );
        let vertex_buffer = (!plan.vertices.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-screen-space-ui-glyph-atlas-vertices"),
                contents: bytemuck::cast_slice(plan.vertices.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
        let storage_pass = &mut self.storage_passes[pass_index];
        storage_pass.vertex_buffer = vertex_buffer;
        storage_pass.draw_commands = plan.draw_commands.clone();

        glyph_atlas_bitmap_renderer_prepare_report(
            plan,
            self.pipeline_resources.len(),
            atlas_size,
            atlas_layer_count,
            atlas_storage_format,
            atlas_resized,
        )
    }

    fn ensure_storage_pass_resources(
        &mut self,
        device: &wgpu::Device,
        pass_index: usize,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_storage_format: GlyphAtlasStorageFormat,
    ) -> bool {
        if self.storage_passes.len() == pass_index {
            let atlas = create_glyph_atlas_bitmap_atlas_resources(
                device,
                &self.bind_group_layout,
                &self.sampler,
                atlas_size,
                atlas_layer_count,
                atlas_storage_format,
            );
            self.storage_passes
                .push(GlyphAtlasBitmapRendererStoragePass::new(atlas));
            return true;
        }

        let storage_pass = &mut self.storage_passes[pass_index];
        if storage_pass
            .atlas
            .matches(atlas_size, atlas_layer_count, atlas_storage_format)
        {
            return false;
        }

        storage_pass.atlas = create_glyph_atlas_bitmap_atlas_resources(
            device,
            &self.bind_group_layout,
            &self.sampler,
            atlas_size,
            atlas_layer_count,
            atlas_storage_format,
        );
        true
    }

    fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        contracts: &[GlyphAtlasGpuPipelineContract],
    ) {
        for contract in contracts {
            if self
                .pipeline_resources
                .iter()
                .any(|resource| resource.key == contract.key)
            {
                continue;
            }
            let pipeline = create_glyph_atlas_bitmap_pipeline(
                device,
                &self.shader,
                &self.pipeline_layout,
                self.target_format,
                *contract,
            );
            self.pipeline_resources
                .push(GlyphAtlasBitmapPipelineResource {
                    key: contract.key,
                    pipeline,
                });
        }
    }

    fn pipeline(&self, key: GlyphAtlasGpuPipelineKey) -> Option<&wgpu::RenderPipeline> {
        self.pipeline_resources
            .iter()
            .find(|resource| resource.key == key)
            .map(|resource| &resource.pipeline)
    }

    fn clear_active_storage_passes(&mut self) -> usize {
        let invalidated_storage_pass_count = self
            .storage_passes
            .iter()
            .filter(|pass| pass.vertex_buffer.is_some() || !pass.draw_commands.is_empty())
            .count();
        for pass in &mut self.storage_passes {
            pass.vertex_buffer = None;
            pass.draw_commands.clear();
        }
        self.storage_passes.truncate(1);
        invalidated_storage_pass_count
    }

    fn report_with_pending_face_invalidation(
        &mut self,
        report: GlyphAtlasBitmapRendererPrepareReport,
    ) -> GlyphAtlasBitmapRendererPrepareReport {
        let invalidated_storage_pass_count = self.pending_invalidated_storage_pass_count;
        self.pending_invalidated_storage_pass_count = 0;
        glyph_atlas_bitmap_renderer_prepare_report_with_face_invalidation(
            report,
            invalidated_storage_pass_count,
        )
    }
}

impl GlyphAtlasBitmapRendererPrepareReport {
    fn with_upload_report(mut self, upload: GlyphAtlasBitmapTextureUploadFrameReport) -> Self {
        self.upload_request_count = upload.request_count;
        self.upload_requeued_count = upload.requeued_upload_count;
        self.upload_missing_page_requeue_count = upload.missing_page_requeue_count;
        self.upload_page_generation_mismatch_requeue_count =
            upload.page_generation_mismatch_requeue_count;
        self.upload_face_invalidated_count = upload.face_invalidated_count;
        self.upload_byte_len = upload.upload_byte_len;
        self.upload_ready_to_write_texture = upload.ready_to_write_texture;
        self.upload_failure_count = upload
            .binding_failure_count
            .saturating_add(upload.staging_failure_count)
            .saturating_add(upload.staged_upload_failure_count)
            .saturating_add(upload.skipped_staged_upload_failure_count)
            .saturating_add(upload.requeued_upload_count);
        self
    }
}

pub(super) fn glyph_atlas_bitmap_renderer_prepare_report_with_upload_report(
    report: GlyphAtlasBitmapRendererPrepareReport,
    upload: GlyphAtlasBitmapTextureUploadFrameReport,
) -> GlyphAtlasBitmapRendererPrepareReport {
    report.with_upload_report(upload)
}

pub(super) fn glyph_atlas_bitmap_renderer_texture_upload_frame_plan<'a>(
    prepared_upload: &'a GlyphAtlasBitmapPreparedUploadPlan,
    atlas: &GlyphAtlasSet,
    face_validity: GlyphAtlasBitmapFaceValidity,
) -> GlyphAtlasBitmapTextureUploadFramePlan<'a> {
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity(
        prepared_upload,
        atlas,
        face_validity,
    )
}

pub(super) fn glyph_atlas_bitmap_renderer_prepare_report_with_face_invalidation(
    mut report: GlyphAtlasBitmapRendererPrepareReport,
    invalidated_storage_pass_count: usize,
) -> GlyphAtlasBitmapRendererPrepareReport {
    report.invalidated_storage_pass_count = report
        .invalidated_storage_pass_count
        .saturating_add(invalidated_storage_pass_count);
    report
}

pub(super) fn glyph_atlas_bitmap_renderer_prepare_report(
    plan: &GlyphAtlasGpuDrawPlan,
    pipeline_count: usize,
    atlas_size: UVec2,
    atlas_layer_count: u32,
    atlas_storage_format: GlyphAtlasStorageFormat,
    atlas_resized: bool,
) -> GlyphAtlasBitmapRendererPrepareReport {
    GlyphAtlasBitmapRendererPrepareReport {
        atlas_size,
        atlas_layer_count: atlas_layer_count.max(1),
        atlas_storage_format,
        storage_pass_count: 1,
        storage_pass_visible_glyph_count: plan.visible_glyph_count,
        mixed_atlas_storage_format: false,
        atlas_resized,
        vertex_count: plan.vertices.len(),
        vertex_buffer_byte_len: std::mem::size_of_val(plan.vertices.as_slice()),
        draw_command_count: plan.draw_commands.len(),
        pipeline_count,
        requires_background_composite: plan.requires_background_composite,
        upload_request_count: 0,
        upload_requeued_count: 0,
        upload_missing_page_requeue_count: 0,
        upload_page_generation_mismatch_requeue_count: 0,
        upload_face_invalidated_count: 0,
        upload_byte_len: 0,
        upload_ready_to_write_texture: false,
        upload_failure_count: 0,
        invalidated_storage_pass_count: 0,
    }
}

pub(super) fn glyph_atlas_bitmap_renderer_prepare_report_for_storage_passes(
    reports: &[GlyphAtlasBitmapRendererPrepareReport],
    pipeline_count: usize,
) -> GlyphAtlasBitmapRendererPrepareReport {
    let Some(first) = reports.first() else {
        return GlyphAtlasBitmapRendererPrepareReport::default();
    };
    let upload_request_count = reports
        .iter()
        .map(|report| report.upload_request_count)
        .sum();
    let upload_requeued_count = reports
        .iter()
        .map(|report| report.upload_requeued_count)
        .sum();
    let upload_missing_page_requeue_count = reports
        .iter()
        .map(|report| report.upload_missing_page_requeue_count)
        .sum();
    let upload_page_generation_mismatch_requeue_count = reports
        .iter()
        .map(|report| report.upload_page_generation_mismatch_requeue_count)
        .sum();
    let upload_face_invalidated_count = reports
        .iter()
        .map(|report| report.upload_face_invalidated_count)
        .sum();
    let upload_failure_count = reports
        .iter()
        .map(|report| report.upload_failure_count)
        .sum();
    let invalidated_storage_pass_count = reports
        .iter()
        .map(|report| report.invalidated_storage_pass_count)
        .sum();
    let upload_ready_to_write_texture = upload_request_count > 0
        && upload_failure_count == 0
        && reports
            .iter()
            .filter(|report| report.upload_request_count > 0)
            .all(|report| report.upload_ready_to_write_texture);

    GlyphAtlasBitmapRendererPrepareReport {
        atlas_size: first.atlas_size,
        atlas_layer_count: reports
            .iter()
            .map(|report| report.atlas_layer_count)
            .sum::<u32>()
            .max(1),
        atlas_storage_format: first.atlas_storage_format,
        storage_pass_count: reports.len(),
        storage_pass_visible_glyph_count: reports
            .iter()
            .map(|report| report.storage_pass_visible_glyph_count)
            .sum(),
        mixed_atlas_storage_format: reports
            .iter()
            .any(|report| report.atlas_storage_format != first.atlas_storage_format),
        atlas_resized: reports.iter().any(|report| report.atlas_resized),
        vertex_count: reports.iter().map(|report| report.vertex_count).sum(),
        vertex_buffer_byte_len: reports
            .iter()
            .map(|report| report.vertex_buffer_byte_len)
            .sum(),
        draw_command_count: reports.iter().map(|report| report.draw_command_count).sum(),
        pipeline_count,
        requires_background_composite: reports
            .iter()
            .any(|report| report.requires_background_composite),
        upload_request_count,
        upload_requeued_count,
        upload_missing_page_requeue_count,
        upload_page_generation_mismatch_requeue_count,
        upload_face_invalidated_count,
        upload_byte_len: reports.iter().map(|report| report.upload_byte_len).sum(),
        upload_ready_to_write_texture,
        upload_failure_count,
        invalidated_storage_pass_count,
    }
}
