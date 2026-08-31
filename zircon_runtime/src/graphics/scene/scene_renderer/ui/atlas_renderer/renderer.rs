use wgpu::util::DeviceExt;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch, WgpuTextureUploadBatch};

use crate::core::math::UVec2;
use crate::text::atlas::render_gpu_plan::{
    GlyphAtlasGpuDrawPlan, GlyphAtlasGpuPipelineContract, GlyphAtlasGpuPipelineKey,
    GlyphAtlasGpuViewportTransform, glyph_atlas_gpu_bind_group_layout,
};
use crate::text::atlas::{
    GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT, GlyphAtlasBitmapFaceValidity,
    GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapPreparedUploadPlan,
    GlyphAtlasBitmapRenderSubmissionPlan, GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasFormat,
    GlyphAtlasSet, GlyphAtlasStorageFormat,
};

use super::super::atlas_texture_upload::{
    GlyphAtlasBitmapTextureUploadFramePlan, GlyphAtlasBitmapTextureUploadFrameReport,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity,
    prepare_glyph_atlas_bitmap_texture_upload_for_resources,
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
};

pub(in crate::graphics::scene::scene_renderer::ui) struct GlyphAtlasBitmapRenderer {
    target_format: wgpu::TextureFormat,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    viewport_uniform_buffer: wgpu::Buffer,
    last_viewport_transform: Option<GlyphAtlasGpuViewportTransform>,
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
            last_viewport_transform: None,
            atlas_resources: vec![GlyphAtlasBitmapRendererAtlasResource::new(
                atlas_format,
                atlas,
            )],
            draw_passes: vec![GlyphAtlasBitmapRendererDrawPass::new()],
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
        buffer_uploads: &mut WgpuBufferUploadBatch,
        force_full_upload: bool,
    ) {
        self.write_viewport_uniform(plan.viewport_transform, buffer_uploads, force_full_upload);
        let atlas_format =
            glyph_atlas_bitmap_renderer_default_format_for_storage(atlas_storage_format);
        let mut report = self.prepare_frame_pass(
            device,
            plan,
            atlas_size,
            atlas_layer_count,
            atlas_format,
            &[(atlas_format, atlas_layer_count)],
            buffer_uploads,
            force_full_upload,
        );
        report = self.report_with_pending_face_invalidation(report);
        self.draw_passes.truncate(1);
        self.last_report = report;
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_idle(&mut self) {
        glyph_atlas_bitmap_renderer_release_idle_draw_passes(&mut self.draw_passes);
        self.last_viewport_transform = None;
        let report = glyph_atlas_bitmap_renderer_idle_prepare_report(
            self.atlas_resources.iter().map(|resource| {
                (
                    resource.atlas.size(),
                    resource.atlas.layer_count(),
                    resource.atlas_format,
                )
            }),
            self.pipeline_resources.len(),
        );
        self.last_report = self.report_with_pending_face_invalidation(report);
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_submission<'a, I>(
        &mut self,
        device: &wgpu::Device,
        submission: &GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: I,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_format: GlyphAtlasFormat,
        buffer_uploads: &mut WgpuBufferUploadBatch,
        texture_uploads: &mut WgpuTextureUploadBatch,
        force_full_upload: bool,
    ) -> GlyphAtlasBitmapPageShadowCommit
    where
        I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
    {
        self.prepare_submission_with_face_validity(
            device,
            submission,
            source_bytes,
            atlas_size,
            atlas_layer_count,
            atlas_format,
            GlyphAtlasBitmapFaceValidity::Valid,
            buffer_uploads,
            texture_uploads,
            force_full_upload,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::ui) fn prepare_submission_with_face_validity<
        'a,
        I,
    >(
        &mut self,
        device: &wgpu::Device,
        submission: &GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: I,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        atlas_format: GlyphAtlasFormat,
        face_validity: GlyphAtlasBitmapFaceValidity,
        buffer_uploads: &mut WgpuBufferUploadBatch,
        texture_uploads: &mut WgpuTextureUploadBatch,
        force_full_upload: bool,
    ) -> GlyphAtlasBitmapPageShadowCommit
    where
        I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
    {
        crate::profile_scope!(
            "runtime",
            "ui_text.atlas_upload",
            "prepare_submission_with_face_validity"
        );
        self.write_viewport_uniform(
            submission.gpu_draw.viewport_transform,
            buffer_uploads,
            force_full_upload,
        );
        let atlas_requirements = glyph_atlas_bitmap_renderer_submission_atlas_requirements(
            submission,
            atlas_format,
            atlas_layer_count,
        );
        let mut report = self.prepare_frame_pass(
            device,
            &submission.gpu_draw,
            atlas_size,
            atlas_layer_count,
            atlas_format,
            atlas_requirements.as_slice(),
            buffer_uploads,
            force_full_upload,
        );
        self.draw_passes.truncate(1);
        let force_full_atlas_replay = force_full_upload || report.atlas_resized;
        let (upload_report, shadow_commit, upload_plan_built) = self.prepare_submission_upload(
            submission,
            source_bytes,
            face_validity,
            texture_uploads,
            force_full_atlas_replay,
        );
        report = report
            .with_upload_report(upload_report)
            .with_upload_plan_preparation(upload_plan_built);
        report = self.report_with_pending_face_invalidation(report);
        self.last_report = report;
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
            pass.set_vertex_buffer(0, instance_buffer.slice(..));
            let mut bound_format = None;
            for command in &draw_pass.draw_commands {
                let atlas_format = command.key.page_key.format;
                if bound_format != Some(atlas_format) {
                    let Some(atlas_resource) = self.atlas_resource(atlas_format) else {
                        bound_format = None;
                        continue;
                    };
                    pass.set_bind_group(0, atlas_resource.atlas.bind_group(), &[]);
                    bound_format = Some(atlas_format);
                }
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

    fn prepare_frame_pass(
        &mut self,
        device: &wgpu::Device,
        plan: &GlyphAtlasGpuDrawPlan,
        atlas_size: UVec2,
        atlas_layer_count: u32,
        fallback_format: GlyphAtlasFormat,
        atlas_requirements: &[(GlyphAtlasFormat, u32)],
        buffer_uploads: &mut WgpuBufferUploadBatch,
        force_full_upload: bool,
    ) -> GlyphAtlasBitmapRendererPrepareReport {
        self.ensure_pipelines(device, &plan.pipeline_contracts);
        let mut atlas_resized = false;
        for &(atlas_format, required_layer_count) in atlas_requirements {
            atlas_resized |=
                self.ensure_atlas_resources(device, atlas_size, required_layer_count, atlas_format);
        }
        self.ensure_draw_pass(0);
        let draw_pass = &mut self.draw_passes[0];
        let (instance_buffer_capacity_byte_len, instance_buffer_reallocation_count) =
            glyph_atlas_bitmap_renderer_write_instance_buffer(
                device,
                draw_pass,
                plan.instances.as_slice(),
                buffer_uploads,
                force_full_upload,
            );
        draw_pass.draw_commands = plan.draw_commands.clone();

        let mut report = glyph_atlas_bitmap_renderer_prepare_report(
            plan,
            self.pipeline_resources.len(),
            atlas_size,
            atlas_layer_count,
            fallback_format.storage_format(),
            atlas_resized,
        );
        report.storage_resource_count = atlas_requirements.len();
        report.ordered_draw_segment_count =
            glyph_atlas_bitmap_renderer_ordered_draw_segment_count(plan);
        report.mixed_atlas_storage_format = atlas_requirements.len() > 1;
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

    fn ensure_draw_pass(&mut self, pass_index: usize) {
        if self.draw_passes.len() == pass_index {
            self.draw_passes
                .push(GlyphAtlasBitmapRendererDrawPass::new());
            return;
        }
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
        &mut self,
        viewport_transform: GlyphAtlasGpuViewportTransform,
        buffer_uploads: &mut WgpuBufferUploadBatch,
        force_full_upload: bool,
    ) {
        if !force_full_upload
            && !glyph_atlas_bitmap_renderer_viewport_uniform_write_required(
                self.last_viewport_transform,
                viewport_transform,
            )
        {
            return;
        }
        buffer_uploads.push(WgpuBufferUpload::from_bytes(
            self.viewport_uniform_buffer.clone(),
            0,
            bytemuck::cast_slice(&viewport_transform.uniform_bytes()),
        ));
        self.last_viewport_transform = Some(viewport_transform);
    }

    fn prepare_submission_upload<'a, I>(
        &self,
        submission: &GlyphAtlasBitmapRenderSubmissionPlan,
        source_bytes: I,
        face_validity: GlyphAtlasBitmapFaceValidity,
        texture_uploads: &mut WgpuTextureUploadBatch,
        force_full_atlas_replay: bool,
    ) -> (
        GlyphAtlasBitmapTextureUploadFrameReport,
        GlyphAtlasBitmapPageShadowCommit,
        bool,
    )
    where
        I: IntoIterator<Item = GlyphAtlasBitmapUploadSourceBytes<'a>>,
    {
        if submission.upload_commands().is_empty() && !force_full_atlas_replay {
            return (
                GlyphAtlasBitmapTextureUploadFrameReport::default(),
                GlyphAtlasBitmapPageShadowCommit::default(),
                false,
            );
        }

        let prepared_upload = if force_full_atlas_replay {
            submission.prepared_upload_with_full_shadow_replay(source_bytes)
        } else {
            submission.prepared_upload(source_bytes)
        };
        let prepared_texture_upload = prepare_glyph_atlas_bitmap_texture_upload_for_resources(
            self.atlas_resources
                .iter()
                .map(|resource| (resource.atlas_format, resource.atlas.texture().clone())),
            &submission.run,
            prepared_upload,
            face_validity,
        );
        let (prepared_texture_uploads, shadow_commit, upload_report) =
            prepared_texture_upload.into_parts();
        texture_uploads.append(prepared_texture_uploads);
        (upload_report, shadow_commit, true)
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

pub(super) fn glyph_atlas_bitmap_renderer_viewport_uniform_write_required(
    current_transform: Option<GlyphAtlasGpuViewportTransform>,
    next_transform: GlyphAtlasGpuViewportTransform,
) -> bool {
    current_transform != Some(next_transform)
}

pub(super) fn glyph_atlas_bitmap_renderer_release_idle_draw_passes(
    draw_passes: &mut Vec<GlyphAtlasBitmapRendererDrawPass>,
) {
    // Idle is the explicit reclamation boundary; stable non-idle frames retain capacity instead.
    draw_passes.truncate(1);
    if let Some(draw_pass) = draw_passes.first_mut() {
        draw_pass.instance_buffer = None;
        draw_pass.instance_buffer_capacity_bytes = 0;
        draw_pass.instance_buffer_payload_hash = None;
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

    fn with_upload_plan_preparation(mut self, built: bool) -> Self {
        self.upload_plan_build_count = usize::from(built);
        self.upload_plan_skip_count = usize::from(!built);
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

pub(super) fn glyph_atlas_bitmap_renderer_submission_atlas_requirements(
    submission: &GlyphAtlasBitmapRenderSubmissionPlan,
    fallback_format: GlyphAtlasFormat,
    fallback_layer_count: u32,
) -> Vec<(GlyphAtlasFormat, u32)> {
    let mut requirements: Vec<(GlyphAtlasFormat, u32)> = Vec::new();
    let mut add_page = |format: GlyphAtlasFormat, page_index: u32| {
        let required_layers = page_index.saturating_add(1).max(1);
        if let Some((_, layer_count)) = requirements
            .iter_mut()
            .find(|(existing_format, _)| *existing_format == format)
        {
            *layer_count = (*layer_count).max(required_layers);
        } else {
            requirements.push((format, required_layers));
        }
    };

    for glyph in &submission.run.glyphs {
        add_page(glyph.page_key.format, glyph.page_key.page_index);
    }
    for copy in &submission.run.upload_copies {
        add_page(copy.page_key.format, copy.page_key.page_index);
    }
    if requirements.is_empty() {
        requirements.push((fallback_format, fallback_layer_count.max(1)));
    }
    requirements
}

pub(super) fn glyph_atlas_bitmap_renderer_ordered_draw_segment_count(
    plan: &GlyphAtlasGpuDrawPlan,
) -> usize {
    let mut previous_format = None;
    let mut segment_count = 0;
    for command in &plan.draw_commands {
        let format = command.key.page_key.format;
        if previous_format != Some(format) {
            segment_count += 1;
            previous_format = Some(format);
        }
    }
    segment_count
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
        storage_resource_count: 1,
        ordered_draw_segment_count: glyph_atlas_bitmap_renderer_ordered_draw_segment_count(plan),
        atlas_resized,
        vertex_count: plan.vertex_count(),
        vertex_buffer_byte_len: std::mem::size_of_val(plan.instances.as_slice()),
        instance_buffer_capacity_byte_len: 0,
        instance_buffer_reallocation_count: 0,
        draw_command_count: plan.draw_commands.len(),
        pipeline_count,
        requires_background_composite: plan.requires_background_composite,
        upload_plan_build_count: 0,
        upload_plan_skip_count: 0,
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
    atlas_resources: impl IntoIterator<Item = (UVec2, u32, GlyphAtlasFormat)>,
    pipeline_count: usize,
) -> GlyphAtlasBitmapRendererPrepareReport {
    let mut atlas_resources = atlas_resources.into_iter();
    let Some((atlas_size, first_layer_count, first_format)) = atlas_resources.next() else {
        return GlyphAtlasBitmapRendererPrepareReport::default();
    };
    let mut atlas_layer_count = first_layer_count.max(1);
    let atlas_storage_format = first_format.storage_format();
    let mut storage_resource_count = 1;
    let mut mixed_atlas_storage_format = false;
    for (_, layer_count, atlas_format) in atlas_resources {
        atlas_layer_count = atlas_layer_count.max(layer_count.max(1));
        storage_resource_count += 1;
        mixed_atlas_storage_format |= atlas_format.storage_format() != atlas_storage_format;
    }

    GlyphAtlasBitmapRendererPrepareReport {
        atlas_size,
        atlas_layer_count,
        atlas_storage_format,
        storage_pass_count: 0,
        mixed_atlas_storage_format,
        storage_resource_count,
        pipeline_count,
        ..GlyphAtlasBitmapRendererPrepareReport::default()
    }
}
