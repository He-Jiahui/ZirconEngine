use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderFrameExtract, RenderPluginRendererOutputs,
};
use crate::core::math::UVec2;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::depth_attachment_operations;
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::mesh::{MeshDraw, MeshPipelineCache};
use crate::graphics::scene::scene_renderer::overlay::{
    BaseScenePass, PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::scene::scene_renderer::particle::ParticleRenderer;
use crate::graphics::scene::scene_renderer::post_process::{
    clustered_lighting_dispatch_groups, clustered_lighting_workgroup_size, ssao_dispatch_groups,
    ssao_workgroup_size, ScenePostProcessResources, SceneRuntimeFeatureFlags,
};
use crate::graphics::scene::scene_renderer::prepass::NormalPrepassPipeline;
use crate::graphics::scene::scene_renderer::sprite::SpriteRenderer;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::{RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps};

use super::super::{RenderGraphComputeDispatchRecord, RenderGraphExecutionResources};

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer) struct RenderPassMeshDrawLists<'a> {
    pub depth_prepass: &'a [&'a MeshDraw],
    pub opaque: &'a [&'a MeshDraw],
    pub alpha_mask: &'a [&'a MeshDraw],
    pub transparent: &'a [&'a MeshDraw],
    pub non_transparent: &'a [&'a MeshDraw],
}

impl<'a> RenderPassMeshDrawLists<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn for_stage(
        &self,
        stage: RenderPassStage,
    ) -> &'a [&'a MeshDraw] {
        match stage {
            RenderPassStage::DepthPrepass => self.depth_prepass,
            RenderPassStage::Opaque3d => self.opaque,
            RenderPassStage::AlphaMask3d => self.alpha_mask,
            RenderPassStage::Transparent3d => self.transparent,
            _ => &[],
        }
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
    particle_renderer: Option<&'a ParticleRenderer>,
    sprite_renderer: Option<&'a SpriteRenderer>,
    deferred: Option<&'a DeferredSceneResources>,
    streamer: Option<&'a ResourceStreamer>,
    mesh_pipelines: Option<&'a mut MeshPipelineCache>,
    mesh_draw_lists: Option<RenderPassMeshDrawLists<'a>>,
    compute_dispatches: Vec<RenderGraphComputeDispatchRecord>,
}

impl std::fmt::Debug for RenderPassGpuExecutionContext<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RenderPassGpuExecutionContext")
            .field("viewport_size", &self.frame.viewport_size)
            .field("has_post_process_stack", &self.post_process_stack.is_some())
            .field("has_overlay_renderer", &self.overlay_renderer.is_some())
            .field("has_prepass", &self.prepass.is_some())
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
            particle_renderer: None,
            sprite_renderer: None,
            deferred: None,
            streamer: None,
            mesh_pipelines: None,
            mesh_draw_lists: None,
            compute_dispatches: Vec::new(),
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

    pub(in crate::graphics::scene::scene_renderer) fn with_prepass_renderer(
        mut self,
        prepass: &'a NormalPrepassPipeline,
        mesh_draw_lists: RenderPassMeshDrawLists<'a>,
    ) -> Self {
        self.prepass = Some(prepass);
        self.mesh_draw_lists = Some(mesh_draw_lists);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_post_process_stack_context(
        mut self,
        post_process_stack: RenderPassPostProcessStackContext<'a>,
    ) -> Self {
        self.post_process_stack = Some(post_process_stack);
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
        mesh_draw_lists: RenderPassMeshDrawLists<'a>,
    ) -> Self {
        self.deferred = Some(deferred);
        self.mesh_draw_lists = Some(mesh_draw_lists);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_mesh_renderer(
        mut self,
        mesh_pipelines: &'a mut MeshPipelineCache,
        streamer: &'a ResourceStreamer,
        mesh_draw_lists: RenderPassMeshDrawLists<'a>,
    ) -> Self {
        self.mesh_pipelines = Some(mesh_pipelines);
        self.streamer = Some(streamer);
        self.mesh_draw_lists = Some(mesh_draw_lists);
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
        if mesh_draw_lists.depth_prepass.is_empty() {
            return Ok(());
        }
        prepass.record_with_attachment_ops(
            self.encoder,
            normal_view,
            depth_view,
            self.scene_bind_group,
            mesh_draw_lists.depth_prepass.iter().copied(),
            normal_attachment_ops,
            depth_attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_shadow_map_to_resource(
        &mut self,
        pass_name: &str,
        shadow_map_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let shadow_map_view = self
            .resources
            .require_texture_view(shadow_map_resource_name)?;
        let _pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass_name),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: shadow_map_view,
                depth_ops: Some(depth_attachment_operations(attachment_ops, 1.0)),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
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
        let draws = mesh_draw_lists.for_stage(stage);
        if draws.is_empty() {
            return Ok(());
        }
        BaseScenePass.record_with_attachment_ops(
            self.encoder,
            self.device,
            color_view,
            depth_view,
            self.scene_bind_group,
            draws.iter().copied(),
            mesh_pipelines,
            streamer,
            self.frame,
            mesh_stage_attachment_ops(stage, attachment_ops),
            mesh_stage_attachment_ops(stage, depth_attachment_ops),
        );
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
        deferred.record_gbuffer_geometry(
            self.encoder,
            gbuffer_albedo_view,
            gbuffer_material_view,
            depth_view,
            self.scene_bind_group,
            albedo_attachment_ops,
            material_attachment_ops,
            mesh_draw_lists.non_transparent.iter().copied(),
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_deferred_lighting_to_resources(
        &mut self,
        pass_name: &str,
        gbuffer_albedo_resource_name: &str,
        gbuffer_normal_resource_name: &str,
        gbuffer_material_resource_name: &str,
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
        deferred.execute_lighting(
            self.device,
            self.encoder,
            self.scene_bind_group,
            gbuffer_albedo_view,
            gbuffer_normal_view,
            gbuffer_material_view,
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

    pub(in crate::graphics::scene::scene_renderer) fn record_post_process_stack(
        &mut self,
        pass_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "post-process stack graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let target = stack.target;
        let history = stack.history_textures;
        let features = stack.runtime_features;
        let scene_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_COLOR)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH)?;
        let scene_normal_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL)?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                self.resources
                    .require_texture_view(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            })
            .transpose()?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)?;
        let bloom_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::BLOOM)?;
        let _final_composited_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::FINAL_COMPOSITED)?;
        let final_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::FINAL_COLOR)?;
        let global_illumination_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION)?;
        let cluster_buffer = self
            .resources
            .require_buffer(PostProcessGraphResourceNames::LIGHT_LIST)?;
        stack.post_process.execute_post_process(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            target.cluster_dimensions,
            scene_color_view,
            scene_depth_view,
            scene_normal_view,
            scene_material_view,
            ambient_occlusion_view,
            history.map(|history| &history.scene_color_view),
            history.map(|history| &history.global_illumination_view),
            bloom_view,
            final_color_view,
            global_illumination_view,
            cluster_buffer,
            self.frame,
            stack.streamer,
            features,
            stack.history_available,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_ssao_to_resources(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        depth_resource_name: &str,
        normal_resource_name: &str,
        ambient_occlusion_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "SSAO graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let normal_view = self.resources.require_texture_view(normal_resource_name)?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(ambient_occlusion_resource_name)?;
        let target = stack.target;
        let enabled = stack.runtime_features.ssao_enabled;
        let dispatch_groups = ssao_dispatch_groups(target.size);
        let workgroup_size = ssao_workgroup_size();
        stack.post_process.execute_ssao(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            depth_view,
            normal_view,
            stack
                .history_textures
                .map(|history| &history.ambient_occlusion_view),
            ambient_occlusion_view,
            enabled,
            stack.history_available,
        );
        if enabled {
            self.compute_dispatches
                .push(RenderGraphComputeDispatchRecord::new(
                    pass_name,
                    executor_id,
                    "zircon-ssao-pipeline",
                    workgroup_size,
                    dispatch_groups,
                    vec![ambient_occlusion_resource_name.to_string()],
                ));
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_clustered_lighting_to_resources(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        depth_resource_name: &str,
        light_list_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "clustered lighting graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let _depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let light_list_buffer = self.resources.require_buffer(light_list_resource_name)?;
        let target = stack.target;
        let enabled = stack.runtime_features.clustered_lighting_enabled;
        let dispatch_groups = clustered_lighting_dispatch_groups(target.cluster_dimensions);
        let workgroup_size = clustered_lighting_workgroup_size();
        stack.post_process.execute_clustered_lighting(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            target.cluster_dimensions,
            light_list_buffer,
            target.cluster_buffer_bytes,
            &self.frame.extract.lighting.directional_lights,
            enabled,
        );
        if enabled {
            self.compute_dispatches
                .push(RenderGraphComputeDispatchRecord::new(
                    pass_name,
                    executor_id,
                    "zircon-cluster-pipeline",
                    workgroup_size,
                    dispatch_groups,
                    vec![light_list_resource_name.to_string()],
                ));
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_bloom_to_resources(
        &mut self,
        pass_name: &str,
        scene_color_resource_name: &str,
        bloom_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "bloom graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let scene_color_view = self
            .resources
            .require_texture_view(scene_color_resource_name)?;
        let bloom_view = self.resources.require_texture_view(bloom_resource_name)?;
        let target = stack.target;
        stack.post_process.execute_bloom(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            scene_color_view,
            bloom_view,
            self.frame.extract.post_process.bloom,
            stack.runtime_features.bloom_enabled,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_depth_of_field_prepare_to_resources(
        &mut self,
        pass_name: &str,
        scene_depth_resource_name: &str,
        coc_resource_name: &str,
        bokeh_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "depth-of-field prepare graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let _scene_depth_view = self
            .resources
            .require_texture_view(scene_depth_resource_name)?;
        let coc_view = self.resources.require_texture_view(coc_resource_name)?;
        let bokeh_view = self.resources.require_texture_view(bokeh_resource_name)?;
        stack
            .post_process
            .execute_depth_of_field_prepare(self.encoder, coc_view, bokeh_view);
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

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer) struct RenderPassPostProcessStackContext<'a> {
    post_process: &'a ScenePostProcessResources,
    target: &'a OffscreenTarget,
    streamer: &'a ResourceStreamer,
    runtime_features: SceneRuntimeFeatureFlags,
    history_textures: Option<&'a SceneFrameHistoryTextures>,
    history_available: bool,
    material_gbuffer_valid: bool,
}

impl<'a> RenderPassPostProcessStackContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn new(
        post_process: &'a ScenePostProcessResources,
        target: &'a OffscreenTarget,
        streamer: &'a ResourceStreamer,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&'a SceneFrameHistoryTextures>,
        history_available: bool,
    ) -> Self {
        Self {
            post_process,
            target,
            streamer,
            runtime_features,
            history_textures,
            history_available,
            material_gbuffer_valid: false,
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_material_gbuffer_valid(
        mut self,
        material_gbuffer_valid: bool,
    ) -> Self {
        self.material_gbuffer_valid = material_gbuffer_valid;
        self
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
