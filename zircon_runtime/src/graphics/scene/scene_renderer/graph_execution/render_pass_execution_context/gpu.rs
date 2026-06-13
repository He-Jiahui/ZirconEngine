use crate::core::framework::render::{
    MotionVectorCameraStatus, RenderFrameExtract, RenderPluginRendererOutputs,
};
use crate::core::math::UVec2;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::depth_attachment_operations;
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommand, MeshDrawCommandStream, MeshDrawReplayStatsAccumulator,
    MeshIndirectDrawExecution, MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::overlay::{
    BaseScenePass, PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::scene::scene_renderer::particle::ParticleRenderer;
use crate::graphics::scene::scene_renderer::prepass::NormalPrepassPipeline;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::scene::scene_renderer::shadow::{ShadowFramePlan, ShadowMapRenderer};
use crate::graphics::scene::scene_renderer::sprite::SpriteRenderer;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::visibility::HzbOcclusionCullReport;
use crate::render_graph::{RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps};

use super::super::{
    RenderGraphComputeDispatchRecord, RenderGraphExecutionResources, RenderGraphLightGridReport,
};

mod hzb_occlusion;
mod mesh_motion_vector;
mod post_process;

pub(in crate::graphics::scene::scene_renderer) use post_process::RenderPassPostProcessStackContext;

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer) struct RenderPassMeshCommandLists<'a> {
    pub replay_stats: &'a MeshDrawReplayStatsAccumulator,
    pub gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
    pub depth_prepass_commands: &'a [MeshDrawCommand],
    pub shadow_commands: &'a [MeshDrawCommand],
    pub opaque_commands: &'a [MeshDrawCommand],
    pub alpha_mask_commands: &'a [MeshDrawCommand],
    pub transparent_commands: &'a [MeshDrawCommand],
    pub velocity_commands: &'a [MeshDrawCommand],
    pub depth_prepass_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub shadow_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub opaque_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub alpha_mask_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub transparent_indirect: Option<&'a MeshIndirectDrawExecution>,
    pub velocity_indirect: Option<&'a MeshIndirectDrawExecution>,
}

impl<'a> RenderPassMeshCommandLists<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn stream_for_stage(
        &self,
        stage: RenderPassStage,
    ) -> MeshDrawCommandStream<'a> {
        match stage {
            RenderPassStage::DepthPrepass => self.depth_prepass_stream(),
            RenderPassStage::Opaque3d => self.opaque_stream(),
            RenderPassStage::AlphaMask3d => self.alpha_mask_stream(),
            RenderPassStage::Transparent3d => self.transparent_stream(),
            RenderPassStage::Shadow => self.shadow_stream(),
            _ => MeshDrawCommandStream::empty(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn depth_prepass_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.depth_prepass_commands, self.depth_prepass_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn shadow_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.shadow_commands, self.shadow_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn opaque_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.opaque_commands, self.opaque_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn alpha_mask_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.alpha_mask_commands, self.alpha_mask_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn transparent_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.transparent_commands, self.transparent_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn velocity_stream(
        &self,
    ) -> MeshDrawCommandStream<'a> {
        MeshDrawCommandStream::new(self.velocity_commands, self.velocity_indirect)
    }

    pub(in crate::graphics::scene::scene_renderer) fn occlusion_cull_candidate_arg_count(
        &self,
    ) -> u32 {
        self.hzb_occlusion_indirect_executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.args_count())
            .sum()
    }

    pub(in crate::graphics::scene::scene_renderer) fn occlusion_cull_candidate_instance_count(
        &self,
    ) -> u32 {
        self.hzb_occlusion_indirect_executions()
            .into_iter()
            .flatten()
            .map(|execution| execution.total_instances())
            .sum()
    }

    pub(in crate::graphics::scene::scene_renderer) fn hzb_occlusion_indirect_executions(
        &self,
    ) -> [Option<&'a MeshIndirectDrawExecution>; 3] {
        [
            self.opaque_indirect,
            self.alpha_mask_indirect,
            self.velocity_indirect,
        ]
    }
}

pub struct RenderPassGpuExecutionContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    frame: &'a ViewportRenderFrame,
    pub scene_bind_group: &'a wgpu::BindGroup,
    pub resources: &'a mut RenderGraphExecutionResources,
    pub plugin_outputs: &'a mut RenderPluginRendererOutputs,
    pub(in crate::graphics::scene::scene_renderer) screen_space_ui_renderer:
        &'a mut ScreenSpaceUiRenderer,
    post_process_stack: Option<RenderPassPostProcessStackContext<'a>>,
    overlay_renderer: Option<&'a mut ViewportOverlayRenderer>,
    prepared_overlays: Option<&'a PreparedOverlayBuffers>,
    prepass: Option<&'a NormalPrepassPipeline>,
    shadow_map_renderer: Option<&'a ShadowMapRenderer>,
    shadow_atlas_resources: Option<&'a ShadowAtlasResources>,
    shadow_frame_plan: Option<&'a ShadowFramePlan>,
    particle_renderer: Option<&'a ParticleRenderer>,
    sprite_renderer: Option<&'a SpriteRenderer>,
    deferred: Option<&'a DeferredSceneResources>,
    streamer: Option<&'a ResourceStreamer>,
    mesh_pipelines: Option<&'a mut MeshPipelineCache>,
    mesh_draw_lists: Option<RenderPassMeshCommandLists<'a>>,
    hzb_occlusion_culler: Option<&'a HzbOcclusionCuller>,
    compute_dispatches: Vec<RenderGraphComputeDispatchRecord>,
    hzb_occlusion_cull_report: Option<HzbOcclusionCullReport>,
    light_grid_report: Option<RenderGraphLightGridReport>,
    motion_vector_camera_status: MotionVectorCameraStatus,
}

impl std::fmt::Debug for RenderPassGpuExecutionContext<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderPassGpuExecutionContext")
            .field("viewport_size", &self.frame.viewport_size)
            .field("has_post_process_stack", &self.post_process_stack.is_some())
            .field("has_overlay_renderer", &self.overlay_renderer.is_some())
            .field("has_prepass", &self.prepass.is_some())
            .field(
                "has_shadow_map_renderer",
                &self.shadow_map_renderer.is_some(),
            )
            .field("has_particle_renderer", &self.particle_renderer.is_some())
            .field("has_sprite_renderer", &self.sprite_renderer.is_some())
            .field("has_deferred_renderer", &self.deferred.is_some())
            .finish_non_exhaustive()
    }
}

impl<'a> RenderPassGpuExecutionContext<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn new(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        frame: &'a ViewportRenderFrame,
        scene_bind_group: &'a wgpu::BindGroup,
        resources: &'a mut RenderGraphExecutionResources,
        plugin_outputs: &'a mut RenderPluginRendererOutputs,
        screen_space_ui_renderer: &'a mut ScreenSpaceUiRenderer,
    ) -> Self {
        Self {
            device,
            queue,
            encoder,
            frame,
            scene_bind_group,
            resources,
            plugin_outputs,
            screen_space_ui_renderer,
            post_process_stack: None,
            overlay_renderer: None,
            prepared_overlays: None,
            prepass: None,
            shadow_map_renderer: None,
            shadow_atlas_resources: None,
            shadow_frame_plan: None,
            particle_renderer: None,
            sprite_renderer: None,
            deferred: None,
            streamer: None,
            mesh_pipelines: None,
            mesh_draw_lists: None,
            hzb_occlusion_culler: None,
            compute_dispatches: Vec::new(),
            hzb_occlusion_cull_report: None,
            light_grid_report: None,
            motion_vector_camera_status: MotionVectorCameraStatus::NotRequested,
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn new_for_test(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        encoder: &'a mut wgpu::CommandEncoder,
        frame: &'a ViewportRenderFrame,
        scene_bind_group: &'a wgpu::BindGroup,
        resources: &'a mut RenderGraphExecutionResources,
        plugin_outputs: &'a mut RenderPluginRendererOutputs,
        screen_space_ui_renderer: &'a mut ScreenSpaceUiRenderer,
    ) -> Self {
        Self::new(
            device,
            queue,
            encoder,
            frame,
            scene_bind_group,
            resources,
            plugin_outputs,
            screen_space_ui_renderer,
        )
    }

    pub fn frame_extract(&self) -> &RenderFrameExtract {
        &self.frame.extract
    }

    pub fn viewport_size(&self) -> UVec2 {
        self.frame.viewport_size
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_compute_dispatches(
        &mut self,
    ) -> Vec<RenderGraphComputeDispatchRecord> {
        std::mem::take(&mut self.compute_dispatches)
    }

    pub fn record_compute_dispatch(
        &mut self,
        pass_name: impl Into<String>,
        executor_id: impl Into<String>,
        pipeline_label: impl Into<String>,
        workgroup_size: [u32; 3],
        dispatch_groups: [u32; 3],
        storage_write_resources: Vec<String>,
    ) {
        self.compute_dispatches
            .push(RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                pipeline_label,
                workgroup_size,
                dispatch_groups,
                storage_write_resources,
            ));
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_hzb_occlusion_cull_report(
        &mut self,
    ) -> Option<HzbOcclusionCullReport> {
        self.hzb_occlusion_cull_report.take()
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_light_grid_report(
        &mut self,
    ) -> Option<RenderGraphLightGridReport> {
        self.light_grid_report.take()
    }

    pub(in crate::graphics::scene::scene_renderer) fn motion_vector_camera_status(
        &self,
    ) -> MotionVectorCameraStatus {
        self.motion_vector_camera_status
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_prepass_renderer(
        mut self,
        prepass: &'a NormalPrepassPipeline,
        mesh_draw_lists: RenderPassMeshCommandLists<'a>,
    ) -> Self {
        self.prepass = Some(prepass);
        self.mesh_draw_lists = Some(mesh_draw_lists);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_shadow_map_renderer(
        mut self,
        shadow_map_renderer: &'a ShadowMapRenderer,
        mesh_draw_lists: RenderPassMeshCommandLists<'a>,
    ) -> Self {
        self.shadow_map_renderer = Some(shadow_map_renderer);
        self.mesh_draw_lists = Some(mesh_draw_lists);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_shadow_receiver(
        mut self,
        shadow_map_renderer: &'a ShadowMapRenderer,
    ) -> Self {
        self.shadow_map_renderer = Some(shadow_map_renderer);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_shadow_atlas_resources(
        mut self,
        shadow_atlas_resources: &'a ShadowAtlasResources,
    ) -> Self {
        self.shadow_atlas_resources = Some(shadow_atlas_resources);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_shadow_frame_plan(
        mut self,
        shadow_frame_plan: &'a ShadowFramePlan,
    ) -> Self {
        self.shadow_frame_plan = Some(shadow_frame_plan);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_overlay_renderer(
        mut self,
        overlay_renderer: &'a mut ViewportOverlayRenderer,
        prepared_overlays: &'a PreparedOverlayBuffers,
    ) -> Self {
        self.overlay_renderer = Some(overlay_renderer);
        self.prepared_overlays = Some(prepared_overlays);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_preview_sky_renderer(
        mut self,
        overlay_renderer: &'a mut ViewportOverlayRenderer,
    ) -> Self {
        self.overlay_renderer = Some(overlay_renderer);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_particle_renderer(
        mut self,
        particle_renderer: &'a ParticleRenderer,
    ) -> Self {
        self.particle_renderer = Some(particle_renderer);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_sprite_renderer(
        mut self,
        sprite_renderer: &'a SpriteRenderer,
        streamer: &'a ResourceStreamer,
    ) -> Self {
        self.sprite_renderer = Some(sprite_renderer);
        self.streamer = Some(streamer);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_deferred_renderer(
        mut self,
        deferred: &'a DeferredSceneResources,
        mesh_draw_lists: RenderPassMeshCommandLists<'a>,
    ) -> Self {
        self.deferred = Some(deferred);
        self.mesh_draw_lists = Some(mesh_draw_lists);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_mesh_renderer(
        mut self,
        mesh_pipelines: &'a mut MeshPipelineCache,
        streamer: &'a ResourceStreamer,
        mesh_draw_lists: RenderPassMeshCommandLists<'a>,
    ) -> Self {
        self.mesh_pipelines = Some(mesh_pipelines);
        self.streamer = Some(streamer);
        self.mesh_draw_lists = Some(mesh_draw_lists);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_hzb_occlusion_culler(
        mut self,
        hzb_occlusion_culler: &'a HzbOcclusionCuller,
    ) -> Self {
        self.hzb_occlusion_culler = Some(hzb_occlusion_culler);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_depth_prepass_to_resources(
        &mut self,
        pass_name: &str,
        normal_resource_name: &str,
        depth_resource_name: &str,
        normal_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let normal_view = self.resources.require_texture_view(normal_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let prepass = self.prepass.ok_or_else(|| {
            format!(
                "depth prepass graph executor for pass `{pass_name}` requires normal prepass context"
            )
        })?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!(
                "depth prepass graph executor for pass `{pass_name}` requires mesh draw context"
            )
        })?;
        if mesh_draw_lists.depth_prepass_commands.is_empty() {
            return Ok(());
        }
        let replay_stats = prepass.record_commands_with_attachment_ops(
            self.encoder,
            normal_view,
            depth_view,
            self.scene_bind_group,
            mesh_draw_lists.gpu_scene_bind_group,
            mesh_draw_lists.depth_prepass_stream(),
            normal_attachment_ops,
            depth_attachment_ops,
        );
        mesh_draw_lists.replay_stats.record(replay_stats);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_shadow_atlas_to_resources(
        &mut self,
        pass_name: &str,
        shadow_atlas_resource_name: &str,
        shadow_atlas_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let shadow_atlas_view = self
            .resources
            .require_texture_view(shadow_atlas_resource_name)?;
        if let Some(shadow_map_renderer) = self.shadow_map_renderer {
            let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
                format!(
                    "shadow atlas graph executor for pass `{pass_name}` requires mesh draw context"
                )
            })?;
            if let Some(shadow_frame_plan) = self.shadow_frame_plan {
                let replay_stats = shadow_map_renderer.record_atlas_commands_with_attachment_ops(
                    self.queue,
                    self.encoder,
                    pass_name,
                    shadow_atlas_view,
                    shadow_frame_plan.atlas_passes(),
                    self.frame,
                    mesh_draw_lists.gpu_scene_bind_group,
                    mesh_draw_lists.shadow_stream(),
                    shadow_atlas_attachment_ops,
                );
                mesh_draw_lists.replay_stats.record(replay_stats);
            } else {
                record_depth_clear_pass(
                    self.encoder,
                    pass_name,
                    shadow_atlas_view,
                    shadow_atlas_attachment_ops,
                );
            }
            return Ok(());
        }
        record_depth_clear_pass(
            self.encoder,
            pass_name,
            shadow_atlas_view,
            shadow_atlas_attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_mesh_stage_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        stage: RenderPassStage,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let color_view = self.resources.require_texture_view(color_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires mesh draw context")
        })?;
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires mesh pipeline context")
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires resource streamer context")
        })?;
        let stream = mesh_draw_lists.stream_for_stage(stage);
        if stream.is_empty() {
            return Ok(());
        }
        let light_grid_params_buffer = self.resources.require_buffer(
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        )?;
        let light_zbins_buffer = self.resources.require_buffer(
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_ZBINS,
        )?;
        let light_tile_masks_buffer = self.resources.require_buffer(
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        )?;
        let replay_stats = BaseScenePass.record_commands_with_attachment_ops(
            self.encoder,
            self.device,
            color_view,
            depth_view,
            self.scene_bind_group,
            mesh_draw_lists.gpu_scene_bind_group,
            [stream],
            mesh_pipelines,
            streamer,
            self.frame,
            self.shadow_atlas_resources,
            Some(light_grid_params_buffer),
            Some(light_zbins_buffer),
            Some(light_tile_masks_buffer),
            mesh_stage_attachment_ops(stage, attachment_ops),
            mesh_stage_attachment_ops(stage, depth_attachment_ops),
        );
        mesh_draw_lists.replay_stats.record(replay_stats);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_deferred_gbuffer_to_resources(
        &mut self,
        pass_name: &str,
        gbuffer_albedo_resource_name: &str,
        gbuffer_material_resource_name: &str,
        depth_resource_name: &str,
        albedo_attachment_ops: RenderGraphAttachmentOps,
        material_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let gbuffer_albedo_view = self
            .resources
            .require_texture_view(gbuffer_albedo_resource_name)?;
        let gbuffer_material_view = self
            .resources
            .require_texture_view(gbuffer_material_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let deferred = self.deferred.ok_or_else(|| {
            format!(
                "deferred graph executor for pass `{pass_name}` requires deferred renderer context"
            )
        })?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!("deferred graph executor for pass `{pass_name}` requires mesh draw context")
        })?;
        let replay_stats = deferred.record_gbuffer_geometry(
            self.encoder,
            gbuffer_albedo_view,
            gbuffer_material_view,
            depth_view,
            self.scene_bind_group,
            mesh_draw_lists.gpu_scene_bind_group,
            albedo_attachment_ops,
            material_attachment_ops,
            [
                mesh_draw_lists.opaque_stream(),
                mesh_draw_lists.alpha_mask_stream(),
            ],
        );
        mesh_draw_lists.replay_stats.record(replay_stats);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_deferred_lighting_to_resources(
        &mut self,
        pass_name: &str,
        gbuffer_albedo_resource_name: &str,
        gbuffer_normal_resource_name: &str,
        gbuffer_material_resource_name: &str,
        scene_depth_resource_name: &str,
        background_resource_name: &str,
        scene_color_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let gbuffer_albedo_view = self
            .resources
            .require_texture_view(gbuffer_albedo_resource_name)?;
        let gbuffer_normal_view = self
            .resources
            .require_texture_view(gbuffer_normal_resource_name)?;
        let gbuffer_material_view = self
            .resources
            .require_texture_view(gbuffer_material_resource_name)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(scene_depth_resource_name)?;
        let light_grid_params_buffer = self.resources.require_buffer(
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        )?;
        let light_zbins_buffer = self.resources.require_buffer(
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_ZBINS,
        )?;
        let light_tile_masks_buffer = self.resources.require_buffer(
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        )?;
        let background_view = self
            .resources
            .require_texture_view(background_resource_name)?;
        let scene_color_view = self
            .resources
            .require_texture_view(scene_color_resource_name)?;
        let deferred = self.deferred.ok_or_else(|| {
            format!(
                "deferred graph executor for pass `{pass_name}` requires deferred renderer context"
            )
        })?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!("deferred graph executor for pass `{pass_name}` requires mesh draw context")
        })?;
        let gpu_scene_bind_group = mesh_draw_lists
            .gpu_scene_bind_group
            .ok_or_else(|| {
                format!(
                    "deferred graph executor for pass `{pass_name}` requires GPUScene bind group"
                )
            })?
            .bind_group();
        deferred.execute_lighting(
            self.device,
            self.encoder,
            self.scene_bind_group,
            gpu_scene_bind_group,
            gbuffer_albedo_view,
            gbuffer_normal_view,
            gbuffer_material_view,
            scene_depth_view,
            self.shadow_atlas_resources,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            background_view,
            scene_color_view,
            attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_sprite_stage_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        stage: RenderPassStage,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let color_view = self.resources.require_texture_view(color_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let sprite_renderer = self.sprite_renderer.ok_or_else(|| {
            format!("sprite graph executor for stage `{stage:?}` requires sprite renderer context")
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "sprite graph executor for stage `{stage:?}` requires resource streamer context"
            )
        })?;
        sprite_renderer.record(
            self.device,
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            streamer,
            self.frame,
            stage,
            attachment_ops,
            depth_attachment_ops,
        );
        Ok(())
    }

    pub fn record_particle_billboards_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
    ) -> Result<(), String> {
        let color_view = self.resources.require_texture_view(color_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let particle_renderer = self.particle_renderer.ok_or_else(|| {
            format!(
                "particle graph executor requires particle renderer context for resources `{color_resource_name}` and `{depth_resource_name}`"
            )
        })?;
        particle_renderer.record(
            self.device,
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_ui_to_resource(
        &mut self,
        resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let color_view = self.resources.require_texture_view(resource_name)?;
        self.screen_space_ui_renderer.record(
            self.device,
            self.queue,
            self.encoder,
            color_view,
            self.frame,
            attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_preview_sky_to_resources(
        &mut self,
        pass_name: &str,
        color_resource_name: &str,
        depth_resource_name: &str,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        if self.overlay_renderer.is_none() {
            return Err(format!(
                "preview sky graph executor for pass `{pass_name}` requires preview sky renderer context"
            ));
        }
        let color_view = self.resources.require_texture_view(color_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let overlay_renderer = self
            .overlay_renderer
            .as_deref_mut()
            .expect("preview sky renderer context was checked before resource resolution");
        overlay_renderer.record_preview_sky_with_attachment_ops(
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            color_attachment_ops,
            depth_attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_overlay_to_resources(
        &mut self,
        pass_name: &str,
        color_resource_name: &str,
        depth_resource_name: &str,
    ) -> Result<(), String> {
        let color_view = self.resources.require_texture_view(color_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let overlay_renderer = self.overlay_renderer.as_deref_mut().ok_or_else(|| {
            format!(
                "overlay graph executor for pass `{pass_name}` requires overlay renderer context"
            )
        })?;
        let prepared_overlays = self.prepared_overlays.ok_or_else(|| {
            format!(
                "overlay graph executor for pass `{pass_name}` requires prepared overlay buffers"
            )
        })?;
        overlay_renderer.record_overlays(
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            prepared_overlays,
        );
        Ok(())
    }
}

fn mesh_stage_attachment_ops(
    stage: RenderPassStage,
    attachment_ops: RenderGraphAttachmentOps,
) -> RenderGraphAttachmentOps {
    if matches!(stage, RenderPassStage::Opaque3d) {
        return RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: attachment_ops.store,
        };
    }
    attachment_ops
}

fn record_depth_clear_pass(
    encoder: &mut wgpu::CommandEncoder,
    pass_name: &str,
    depth_view: &wgpu::TextureView,
    attachment_ops: RenderGraphAttachmentOps,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(pass_name),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(depth_attachment_operations(attachment_ops, 1.0)),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}
