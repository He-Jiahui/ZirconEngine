use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessGraphResourceNames, RenderFrameExtract,
    RenderPluginRendererOutputs, ShaderQualityTier,
};
use crate::core::math::UVec2;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::overlay::{
    PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::scene::scene_renderer::particle::ParticleRenderer;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::scene::scene_renderer::shadow::{ShadowFramePlan, ShadowMapRenderer};
use crate::graphics::scene::scene_renderer::sprite::SpriteRenderer;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::types::{
    ViewportCameraStackAttachmentPolicy, ViewportRenderFrame, ViewportRenderRegion,
};
use crate::graphics::visibility::HzbOcclusionCullReport;

use super::super::{
    RenderGraphComputeDispatchRecord, RenderGraphExecutionResources, RenderGraphLightGridReport,
};
use super::RgResourceResolver;

mod deferred;
mod hzb_occlusion;
mod mesh_command_lists;
mod mesh_recording;
mod oit;
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
    pub resources: &'a RenderGraphExecutionResources,
    pub plugin_outputs: &'a mut RenderPluginRendererOutputs,
    resource_resolver: Option<RgResourceResolver<'a>>,
    pub(in crate::graphics::scene::scene_renderer) screen_space_ui_renderer:
        Option<&'a mut ScreenSpaceUiRenderer>,
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
            .field(
                "has_screen_space_ui_renderer",
                &self.screen_space_ui_renderer.is_some(),
            )
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
        resources: &'a RenderGraphExecutionResources,
        plugin_outputs: &'a mut RenderPluginRendererOutputs,
        screen_space_ui_renderer: Option<&'a mut ScreenSpaceUiRenderer>,
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
        resources: &'a RenderGraphExecutionResources,
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
            Some(screen_space_ui_renderer),
        )
    }

    pub fn frame_extract(&self) -> &RenderFrameExtract {
        &self.frame.extract
    }

    pub fn viewport_size(&self) -> UVec2 {
        self.frame.viewport_size
    }

    pub fn shader_quality(&self) -> ShaderQualityTier {
        self.frame.shader_quality()
    }

    pub fn previous_motion_vector_camera(
        &self,
    ) -> Option<&crate::core::framework::render::ViewportCameraSnapshot> {
        self.frame.previous_motion_vector_camera()
    }

    pub fn history_available(&self) -> bool {
        self.post_process_stack
            .map(|stack| stack.history_available)
            .unwrap_or(false)
    }

    pub fn hybrid_gi_history_available(&self) -> bool {
        self.post_process_stack
            .is_some_and(RenderPassPostProcessStackContext::hybrid_gi_history_available)
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
        mesh_draw_lists: RenderPassMeshCommandLists<'a>,
    ) -> Self {
        self.mesh_pipelines = Some(mesh_pipelines);
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

#[cfg(test)]
mod tests;
