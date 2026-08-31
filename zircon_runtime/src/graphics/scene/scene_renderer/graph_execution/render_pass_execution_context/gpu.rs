use std::cell::Cell;

use crate::core::framework::render::{
    DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA, MotionVectorCameraStatus, PostProcessExtract,
    PostProcessGraphResourceNames, RenderFrameExtract, RenderPassNativeResourceCreateMetrics,
    RenderPipelinePhase, RenderPluginRendererOutputs, ShaderQualityTier, VolumetricFogSettings,
};
use crate::core::math::UVec2;
use crate::graphics::backend::{
    GpuPipelineStatisticsScope, GpuPipelineStatisticsTimer, ViewportSurface,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::environment::IblBakeWgpuPipelineCache;
use crate::graphics::scene::scene_renderer::history::SceneHistoryWriteIntent;
use crate::graphics::scene::scene_renderer::hzb::{HzbOcclusionCuller, HzbOcclusionParamsCommit};
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::overlay::{
    PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::scene::scene_renderer::particle::ParticleRenderer;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::scene::scene_renderer::shadow::{ShadowFramePlan, ShadowMapRenderer};
use crate::graphics::scene::scene_renderer::sprite::SpriteRenderer;
use crate::graphics::scene::scene_renderer::ui::{
    ScreenSpaceUiPreparedUpload, ScreenSpaceUiRenderer,
};
use crate::graphics::types::{
    ViewportCameraStackAttachmentPolicy, ViewportRenderFrame, ViewportRenderRegion,
};
use crate::graphics::visibility::HzbOcclusionCullReport;
use zr_rhi_wgpu::{WgpuBufferUploadBatch, WgpuTextureUploadBatch};

use super::super::{
    RenderGraphComputeDispatchRecord, RenderGraphExecutionResources, RenderGraphLightGridReport,
    RenderPassDeviceEpoch,
};
use super::RgResourceResolver;

mod buffer_uploads;
mod deferred;
mod half_res_transparency;
mod hzb_occlusion;
mod mesh_command_lists;
mod mesh_recording;
mod native;
mod oit;
mod output_target;
mod particle;
mod post_process;
mod reports;
mod resource_lookup;
mod surface;

pub use buffer_uploads::{RenderPassBufferUploadRecorder, RenderPassBufferUploadSink};
pub(in crate::graphics::scene::scene_renderer) use mesh_command_lists::RenderPassMeshCommandLists;
pub use native::{
    RenderPassGpuNativeContext, RenderPassGpuRecordingContext, RenderPassGpuResourceFactory,
};
pub use particle::ParticleGpuTransparentDrawContext;
pub(in crate::graphics::scene::scene_renderer) use post_process::RenderPassPostProcessStackContext;

pub struct RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) device: &'a wgpu::Device,
    pub(in crate::graphics::scene::scene_renderer) encoder: &'a mut wgpu::CommandEncoder,
    frame: &'a ViewportRenderFrame,
    scene_bind_group_layout: &'a wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer) scene_bind_group: &'a wgpu::BindGroup,
    pub(in crate::graphics::scene::scene_renderer) resources: &'a RenderGraphExecutionResources,
    pub(in crate::graphics::scene::scene_renderer) plugin_outputs:
        &'a mut RenderPluginRendererOutputs,
    surface_frame: Option<(
        &'a ViewportSurface,
        &'a zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget,
    )>,
    surface_present_error: Option<crate::graphics::types::GraphicsError>,
    output_target_writeback_plan: crate::core::framework::render::RenderCameraTargetWritebackReport,
    output_target_writeback_report:
        Option<crate::core::framework::render::RenderCameraTargetWritebackReport>,
    output_target_writeback_error: Option<crate::graphics::types::GraphicsError>,
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
    pub(in crate::graphics::scene::scene_renderer) ibl_bake_pipeline_cache:
        Option<&'a mut IblBakeWgpuPipelineCache>,
    pub(in crate::graphics::scene::scene_renderer) mesh_draw_lists:
        Option<RenderPassMeshCommandLists<'a>>,
    hzb_occlusion_culler: Option<&'a HzbOcclusionCuller>,
    pipeline_statistics_timer: Option<&'a mut GpuPipelineStatisticsTimer>,
    compute_dispatches: Vec<RenderGraphComputeDispatchRecord>,
    hzb_occlusion_cull_report: Option<HzbOcclusionCullReport>,
    light_grid_report: Option<RenderGraphLightGridReport>,
    taa_reactive_mask_encoded_pass_count: usize,
    taa_reactive_mask_encoded_write_bytes: u64,
    taa_resolve_bind_group_create_count: usize,
    native_resource_creates: Cell<RenderPassNativeResourceCreateMetrics>,
    history_writes: SceneHistoryWriteIntent,
    motion_vector_camera_status: MotionVectorCameraStatus,
    half_resolution_transparency_depth_sigma: u16,
    buffer_uploads: WgpuBufferUploadBatch,
    texture_uploads: WgpuTextureUploadBatch,
    screen_space_ui_upload_commits: Vec<ScreenSpaceUiPreparedUpload>,
    hzb_occlusion_params_commits: Vec<HzbOcclusionParamsCommit>,
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
            encoder,
            frame,
            scene_bind_group_layout,
            target_format,
            depth_format,
            scene_bind_group,
            resources,
            plugin_outputs,
            surface_frame: None,
            surface_present_error: None,
            output_target_writeback_plan: Default::default(),
            output_target_writeback_report: None,
            output_target_writeback_error: None,
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
            ibl_bake_pipeline_cache: None,
            mesh_draw_lists: None,
            hzb_occlusion_culler: None,
            pipeline_statistics_timer: None,
            compute_dispatches: Vec::new(),
            hzb_occlusion_cull_report: None,
            light_grid_report: None,
            taa_reactive_mask_encoded_pass_count: 0,
            taa_reactive_mask_encoded_write_bytes: 0,
            taa_resolve_bind_group_create_count: 0,
            native_resource_creates: Cell::new(RenderPassNativeResourceCreateMetrics::default()),
            history_writes: SceneHistoryWriteIntent::default(),
            motion_vector_camera_status: MotionVectorCameraStatus::NotRequested,
            half_resolution_transparency_depth_sigma: DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            buffer_uploads: WgpuBufferUploadBatch::new(),
            texture_uploads: WgpuTextureUploadBatch::new(),
            screen_space_ui_upload_commits: Vec::new(),
            hzb_occlusion_params_commits: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn new_for_test(
        device: &'a wgpu::Device,
        _queue: &'a wgpu::Queue,
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

    pub fn post_process(&self) -> &PostProcessExtract {
        self.frame.post_process()
    }

    pub fn volumetric_fog(&self) -> VolumetricFogSettings {
        self.frame.volumetric_fog()
    }

    pub fn viewport_size(&self) -> UVec2 {
        self.frame.viewport_size
    }

    pub fn shader_quality(&self) -> ShaderQualityTier {
        self.frame.shader_quality()
    }

    /// Returns the device identity that materialized this graph execution.
    /// Executors must reject persistent native caches when this fact is absent.
    pub fn device_epoch(&self) -> Option<RenderPassDeviceEpoch> {
        self.resources.device_epoch()
    }

    /// Borrows the native command-recording handles for one graph pass operation.
    ///
    /// The capability is intentionally short-lived and does not include the graph resource table
    /// or plugin output mailbox. Callers should resolve graph resources through the typed lookup
    /// methods before requesting this capability.
    pub fn native_context(&mut self) -> RenderPassGpuNativeContext<'a, '_> {
        RenderPassGpuNativeContext {
            device: self.device,
            encoder: self.encoder,
            scene_bind_group: self.scene_bind_group,
            scene_bind_group_layout: self.scene_bind_group_layout,
            native_resource_creates: &self.native_resource_creates,
        }
    }

    fn record_native_resource_create(
        &self,
        record: impl FnOnce(&mut RenderPassNativeResourceCreateMetrics),
    ) {
        let mut metrics = self.native_resource_creates.get();
        record(&mut metrics);
        self.native_resource_creates.set(metrics);
    }

    /// Returns the mutable plugin output mailbox for a graph pass completion projection.
    pub fn plugin_outputs_mut(&mut self) -> &mut RenderPluginRendererOutputs {
        self.plugin_outputs
    }

    /// Returns the read-only plugin output mailbox for a graph pass observation.
    pub fn plugin_outputs(&self) -> &RenderPluginRendererOutputs {
        self.plugin_outputs
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_surface_frame(
        mut self,
        surface_frame: Option<(
            &'a ViewportSurface,
            &'a zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget,
        )>,
    ) -> Self {
        self.surface_frame = surface_frame;
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_output_target_writeback_plan(
        mut self,
        plan: crate::core::framework::render::RenderCameraTargetWritebackReport,
    ) -> Self {
        self.output_target_writeback_plan = plan;
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_surface_present_error(
        &mut self,
    ) -> Option<crate::graphics::types::GraphicsError> {
        self.surface_present_error.take()
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_output_target_writeback_report(
        &mut self,
    ) -> Option<crate::core::framework::render::RenderCameraTargetWritebackReport> {
        self.output_target_writeback_report.take()
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_output_target_writeback_error(
        &mut self,
    ) -> Option<crate::graphics::types::GraphicsError> {
        self.output_target_writeback_error.take()
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_half_resolution_transparency_depth_sigma(
        mut self,
        depth_sigma: u16,
    ) -> Self {
        self.half_resolution_transparency_depth_sigma = depth_sigma.max(1);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_pipeline_statistics_timer(
        mut self,
        pipeline_statistics_timer: &'a mut GpuPipelineStatisticsTimer,
    ) -> Self {
        self.pipeline_statistics_timer = Some(pipeline_statistics_timer);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn reserve_pipeline_statistics_scope(
        &mut self,
        pass_name: &str,
    ) -> Option<GpuPipelineStatisticsScope> {
        self.pipeline_statistics_timer
            .as_deref_mut()
            .and_then(|timer| timer.reserve_pass(pass_name))
    }

    pub fn previous_motion_vector_camera(
        &self,
    ) -> Option<&crate::core::framework::render::ViewportCameraSnapshot> {
        self.frame.previous_motion_vector_camera()
    }

    pub fn history_available(
        &self,
        domain: crate::graphics::scene::scene_renderer::history::SceneHistoryDomain,
    ) -> bool {
        self.post_process_stack
            .map(|stack| stack.history_available(domain))
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
        let scene_region = self
            .frame
            .render_region_for_phase(RenderPipelinePhase::SceneLinear)
            .unwrap_or_else(|| self.render_region())
            .local_render_region();
        let phase_region = PostProcessGraphResourceNames::view_family_pipeline_phase(resource_name)
            .and_then(|phase| self.frame.render_region_for_phase(phase))
            .unwrap_or(scene_region);
        let resource_region = if writes_physical_output_resource(resource_name) {
            phase_region
        } else {
            phase_region.local_render_region()
        };
        if matches!(
            resource_name,
            PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR
                | PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH
        ) {
            let local_size = resource_region.local_size();
            let half_size = UVec2::new(
                local_size.x.saturating_add(1) / 2,
                local_size.y.saturating_add(1) / 2,
            );
            return resource_region.with_local_size(half_size);
        }
        resource_region
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

    pub(in crate::graphics::scene::scene_renderer) fn with_ibl_bake_pipeline_cache(
        mut self,
        ibl_bake_pipeline_cache: &'a mut IblBakeWgpuPipelineCache,
    ) -> Self {
        self.ibl_bake_pipeline_cache = Some(ibl_bake_pipeline_cache);
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

impl RenderPassGpuResourceFactory for RenderPassGpuExecutionContext<'_> {
    fn create_buffer_init(
        &self,
        descriptor: &wgpu::util::BufferInitDescriptor<'_>,
    ) -> wgpu::Buffer {
        self.record_native_resource_create(RenderPassNativeResourceCreateMetrics::record_buffer);
        wgpu::util::DeviceExt::create_buffer_init(self.device, descriptor)
    }

    fn create_bind_group(&self, descriptor: &wgpu::BindGroupDescriptor<'_>) -> wgpu::BindGroup {
        self.record_native_resource_create(
            RenderPassNativeResourceCreateMetrics::record_bind_group,
        );
        self.device.create_bind_group(descriptor)
    }

    fn create_bind_group_layout(
        &self,
        descriptor: &wgpu::BindGroupLayoutDescriptor<'_>,
    ) -> wgpu::BindGroupLayout {
        self.record_native_resource_create(
            RenderPassNativeResourceCreateMetrics::record_bind_group_layout,
        );
        self.device.create_bind_group_layout(descriptor)
    }

    fn create_shader_module(
        &self,
        descriptor: wgpu::ShaderModuleDescriptor<'_>,
    ) -> wgpu::ShaderModule {
        self.record_native_resource_create(
            RenderPassNativeResourceCreateMetrics::record_shader_module,
        );
        self.device.create_shader_module(descriptor)
    }

    fn create_pipeline_layout(
        &self,
        descriptor: &wgpu::PipelineLayoutDescriptor<'_>,
    ) -> wgpu::PipelineLayout {
        self.record_native_resource_create(
            RenderPassNativeResourceCreateMetrics::record_pipeline_layout,
        );
        self.device.create_pipeline_layout(descriptor)
    }

    fn create_compute_pipeline(
        &self,
        descriptor: &wgpu::ComputePipelineDescriptor<'_>,
    ) -> wgpu::ComputePipeline {
        self.record_native_resource_create(
            RenderPassNativeResourceCreateMetrics::record_compute_pipeline,
        );
        self.device.create_compute_pipeline(descriptor)
    }

    fn create_render_pipeline(
        &self,
        descriptor: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> wgpu::RenderPipeline {
        self.record_native_resource_create(
            RenderPassNativeResourceCreateMetrics::record_render_pipeline,
        );
        self.device.create_render_pipeline(descriptor)
    }
}

pub(in crate::graphics::scene::scene_renderer) fn writes_physical_output_resource(
    resource_name: &str,
) -> bool {
    matches!(
        resource_name,
        PostProcessGraphResourceNames::FINAL_COLOR | PostProcessGraphResourceNames::VIEWPORT_OUTPUT
    )
}

#[cfg(test)]
mod tests;
