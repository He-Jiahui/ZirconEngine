use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessGraphResourceNames, RenderFrameExtract,
    RenderPluginRendererOutputs,
};
use crate::core::math::UVec2;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommandReplayer;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::overlay::{
    BaseScenePass, PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::scene::scene_renderer::particle::ParticleRenderer;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::scene::scene_renderer::shadow::{ShadowFramePlan, ShadowMapRenderer};
use crate::graphics::scene::scene_renderer::sprite::SpriteRenderer;
use crate::graphics::scene::scene_renderer::transparent::has_transparent_sprite_submissions;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::types::{
    ViewportCameraStackAttachmentPolicy, ViewportRenderFrame, ViewportRenderRegion,
};
use crate::graphics::visibility::HzbOcclusionCullReport;
use crate::render_graph::{
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphResourceAccessKind,
};

use super::super::{
    RenderGraphComputeDispatchRecord, RenderGraphExecutionResources, RenderGraphLightGridReport,
};
use super::RgResourceResolver;

mod deferred;
mod hzb_occlusion;
mod mesh_command_lists;
mod particle;
mod post_process;
mod reports;
mod resource_lookup;
mod surface;

pub(in crate::graphics::scene::scene_renderer) use mesh_command_lists::RenderPassMeshCommandLists;
pub use particle::ParticleGpuTransparentDrawContext;
pub(in crate::graphics::scene::scene_renderer) use post_process::RenderPassPostProcessStackContext;

pub struct RenderPassGpuExecutionContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    frame: &'a ViewportRenderFrame,
    scene_bind_group_layout: &'a wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    pub scene_bind_group: &'a wgpu::BindGroup,
    pub resources: &'a mut RenderGraphExecutionResources,
    pub plugin_outputs: &'a mut RenderPluginRendererOutputs,
    resource_resolver: Option<RgResourceResolver<'a>>,
    pub(in crate::graphics::scene::scene_renderer) screen_space_ui_renderer:
        &'a mut ScreenSpaceUiRenderer,
    post_process_stack: Option<RenderPassPostProcessStackContext<'a>>,
    overlay_renderer: Option<&'a mut ViewportOverlayRenderer>,
    prepared_overlays: Option<&'a PreparedOverlayBuffers>,
    shadow_map_renderer: Option<&'a ShadowMapRenderer>,
    pub(in crate::graphics::scene::scene_renderer) shadow_atlas_resources:
        Option<&'a ShadowAtlasResources>,
    shadow_frame_plan: Option<&'a ShadowFramePlan>,
    particle_renderer: Option<&'a ParticleRenderer>,
    sprite_renderer: Option<&'a SpriteRenderer>,
    deferred: Option<&'a DeferredSceneResources>,
    pub(in crate::graphics::scene::scene_renderer) streamer: Option<&'a ResourceStreamer>,
    pub(in crate::graphics::scene::scene_renderer) mesh_pipelines:
        Option<&'a mut MeshPipelineCache>,
    pub(in crate::graphics::scene::scene_renderer) mesh_draw_lists:
        Option<RenderPassMeshCommandLists<'a>>,
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
        scene_bind_group_layout: &'a wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
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
            scene_bind_group_layout,
            target_format,
            depth_format,
            scene_bind_group,
            resources,
            plugin_outputs,
            resource_resolver: None,
            screen_space_ui_renderer,
            post_process_stack: None,
            overlay_renderer: None,
            prepared_overlays: None,
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
        scene_bind_group_layout: &'a wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
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
            scene_bind_group_layout,
            target_format,
            depth_format,
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

    pub(in crate::graphics::scene::scene_renderer) fn render_region(&self) -> ViewportRenderRegion {
        self.frame.render_region()
    }

    pub(in crate::graphics::scene::scene_renderer) fn render_region_for_write_resource(
        &self,
        resource_name: &str,
    ) -> ViewportRenderRegion {
        if writes_physical_output_resource(resource_name) {
            self.render_region()
        } else {
            self.render_region().local_render_region()
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn camera_stack_attachment_policy(
        &self,
    ) -> ViewportCameraStackAttachmentPolicy {
        self.frame.camera_stack_attachment_policy()
    }

    pub fn scene_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.scene_bind_group_layout
    }

    pub fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }

    pub fn depth_format(&self) -> wgpu::TextureFormat {
        self.depth_format
    }

    pub(in crate::graphics::scene::scene_renderer) fn resource_resolver(
        &self,
    ) -> Option<RgResourceResolver<'a>> {
        self.resource_resolver
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
        streamer: &'a ResourceStreamer,
        mesh_draw_lists: RenderPassMeshCommandLists<'a>,
    ) -> Self {
        self.deferred = Some(deferred);
        self.streamer = Some(streamer);
        self.mesh_draw_lists = Some(mesh_draw_lists);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_deferred_lighting_renderer(
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

    pub(in crate::graphics::scene::scene_renderer) fn resource_streamer(
        &self,
    ) -> Option<&'a ResourceStreamer> {
        self.streamer
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
        depth_resource_name: &str,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!(
                "depth prepass graph executor for pass `{pass_name}` requires mesh draw context"
            )
        })?;
        if mesh_draw_lists.depth_prepass_commands.is_empty() {
            return Ok(());
        }
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "depth prepass graph executor for pass `{pass_name}` requires resource streamer context"
            )
        })?;
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!(
                "depth prepass graph executor for pass `{pass_name}` requires mesh pipeline context"
            )
        })?;
        let forward_shadow_receiver_bind_group = mesh_pipelines
            .create_forward_shadow_receiver_bind_group(
                self.device,
                self.shadow_atlas_resources,
                None,
                None,
                None,
            );
        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("DepthPrepass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(depth_attachment_operations(depth_attachment_ops, 1.0)),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !self
            .frame
            .render_region()
            .local_render_region()
            .apply_physical_to_render_pass(&mut pass)
        {
            return Ok(());
        }
        pass.set_bind_group(0, self.scene_bind_group, &[]);
        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(
            &mut pass,
            mesh_draw_lists.depth_prepass_stream(),
            |replayer, pass, command| {
                debug_assert_eq!(command.pipeline_kind, MeshPassPipelineKind::DepthPrepass);
                if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id)
                {
                    let pipeline = mesh_pipelines
                        .ensure_depth_prepass_pipeline_for_variant(
                            self.device,
                            streamer,
                            command.pipeline_variant_id,
                        )
                        .expect(
                            "depth prepass command must resolve a cache-backed pipeline variant",
                        );
                    pass.set_pipeline(pipeline);
                }
                replayer.bind_forward_shadow_receiver_if_needed(
                    pass,
                    &forward_shadow_receiver_bind_group,
                );
                replayer.bind_gpu_scene_if_needed(
                    pass,
                    command,
                    mesh_draw_lists.gpu_scene_bind_group,
                );
                replayer.bind_standard_material_if_needed(pass, command);
                replayer.bind_geometry_if_needed(pass, command);
                true
            },
        );
        let replay_stats = replayer.stats();
        mesh_draw_lists.replay_stats.record(replay_stats);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_shadow_atlas_to_resources(
        &mut self,
        pass_name: &str,
        shadow_atlas_resource_name: &str,
        shadow_atlas_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let shadow_atlas_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            shadow_atlas_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        if let Some(shadow_map_renderer) = self.shadow_map_renderer {
            let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
                format!(
                    "shadow atlas graph executor for pass `{pass_name}` requires mesh draw context"
                )
            })?;
            if let Some(shadow_frame_plan) = self.shadow_frame_plan {
                let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
                    format!(
                        "shadow atlas graph executor for pass `{pass_name}` requires mesh pipeline context"
                    )
                })?;
                let streamer = self.streamer.ok_or_else(|| {
                    format!(
                        "shadow atlas graph executor for pass `{pass_name}` requires resource streamer context"
                    )
                })?;
                let replay_stats = shadow_map_renderer.record_atlas_commands_with_attachment_ops(
                    self.device,
                    self.queue,
                    self.encoder,
                    pass_name,
                    shadow_atlas_view,
                    mesh_pipelines,
                    streamer,
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
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let render_region = self.render_region_for_write_resource(color_resource_name);
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires mesh draw context")
        })?;
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires mesh pipeline context")
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires resource streamer context")
        })?;
        let mixes_transparent_sprites = matches!(stage, RenderPassStage::Transparent3d)
            && self.sprite_renderer.is_some()
            && has_transparent_sprite_submissions(&self.frame.extract.sprites.phase_queue);
        let stream = mesh_draw_lists.stream_for_stage(stage);
        if stream.is_empty() && !mixes_transparent_sprites {
            return Ok(());
        }
        let light_grid_params_buffer = Self::optional_buffer_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_zbins_buffer = Self::optional_buffer_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_ZBINS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_tile_masks_buffer = Self::optional_buffer_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let replay_stats = if mixes_transparent_sprites {
            let sprite_renderer = self.sprite_renderer.ok_or_else(|| {
                format!(
                    "transparent mixed graph executor for stage `{stage:?}` requires sprite renderer context"
                )
            })?;
            BaseScenePass.record_transparent_mixed_with_attachment_ops(
                self.encoder,
                self.device,
                color_view,
                depth_view,
                self.scene_bind_group,
                mesh_draw_lists.gpu_scene_bind_group,
                mesh_draw_lists.transparent_commands,
                mesh_pipelines,
                streamer,
                sprite_renderer,
                self.frame,
                self.shadow_atlas_resources,
                render_region,
                light_grid_params_buffer,
                light_zbins_buffer,
                light_tile_masks_buffer,
                mesh_stage_attachment_ops(stage, attachment_ops),
                mesh_stage_attachment_ops(stage, depth_attachment_ops),
            )
        } else {
            BaseScenePass.record_commands_with_attachment_ops(
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
                render_region,
                light_grid_params_buffer,
                light_zbins_buffer,
                light_tile_masks_buffer,
                mesh_stage_attachment_ops(stage, attachment_ops),
                mesh_stage_attachment_ops(stage, depth_attachment_ops),
            )
        };
        mesh_draw_lists.replay_stats.record(replay_stats);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_taa_reactive_mask_mesh_to_resource(
        &mut self,
        pass_name: &str,
        taa_reactive_mask_resource_name: &str,
        scene_depth_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!(
                "TAA reactive mask mesh graph executor for pass `{pass_name}` requires mesh draw context"
            )
        })?;
        let stream = mesh_draw_lists.taa_reactive_mask_stream();
        if stream.is_empty() {
            return Ok(());
        }
        let render_region = self
            .render_region_for_write_resource(taa_reactive_mask_resource_name)
            .local_render_region();
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!(
                "TAA reactive mask mesh graph executor for pass `{pass_name}` requires mesh pipeline context"
            )
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "TAA reactive mask mesh graph executor for pass `{pass_name}` requires resource streamer context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let taa_reactive_mask_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            taa_reactive_mask_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let device = self.device;
        let forward_shadow_receiver_bind_group = mesh_pipelines
            .create_forward_shadow_receiver_bind_group(
                device,
                self.shadow_atlas_resources,
                None,
                None,
                None,
            );
        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass_name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: taa_reactive_mask_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: scene_depth_view,
                depth_ops: Some(depth_attachment_operations(
                    RenderGraphAttachmentOps::load_store(),
                    1.0,
                )),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_physical_to_render_pass(&mut pass) {
            return Ok(());
        }
        pass.set_bind_group(0, self.scene_bind_group, &[]);
        pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);

        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(&mut pass, stream, |replayer, pass, command| {
            if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id) {
                let pipeline = mesh_pipelines
                    .ensure_taa_reactive_mask_pipeline_for_variant(
                        device,
                        streamer,
                        command.pipeline_variant_id,
                    )
                    .expect(
                        "TAA reactive mask command must resolve a cache-backed pipeline variant",
                    );
                pass.set_pipeline(pipeline);
            }
            replayer.bind_gpu_scene_if_needed(pass, command, mesh_draw_lists.gpu_scene_bind_group);
            replayer.bind_standard_material_if_needed(pass, command);
            replayer.bind_geometry_if_needed(pass, command);
            true
        });
        mesh_draw_lists.replay_stats.record(replayer.stats());
        Ok(())
    }
}

pub(in crate::graphics::scene::scene_renderer) fn writes_physical_output_resource(
    resource_name: &str,
) -> bool {
    matches!(
        resource_name,
        PostProcessGraphResourceNames::FINAL_COLOR
            | PostProcessGraphResourceNames::VIEWPORT_OUTPUT
            | PostProcessGraphResourceNames::FINAL_COMPOSITED
            | PostProcessGraphResourceNames::COLOR_GRADED
            | PostProcessGraphResourceNames::EFFECT_STACKED
    )
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

#[cfg(test)]
mod tests {
    #[test]
    fn depth_prepass_binds_forward_shadow_receiver_layout_slot() {
        let source = include_str!("gpu.rs");

        assert!(source.contains("record_depth_prepass_to_resources"));
        assert!(source.contains("create_forward_shadow_receiver_bind_group"));
        assert!(source.contains("bind_forward_shadow_receiver_if_needed"));
    }
}
