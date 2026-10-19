use std::cell::Cell;

use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessExtract, PostProcessGraphResourceNames,
    RenderFrameExtract, RenderPassNativeResourceCreateMetrics, RenderPipelinePhase,
    RenderPluginRendererOutputs, RenderVirtualGeometryDebugSnapshot, ShaderQualityTier,
    VolumetricFogSettings, DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
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

#[path = "../../context/gpu/buffer_uploads.rs"]
mod buffer_uploads;
#[path = "gpu/deferred.rs"]
mod deferred;
#[path = "../../context/gpu/half_res_transparency.rs"]
mod half_res_transparency;
#[path = "../../context/gpu/hzb_occlusion.rs"]
mod hzb_occlusion;
#[path = "../../context/gpu/mesh_command_lists.rs"]
mod mesh_command_lists;
#[path = "../../context/gpu/mesh_recording.rs"]
mod mesh_recording;
#[path = "gpu/native.rs"]
mod native;
#[path = "gpu/oit.rs"]
mod oit;
#[path = "../../context/gpu/output_target.rs"]
mod output_target;
#[path = "gpu/particle.rs"]
mod particle;
#[path = "gpu/post.rs"]
mod post_process;
#[path = "gpu/reports.rs"]
mod reports;
#[path = "../../context/gpu/resource_lookup.rs"]
mod resource_lookup;
#[path = "gpu/surface.rs"]
mod surface;
#[path = "gpu/virtual_geometry_probe.rs"]
mod virtual_geometry_probe;

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
    virtual_geometry_readbacks: Vec<VirtualGeometryNativeReadbackPacket>,
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
            virtual_geometry_readbacks: Vec::new(),
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

    pub fn reserve_virtual_geometry_readback(
        &self,
        bytes: u64,
    ) -> Result<Option<VirtualGeometryReadbackReservation>, String> {
        let Some(scope) = self.frame.virtual_geometry_readback_scope.as_ref() else {
            return Ok(None);
        };
        let epoch = self
            .device_epoch()
            .ok_or("virtual geometry readback requires materialized device epoch")?;
        if bytes == 0
            || bytes > VirtualGeometryReadbackBudget::LIMIT_BYTES
            || bytes % wgpu::COPY_BUFFER_ALIGNMENT != 0
        {
            return Err(
                "virtual geometry readback length is not a bounded aligned payload".to_string(),
            );
        }
        let Some(credit) = scope.try_acquire_credit(bytes) else {
            return Ok(None);
        };
        Ok(Some(VirtualGeometryReadbackReservation { credit, epoch }))
    }

    pub fn enqueue_reserved_virtual_geometry_readback(
        &mut self,
        reservation: VirtualGeometryReadbackReservation,
        staging: wgpu::Buffer,
        range: std::ops::Range<u64>,
        decoder: VirtualGeometryReadbackDecoder,
    ) -> Result<(), String> {
        self.enqueue_reserved_virtual_geometry_readback_with_inspection(
            reservation,
            staging,
            range,
            decoder,
            None,
        )
    }

    pub fn enqueue_reserved_virtual_geometry_readback_with_inspection(
        &mut self,
        reservation: VirtualGeometryReadbackReservation,
        staging: wgpu::Buffer,
        range: std::ops::Range<u64>,
        decoder: VirtualGeometryReadbackDecoder,
        inspection_decoder: Option<
            VirtualGeometryNativeInspectionDecoder<
                crate::core::framework::render::RenderVirtualGeometryReadbackOutputs,
            >,
        >,
    ) -> Result<(), String> {
        let Some(scope) = self.frame.virtual_geometry_readback_scope.as_ref() else {
            return Err("virtual geometry readback frame scope is absent".to_string());
        };
        if reservation.credit.scope.scope_id != scope.scope_id
            || self.device_epoch() != Some(reservation.epoch)
            || range.start != 0
            || range.end != reservation.credit.bytes
            || staging.size() != reservation.credit.bytes
            || range.end.checked_sub(range.start) != Some(reservation.credit.bytes)
            || range.end > staging.size()
            || !staging
                .usage()
                .contains(wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST)
        {
            return Err("virtual geometry readback reservation or staging range does not match the recording frame".to_string());
        }
        self.virtual_geometry_readbacks
            .push(VirtualGeometryNativeReadbackPacket {
                scope: scope.clone(),
                epoch: reservation.epoch,
                staging,
                range,
                decoder,
                inspection_decoder,
                credit: reservation.credit,
                native_probe: None,
            });
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_virtual_geometry_readbacks(
        &mut self,
    ) -> Vec<VirtualGeometryNativeReadbackPacket> {
        std::mem::take(&mut self.virtual_geometry_readbacks)
    }

    /// Retains current-frame physical inputs until the single terminal map consumes them.
    pub fn register_virtual_geometry_native_inspection(
        &self,
        sections: VirtualGeometryNativeInspectionSections,
        decoder: VirtualGeometryNativeInspectionDecoder<
            crate::core::framework::render::RenderVirtualGeometryReadbackOutputs,
        >,
    ) -> Result<(), String> {
        let scope = self
            .frame
            .virtual_geometry_readback_scope
            .as_ref()
            .ok_or("virtual geometry inspection frame scope is absent")?;
        let epoch = self
            .device_epoch()
            .ok_or("virtual geometry inspection device epoch is absent")?;
        if sections.is_empty() || sections.len() > 12 {
            return Err("virtual geometry inspection section count is invalid".into());
        }
        let mut previous = None;
        let mut bytes = 0_u64;
        for (id, buffer, range) in &sections {
            let length = range
                .end
                .checked_sub(range.start)
                .ok_or("virtual geometry inspection section range is reversed")?;
            if *id > 11
                || previous.is_some_and(|value| value >= *id)
                || length == 0
                || range.start % wgpu::COPY_BUFFER_ALIGNMENT != 0
                || length % wgpu::COPY_BUFFER_ALIGNMENT != 0
                || range.end > buffer.size()
                || !buffer.usage().contains(wgpu::BufferUsages::COPY_SRC)
            {
                return Err(
                    "virtual geometry inspection section identity, range or usage is invalid"
                        .into(),
                );
            }
            previous = Some(*id);
            bytes = bytes
                .checked_add(length)
                .ok_or("virtual geometry inspection physical length overflow")?;
        }
        if sections[0].0 != 0 || bytes > VirtualGeometryReadbackBudget::LIMIT_BYTES {
            return Err("virtual geometry inspection lacks its bounded cull-input section".into());
        }
        let mut slot = scope
            .inspection
            .lock()
            .map_err(|_| "virtual geometry inspection slot lock is poisoned")?;
        if !matches!(*slot, VirtualGeometryInspectionState::Empty) {
            return Err("virtual geometry inspection was already recorded or consumed".into());
        }
        *slot = VirtualGeometryInspectionState::Pending(RecordedVirtualGeometryInspection {
            epoch,
            sections,
            decoder,
        });
        Ok(())
    }

    /// Takes the physical section leases once, preserving the producer epoch and frame scope.
    pub fn take_virtual_geometry_native_inspection(
        &self,
    ) -> Result<
        Option<(
            VirtualGeometryNativeInspectionSections,
            VirtualGeometryNativeInspectionDecoder<
                crate::core::framework::render::RenderVirtualGeometryReadbackOutputs,
            >,
        )>,
        String,
    > {
        let scope = self
            .frame
            .virtual_geometry_readback_scope
            .as_ref()
            .ok_or("virtual geometry inspection frame scope is absent")?;
        let mut slot = scope
            .inspection
            .lock()
            .map_err(|_| "virtual geometry inspection slot lock is poisoned")?;
        match std::mem::replace(&mut *slot, VirtualGeometryInspectionState::Consumed) {
            VirtualGeometryInspectionState::Empty => Ok(None),
            VirtualGeometryInspectionState::Consumed => {
                Err("virtual geometry inspection was consumed more than once".into())
            }
            VirtualGeometryInspectionState::Pending(recorded) => {
                if self.device_epoch() != Some(recorded.epoch) {
                    return Err("virtual geometry inspection producer device epoch changed".into());
                }
                Ok(Some((recorded.sections, recorded.decoder)))
            }
        }
    }

    pub fn virtual_geometry_execution_data(
        &self,
    ) -> Option<&crate::core::framework::render::RenderVirtualGeometryExecutionData> {
        self.frame.virtual_geometry_execution_data.as_deref()
    }

    pub fn virtual_geometry_execution_inputs(
        &self,
    ) -> &crate::core::framework::render::RenderVirtualGeometryExecutionInputs {
        &self.frame.virtual_geometry_execution_inputs
    }

    pub fn virtual_geometry_debug_snapshot(&self) -> Option<&RenderVirtualGeometryDebugSnapshot> {
        self.frame.virtual_geometry_debug_snapshot.as_deref()
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
    pub(crate) fn subsurface_profiles(
        &self,
    ) -> &[crate::core::framework::render::SubsurfaceProfileData] {
        self.frame.subsurface_profiles()
    }

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
#[path = "gpu/tests/cases.rs"]
mod tests;

/// Native feedback allocation remains charged until its decoded payload is consumed or dropped.
#[derive(Debug)]
pub(in crate::graphics) struct VirtualGeometryReadbackBudget {
    live_bytes: std::sync::atomic::AtomicU64,
    next_scope: std::sync::atomic::AtomicU64,
    skipped: std::sync::atomic::AtomicU64,
}

impl VirtualGeometryReadbackBudget {
    const LIMIT_BYTES: u64 = 64 * 1024 * 1024;

    pub(in crate::graphics) fn new() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            live_bytes: std::sync::atomic::AtomicU64::new(0),
            next_scope: std::sync::atomic::AtomicU64::new(0),
            skipped: std::sync::atomic::AtomicU64::new(0),
        })
    }

    pub(in crate::graphics) fn scope(
        self: &std::sync::Arc<Self>,
        viewport: crate::core::framework::render::RenderViewportHandle,
        viewport_generation: u64,
        frame_generation: u64,
        slots: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> Option<VirtualGeometryReadbackScope> {
        use std::sync::atomic::Ordering;
        let scope_id = self
            .next_scope
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(1)
            })
            .ok()?
            .checked_add(1)?;
        Some(VirtualGeometryReadbackScope {
            scope_id,
            viewport,
            viewport_generation,
            frame_generation,
            budget: std::sync::Arc::clone(self),
            slots,
            inspection: std::sync::Arc::new(std::sync::Mutex::new(
                VirtualGeometryInspectionState::Empty,
            )),
        })
    }
}

/// Physical source sections retain their native buffer leases until terminal mapping completes.
pub type VirtualGeometryNativeInspectionSections = Vec<(u32, wgpu::Buffer, std::ops::Range<u64>)>;
pub type VirtualGeometryNativeInspectionDecoder<T> = Box<
    dyn FnOnce(
            &[u8],
            &T,
        ) -> Result<
            crate::core::framework::render::RenderVirtualGeometryNativeReadbackInspection,
            String,
        > + Send
        + 'static,
>;

struct RecordedVirtualGeometryInspection {
    epoch: RenderPassDeviceEpoch,
    sections: VirtualGeometryNativeInspectionSections,
    decoder: VirtualGeometryNativeInspectionDecoder<
        crate::core::framework::render::RenderVirtualGeometryReadbackOutputs,
    >,
}
impl std::fmt::Debug for RecordedVirtualGeometryInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordedVirtualGeometryInspection")
            .field("epoch", &self.epoch)
            .field("section_count", &self.sections.len())
            .finish_non_exhaustive()
    }
}
#[derive(Debug)]
enum VirtualGeometryInspectionState {
    Empty,
    Pending(RecordedVirtualGeometryInspection),
    Consumed,
}

#[derive(Clone, Debug)]
pub(in crate::graphics) struct VirtualGeometryReadbackScope {
    pub(in crate::graphics) scope_id: u64,
    pub(in crate::graphics) viewport: crate::core::framework::render::RenderViewportHandle,
    pub(in crate::graphics) viewport_generation: u64,
    pub(in crate::graphics) frame_generation: u64,
    budget: std::sync::Arc<VirtualGeometryReadbackBudget>,
    slots: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    inspection: std::sync::Arc<std::sync::Mutex<VirtualGeometryInspectionState>>,
}

#[derive(Debug)]
struct VirtualGeometryReadbackCredit {
    scope: VirtualGeometryReadbackScope,
    bytes: u64,
}

impl Drop for VirtualGeometryReadbackCredit {
    fn drop(&mut self) {
        use std::sync::atomic::Ordering;
        self.scope
            .budget
            .live_bytes
            .fetch_sub(self.bytes, Ordering::AcqRel);
        self.scope.slots.fetch_sub(1, Ordering::AcqRel);
    }
}

/// The admitted payload and viewport slot must be reserved before allocating staging storage.
pub struct VirtualGeometryReadbackReservation {
    credit: VirtualGeometryReadbackCredit,
    epoch: RenderPassDeviceEpoch,
}

pub type VirtualGeometryReadbackDecoder =
    NativeReadbackDecoder<crate::core::framework::render::RenderVirtualGeometryReadbackOutputs>;
type NativeReadbackDecoder<T> = Box<dyn FnOnce(&[u8]) -> Result<T, String> + Send + 'static>;
pub(in crate::graphics) type VirtualGeometryNativeReadbackPacket =
    NativeReadbackPacket<crate::core::framework::render::RenderVirtualGeometryReadbackOutputs>;
pub(in crate::graphics) type VirtualGeometryReadyReadback =
    NativeReadyReadback<crate::core::framework::render::RenderVirtualGeometryReadbackOutputs>;
pub(in crate::graphics) type VirtualGeometryTicketedReadback =
    NativeTicketedReadback<crate::core::framework::render::RenderVirtualGeometryReadbackOutputs>;
pub(in crate::graphics) struct NativeReadbackPacket<T> {
    pub(in crate::graphics) scope: VirtualGeometryReadbackScope,
    pub(in crate::graphics) epoch: RenderPassDeviceEpoch,
    staging: wgpu::Buffer,
    range: std::ops::Range<u64>,
    decoder: NativeReadbackDecoder<T>,
    inspection_decoder: Option<VirtualGeometryNativeInspectionDecoder<T>>,
    credit: VirtualGeometryReadbackCredit,
    native_probe: Option<std::sync::Arc<virtual_geometry_probe::NativeProbe>>,
}
pub(in crate::graphics) struct NativeReadyReadback<T> {
    pub(in crate::graphics) outputs: T,
    pub(in crate::graphics) inspection:
        Option<crate::core::framework::render::RenderVirtualGeometryNativeReadbackInspection>,
    _credit: Option<VirtualGeometryReadbackCredit>,
    native_probe: Option<std::sync::Arc<virtual_geometry_probe::NativeProbe>>,
}
pub(in crate::graphics) struct NativeTicketedReadback<T> {
    pub(in crate::graphics) scope: VirtualGeometryReadbackScope,
    pub(in crate::graphics) epoch: RenderPassDeviceEpoch,
    pub(in crate::graphics) ticket: crate::rhi::SubmissionTicket,
    submission_completed: bool,
    completion: std::sync::Arc<std::sync::Mutex<Option<Result<NativeReadyReadback<T>, String>>>>,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl<T: Send + 'static> NativeReadbackPacket<T> {
    /// Actual submission transfers physical credit and source leases to the map callback.
    pub(in crate::graphics) fn bind_ticket(
        self,
        ticket: crate::rhi::SubmissionTicket,
    ) -> NativeTicketedReadback<T> {
        if let Some(probe) = &self.native_probe {
            probe.bind_origin(ticket);
        }
        let completion = std::sync::Arc::new(std::sync::Mutex::new(None));
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let result = NativeTicketedReadback {
            scope: self.scope.clone(),
            epoch: self.epoch,
            ticket,
            submission_completed: false,
            completion: std::sync::Arc::clone(&completion),
            cancelled: std::sync::Arc::clone(&cancelled),
        };
        let NativeReadbackPacket {
            staging,
            range,
            decoder,
            inspection_decoder,
            credit,
            native_probe,
            ..
        } = self;
        let callback_buffer = staging.clone();
        staging
            .slice(range.clone())
            .map_async(wgpu::MapMode::Read, move |mapped| {
                use std::sync::atomic::Ordering;
                let delivery = match mapped {
                    Ok(()) if !cancelled.load(Ordering::Acquire) => {
                        let view = callback_buffer.slice(range).get_mapped_range();
                        let decoded = decoder(&view).and_then(|outputs| {
                            let inspection = inspection_decoder
                                .map(|decode| decode(&view, &outputs))
                                .transpose()?;
                            Ok((outputs, inspection))
                        });
                        drop(view);
                        callback_buffer.unmap();
                        decoded.map(|(outputs, inspection)| NativeReadyReadback {
                            outputs,
                            inspection,
                            _credit: Some(credit),
                            native_probe,
                        })
                    }
                    Ok(()) => {
                        callback_buffer.unmap();
                        Err("virtual geometry readback cancelled".to_string())
                    }
                    Err(error) => Err(format!("virtual geometry native mapping failed: {error}")),
                };
                if !cancelled.load(Ordering::Acquire) {
                    *completion.lock().unwrap_or_else(|e| e.into_inner()) = Some(delivery);
                }
            });
        result
    }
}
impl<T> NativeTicketedReadback<T> {
    pub(in crate::graphics) fn observe_status(
        &mut self,
        status: Option<crate::rhi::SubmissionStatus>,
    ) {
        match status {
            Some(crate::rhi::SubmissionStatus::Completed) => self.submission_completed = true,
            Some(status) if status.is_terminal() => {
                self.cancelled
                    .store(true, std::sync::atomic::Ordering::Release);
                *self.completion.lock().unwrap_or_else(|e| e.into_inner()) = Some(Err(
                    "virtual geometry scene submission did not complete".to_string(),
                ));
            }
            _ => {}
        }
    }
    pub(in crate::graphics) fn take_ready(
        &mut self,
    ) -> Option<Result<NativeReadyReadback<T>, String>> {
        let mut delivery = self.completion.lock().unwrap_or_else(|e| e.into_inner());
        if self.submission_completed || delivery.as_ref().is_some_and(Result::is_err) {
            delivery.take()
        } else {
            None
        }
    }
}
impl<T> Drop for NativeTicketedReadback<T> {
    fn drop(&mut self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
    }
}

impl VirtualGeometryReadbackScope {
    fn try_acquire_credit(&self, bytes: u64) -> Option<VirtualGeometryReadbackCredit> {
        use std::sync::atomic::Ordering;
        let scope = self;
        if scope
            .slots
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                (value < 3).then_some(value + 1)
            })
            .is_err()
        {
            scope.budget.skipped.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        if scope
            .budget
            .live_bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value
                    .checked_add(bytes)
                    .filter(|next| *next <= VirtualGeometryReadbackBudget::LIMIT_BYTES)
            })
            .is_err()
        {
            scope.slots.fetch_sub(1, Ordering::AcqRel);
            scope.budget.skipped.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        Some(VirtualGeometryReadbackCredit {
            scope: self.clone(),
            bytes,
        })
    }
}

#[cfg(test)]
mod virtual_geometry_native_readback_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn submitted_cancelled_readback_holds_credit_until_native_map_terminal() {
        let backend = crate::graphics::backend::RenderBackend::new_offscreen().unwrap();
        let budget = VirtualGeometryReadbackBudget::new();
        let slots = Arc::new(AtomicUsize::new(0));
        let scope = budget
            .scope(
                crate::core::framework::render::RenderViewportHandle::new(7),
                1,
                1,
                Arc::clone(&slots),
            )
            .unwrap();
        let credit = scope.try_acquire_credit(16).unwrap();
        let staging = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-vg-cancel-credit-regression"),
            size: 16,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.clear_buffer(&staging, 0, None);
        let ticket = backend
            .submit_graphics_command_buffers(vec![encoder.finish()])
            .unwrap();
        let decoded = Arc::new(AtomicUsize::new(0));
        let decoded_callback = Arc::clone(&decoded);
        let packet = VirtualGeometryNativeReadbackPacket {
            scope,
            epoch: RenderPassDeviceEpoch::from_profile(backend.device_profile()),
            staging,
            range: 0..16,
            credit,
            native_probe: None,
            inspection_decoder: None,
            decoder: Box::new(move |_| {
                decoded_callback.fetch_add(1, Ordering::SeqCst);
                Ok(Default::default())
            }),
        };
        let owner = packet.bind_ticket(ticket);
        assert_eq!(budget.live_bytes.load(Ordering::Acquire), 16);
        drop(owner);
        assert_eq!(
            budget.live_bytes.load(Ordering::Acquire),
            16,
            "viewport cancellation must not release callback-owned physical credit"
        );
        assert_eq!(slots.load(Ordering::Acquire), 1);
        let next_scope = budget
            .scope(
                crate::core::framework::render::RenderViewportHandle::new(7),
                2,
                2,
                Arc::new(AtomicUsize::new(0)),
            )
            .unwrap();
        assert!(
            next_scope
                .try_acquire_credit(VirtualGeometryReadbackBudget::LIMIT_BYTES)
                .is_none(),
            "recreated viewport or new epoch cannot reset framework live bytes"
        );
        backend
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        assert_eq!(
            decoded.load(Ordering::SeqCst),
            0,
            "cancelled callback must not decode or publish"
        );
        assert_eq!(budget.live_bytes.load(Ordering::Acquire), 0);
        assert_eq!(slots.load(Ordering::Acquire), 0);
    }

    #[test]
    fn viewport_readback_cap_spans_histories_and_ready_credit() {
        let budget = VirtualGeometryReadbackBudget::new();
        let slots = Arc::new(AtomicUsize::new(0));
        let make_scope = |generation| {
            budget
                .scope(
                    crate::core::framework::render::RenderViewportHandle::new(8),
                    1,
                    generation,
                    Arc::clone(&slots),
                )
                .unwrap()
        };
        let first = make_scope(1).try_acquire_credit(16).unwrap();
        let second = make_scope(2).try_acquire_credit(16).unwrap();
        let third = make_scope(3).try_acquire_credit(16).unwrap();
        assert!(make_scope(4).try_acquire_credit(16).is_none());
        let ready = VirtualGeometryReadyReadback {
            outputs: Default::default(),
            inspection: None,
            _credit: Some(first),
            native_probe: None,
        };
        assert!(
            make_scope(5).try_acquire_credit(16).is_none(),
            "clear-only ready output still owns a slot"
        );
        drop(ready);
        assert!(make_scope(6).try_acquire_credit(16).is_some());
        drop((second, third));
        assert_eq!(slots.load(Ordering::Acquire), 0);
        assert_eq!(budget.live_bytes.load(Ordering::Acquire), 0);
    }
}

impl RenderPassGpuExecutionContext<'_> {
    /// Author topology and full identities from this frame's actual bound mesh plan.
    /// Visibility and live command counts remain GPU-produced.
    pub fn virtual_geometry_indexed_author_runs(
        &self,
    ) -> Option<&[crate::graphics::VirtualGeometryIndexedAuthorRun]> {
        self.resources
            .virtual_geometry_indexed_plan
            .as_ref()
            .map(|plan| plan.authors())
    }
}
