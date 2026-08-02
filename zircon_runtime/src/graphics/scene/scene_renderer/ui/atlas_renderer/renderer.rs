use wgpu::util::DeviceExt;

use crate::core::math::UVec2;
use crate::text::atlas::render_gpu_plan::{
    glyph_atlas_gpu_bind_group_layout, GlyphAtlasGpuDrawPlan, GlyphAtlasGpuPipelineContract,
    GlyphAtlasGpuPipelineKey, GlyphAtlasGpuViewportTransform,
};
use crate::text::atlas::{
    glyph_atlas_bitmap_page_shadow_commit, GlyphAtlasBitmapFaceValidity,
    GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapPreparedUploadPlan,
    GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasFormat,
    GlyphAtlasSet, GlyphAtlasStorageFormat, GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT,
};

use super::super::atlas_texture_upload::{
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity,
    write_glyph_atlas_bitmap_texture_upload_frame_plan, GlyphAtlasBitmapTextureUploadFramePlan,
    GlyphAtlasBitmapTextureUploadFrameReport,
};
use super::instance_buffer::glyph_atlas_bitmap_renderer_write_instance_buffer;
use super::pipeline::{
    create_glyph_atlas_bitmap_pipeline, create_glyph_atlas_bitmap_pipeline_layout,
    create_glyph_atlas_bitmap_shader,
};
use super::resources::{
    create_glyph_atlas_bitmap_atlas_resources, create_glyph_atlas_bitmap_bind_group_layout,
    create_glyph_atlas_bitmap_sampler,
};
use super::state::{
    GlyphAtlasBitmapPipelineResource, GlyphAtlasBitmapRendererAtlasResource,
    GlyphAtlasBitmapRendererDrawPass, GlyphAtlasBitmapRendererPrepareReport,
    GlyphAtlasBitmapRendererStorageSubmission,
};

pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapRenderer {
    target_format: wgpu::TextureFormat,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    viewport_uniform_buffer: wgpu::Buffer,
    atlas_resources: Vec<GlyphAtlasBitmapRendererAtlasResource>,
    draw_passes: Vec<GlyphAtlasBitmapRendererDrawPass>,
    pipeline_resources: Vec<GlyphAtlasBitmapPipelineResource>,
    pending_invalidated_storage_pass_count: usize,
    last_report: GlyphAtlasBitmapRendererPrepareReport,
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
        let viewport_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-screen-space-ui-glyph-atlas-viewport"),
                contents: bytemuck::cast_slice(
                    &GlyphAtlasGpuViewportTransform::default().uniform_bytes(),
                ),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let atlas_format = GlyphAtlasFormat::AlphaMask;
        let atlas = create_glyph_atlas_bitmap_atlas_resources(
            device,
            &bind_group_layout,
            &sampler,
            &viewport_uniform_buffer,
            UVec2::new(1, 1),
            1,
            atlas_format.storage_format(),
        );

        Self {
            target_format,
            shader,
            pipeline_layout,
            sampler,
            bind_group_layout,
            viewport_uniform_buffer,
            atlas_resources: vec![GlyphAtlasBitmapRendererAtlasResource::new(
                atlas_format,
                atlas,
            )],
            draw_passes: vec![GlyphAtlasBitmapRendererDrawPass::new(atlas_format)],
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
        queue: &wgpu::Queue,
        plan: &GlyphAtlasGpuDrawPlan,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_storage_format: GlyphAtlasStorageFormat,
    ) {
        self.write_viewport_uniform(queue, plan.viewport_transform);
        let mut report = self.prepare_storage_pass(
            device,
            queue,
            plan,
            atlas_size,
            atlas_layer_count,
            glyph_atlas_bitmap_renderer_default_format_for_storage(atlas_storage_format),
            0,
        );
        report = self.report_with_pending_face_invalidation(report);
        self.draw_passes.truncate(1);
        self.last_report = report;
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_idle(&mut self) {
        glyph_atlas_bitmap_renderer_release_idle_draw_passes(&mut self.draw_passes);
        let report = glyph_atlas_bitmap_renderer_idle_prepare_report(
            self.atlas_resources.iter().map(|resource| {
                (
                    resource.atlas.size(),
                    resource.atlas.layer_count(),
                    resource.atlas.storage_format(),
                )
            }),
            self.pipeline_resources.len(),
        );
        self.last_report = self.report_with_pending_face_invalidation(report);
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_submission<'a, I>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        submission: &GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: I,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_format: GlyphAtlasFormat,
    ) -> GlyphAtlasBitmapPageShadowCommit
    where
        I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
    {
        self.prepare_submission_with_face_validity(
            device,
            queue,
            submission,
            source_bytes,
            atlas_size,
            atlas_layer_count,
            atlas_format,
            GlyphAtlasBitmapFaceValidity::Valid,
        )
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
        atlas_format: GlyphAtlasFormat,
        face_validity: GlyphAtlasBitmapFaceValidity,
    ) -> GlyphAtlasBitmapPageShadowCommit
    where
        I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
    {
        self.write_viewport_uniform(queue, submission.gpu_draw.viewport_transform);
        let mut report = self.prepare_storage_pass(
            device,
            queue,
            &submission.gpu_draw,
            atlas_size,
            atlas_layer_count,
            atlas_format,
            0,
        );
        self.draw_passes.truncate(1);
        let prepared_upload = submission.prepared_upload(source_bytes);
        let upload_frame = glyph_atlas_bitmap_renderer_texture_upload_frame_plan(
            &prepared_upload,
            &submission.run.atlas,
            face_validity,
        );
        let upload_report = self.write_upload_frame(queue, atlas_format, &upload_frame);
        drop(upload_frame);
        let shadow_commit = glyph_atlas_bitmap_page_shadow_commit(
            &submission.run,
            prepared_upload,
            upload_report.ready_to_write_texture,
        );
        report = report.with_upload_report(upload_report);
        report = self.report_with_pending_face_invalidation(report);
        self.last_report = report;
        shadow_commit
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_storage_submissions<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        submissions: &[GlyphAtlasBitmapRendererStorageSubmission<'a>],
        atlas_size: UVec2,
    ) -> GlyphAtlasBitmapPageShadowCommit {
        if submissions.is_empty() {
            self.prepare_idle();
            return GlyphAtlasBitmapPageShadowCommit::default();
        }

        let mut pass_reports = Vec::with_capacity(submissions.len());
        let mut shadow_commit = GlyphAtlasBitmapPageShadowCommit::default();
        self.write_viewport_uniform(queue, submissions[0].submission.gpu_draw.viewport_transform);
        for (pass_index, submission) in submissions.iter().enumerate() {
            debug_assert_eq!(
                submission.submission.gpu_draw.viewport_transform,
                submissions[0].submission.gpu_draw.viewport_transform,
                "mixed storage submissions must share the UI viewport uniform",
            );
            let mut report = self.prepare_storage_pass(
                device,
                queue,
                &submission.submission.gpu_draw,
                atlas_size,
                submission.atlas_layer_count,
                submission.atlas_format,
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
            let upload_report =
                self.write_upload_frame(queue, submission.atlas_format, &upload_frame);
            drop(upload_frame);
            shadow_commit.extend(glyph_atlas_bitmap_page_shadow_commit(
                &submission.submission.run,
                prepared_upload,
                upload_report.ready_to_write_texture,
            ));
            report = report.with_upload_report(upload_report);
            pass_reports.push(report);
        }

        self.draw_passes.truncate(submissions.len());
        let report = glyph_atlas_bitmap_renderer_prepare_report_for_storage_passes(
            &pass_reports,
            self.pipeline_resources.len(),
        );
        self.last_report = self.report_with_pending_face_invalidation(report);
        shadow_commit
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
        for draw_pass in &self.draw_passes {
            let Some(instance_buffer) = draw_pass.instance_buffer.as_ref() else {
                continue;
            };
            if draw_pass.draw_commands.is_empty() {
                continue;
            }
            let Some(atlas_resource) = self.atlas_resource(draw_pass.atlas_format) else {
                continue;
            };

            pass.set_bind_group(0, atlas_resource.atlas.bind_group(), &[]);
            pass.set_vertex_buffer(0, instance_buffer.slice(..));
            for command in &draw_pass.draw_commands {
                let Some(pipeline) = self.pipeline(command.pipeline_key) else {
                    continue;
                };
                pass.set_pipeline(pipeline);
                pass.draw(
                    0..6,
                    command.instance_start..command.instance_start + command.instance_count,
                );
            }
        }
    }

    fn prepare_storage_pass(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &GlyphAtlasGpuDrawPlan,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_format: GlyphAtlasFormat,
        pass_index: usize,
    ) -> GlyphAtlasBitmapRendererPrepareReport {
        self.ensure_pipelines(device, &plan.pipeline_contracts);
        let atlas_resized =
            self.ensure_atlas_resources(device, atlas_size, atlas_layer_count, atlas_format);
        self.ensure_draw_pass(pass_index, atlas_format);
        let draw_pass = &mut self.draw_passes[pass_index];
        let (instance_buffer_capacity_byte_len, instance_buffer_reallocation_count) =
            glyph_atlas_bitmap_renderer_write_instance_buffer(
                device,
                queue,
                draw_pass,
                plan.instances.as_slice(),
            );
        draw_pass.draw_commands = plan.draw_commands.clone();

        let mut report = glyph_atlas_bitmap_renderer_prepare_report(
            plan,
            self.pipeline_resources.len(),
            atlas_size,
            atlas_layer_count,
            atlas_format.storage_format(),
            atlas_resized,
        );
        report.instance_buffer_capacity_byte_len = instance_buffer_capacity_byte_len;
        report.instance_buffer_reallocation_count = instance_buffer_reallocation_count;
        report
    }

    fn ensure_atlas_resources(
        &mut self,
        device: &wgpu::Device,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_format: GlyphAtlasFormat,
    ) -> bool {
        let atlas_storage_format = atlas_format.storage_format();
        if self.atlas_resource(atlas_format).is_some_and(|resource| {
            resource
                .atlas
                .supports(atlas_size, atlas_layer_count, atlas_storage_format)
        }) {
            return false;
        }

        let atlas_layer_capacity =
            glyph_atlas_bitmap_renderer_atlas_layer_capacity(atlas_layer_count);
        let atlas = create_glyph_atlas_bitmap_atlas_resources(
            device,
            &self.bind_group_layout,
            &self.sampler,
            &self.viewport_uniform_buffer,
            atlas_size,
            atlas_layer_capacity,
            atlas_storage_format,
        );
        if let Some(resource) = self.atlas_resource_mut(atlas_format) {
            resource.atlas = atlas;
        } else {
            self.atlas_resources
                .push(GlyphAtlasBitmapRendererAtlasResource::new(
                    atlas_format,
                    atlas,
                ));
        }
        true
    }

    fn ensure_draw_pass(&mut self, pass_index: usize, atlas_format: GlyphAtlasFormat) {
        if self.draw_passes.len() == pass_index {
            self.draw_passes
                .push(GlyphAtlasBitmapRendererDrawPass::new(atlas_format));
            return;
        }
        self.draw_passes[pass_index].atlas_format = atlas_format;
    }

    fn atlas_resource(
        &self,
        atlas_format: GlyphAtlasFormat,
    ) -> Option<&GlyphAtlasBitmapRendererAtlasResource> {
        self.atlas_resources
            .iter()
            .find(|resource| resource.atlas_format == atlas_format)
    }

    fn atlas_resource_mut(
        &mut self,
        atlas_format: GlyphAtlasFormat,
    ) -> Option<&mut GlyphAtlasBitmapRendererAtlasResource> {
        self.atlas_resources
            .iter_mut()
            .find(|resource| resource.atlas_format == atlas_format)
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
        let invalidated_storage_pass_count = self.clear_draw_state();
        self.draw_passes.truncate(1);
        invalidated_storage_pass_count
    }

    fn clear_draw_state(&mut self) -> usize {
        let active_storage_pass_count = self
            .draw_passes
            .iter()
            .filter(|pass| !pass.draw_commands.is_empty())
            .count();
        for pass in &mut self.draw_passes {
            pass.draw_commands.clear();
        }
        active_storage_pass_count
    }

    fn write_viewport_uniform(
        &self,
        queue: &wgpu::Queue,
        viewport_transform: GlyphAtlasGpuViewportTransform,
    ) {
        queue.write_buffer(
            &self.viewport_uniform_buffer,
            0,
            bytemuck::cast_slice(&viewport_transform.uniform_bytes()),
        );
    }

    fn write_upload_frame(
        &self,
        queue: &wgpu::Queue,
        atlas_format: GlyphAtlasFormat,
        upload_frame: &GlyphAtlasBitmapTextureUploadFramePlan<'_>,
    ) -> GlyphAtlasBitmapTextureUploadFrameReport {
        self.atlas_resource(atlas_format).map_or_else(
            || {
                glyph_atlas_bitmap_renderer_upload_report_without_atlas_resource(
                    upload_frame.report(),
                )
            },
            |resource| {
                write_glyph_atlas_bitmap_texture_upload_frame_plan(
                    queue,
                    resource.atlas.texture(),
                    upload_frame,
                )
            },
        )
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

pub(super) fn glyph_atlas_bitmap_renderer_release_idle_draw_passes(
    draw_passes: &mut Vec<GlyphAtlasBitmapRendererDrawPass>,
) {
    // Idle is the explicit reclamation boundary; stable non-idle frames retain capacity instead.
    draw_passes.truncate(1);
    if let Some(draw_pass) = draw_passes.first_mut() {
        draw_pass.instance_buffer = None;
        draw_pass.instance_buffer_capacity_bytes = 0;
        draw_pass.draw_commands.clear();
    }
}

impl GlyphAtlasBitmapRendererPrepareReport {
    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::ui) fn with_upload_counters_for_test(
        mut self,
        request_count: usize,
        upload_byte_len: usize,
        requeued_count: usize,
        failure_count: usize,
        ready_to_write_texture: bool,
    ) -> Self {
        self.upload_request_count = request_count;
        self.upload_byte_len = upload_byte_len;
        self.upload_requeued_count = requeued_count;
        self.upload_failure_count = failure_count;
        self.upload_ready_to_write_texture = ready_to_write_texture;
        self
    }

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

fn glyph_atlas_bitmap_renderer_default_format_for_storage(
    storage_format: GlyphAtlasStorageFormat,
) -> GlyphAtlasFormat {
    match storage_format {
        GlyphAtlasStorageFormat::R8Unorm => GlyphAtlasFormat::AlphaMask,
        GlyphAtlasStorageFormat::Rgba8Unorm => GlyphAtlasFormat::Color,
    }
}

pub(super) fn glyph_atlas_bitmap_renderer_atlas_layer_capacity(required_layer_count: u32) -> u32 {
    let maximum_resident_layers =
        u32::try_from(GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT).unwrap_or(u32::MAX);
    required_layer_count.max(maximum_resident_layers).max(1)
}

pub(super) fn glyph_atlas_bitmap_renderer_prepare_report_with_upload_report(
    report: GlyphAtlasBitmapRendererPrepareReport,
    upload: GlyphAtlasBitmapTextureUploadFrameReport,
) -> GlyphAtlasBitmapRendererPrepareReport {
    report.with_upload_report(upload)
}

pub(super) fn glyph_atlas_bitmap_renderer_upload_report_without_atlas_resource(
    mut upload: GlyphAtlasBitmapTextureUploadFrameReport,
) -> GlyphAtlasBitmapTextureUploadFrameReport {
    upload.binding_failure_count = upload
        .binding_failure_count
        .saturating_add(upload.request_count);
    upload.ready_to_write_texture = false;
    upload
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
        vertex_count: plan.vertex_count(),
        vertex_buffer_byte_len: std::mem::size_of_val(plan.instances.as_slice()),
        instance_buffer_capacity_byte_len: 0,
        instance_buffer_reallocation_count: 0,
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

pub(super) fn glyph_atlas_bitmap_renderer_idle_prepare_report(
    storage_passes: impl IntoIterator<Item = (UVec2, u32, GlyphAtlasStorageFormat)>,
    pipeline_count: usize,
) -> GlyphAtlasBitmapRendererPrepareReport {
    let mut storage_passes = storage_passes.into_iter();
    let Some((atlas_size, first_layer_count, atlas_storage_format)) = storage_passes.next() else {
        return GlyphAtlasBitmapRendererPrepareReport::default();
    };
    let mut atlas_layer_count = first_layer_count.max(1);
    let mut storage_pass_count = 1;
    let mut mixed_atlas_storage_format = false;
    for (_, layer_count, storage_format) in storage_passes {
        atlas_layer_count = atlas_layer_count.saturating_add(layer_count.max(1));
        storage_pass_count += 1;
        mixed_atlas_storage_format |= storage_format != atlas_storage_format;
    }

    GlyphAtlasBitmapRendererPrepareReport {
        atlas_size,
        atlas_layer_count,
        atlas_storage_format,
        storage_pass_count,
        mixed_atlas_storage_format,
        pipeline_count,
        ..GlyphAtlasBitmapRendererPrepareReport::default()
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
        instance_buffer_capacity_byte_len: reports
            .iter()
            .map(|report| report.instance_buffer_capacity_byte_len)
            .sum(),
        instance_buffer_reallocation_count: reports
            .iter()
            .map(|report| report.instance_buffer_reallocation_count)
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
