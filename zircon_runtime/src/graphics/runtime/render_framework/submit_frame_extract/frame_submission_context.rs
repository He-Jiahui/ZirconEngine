use std::sync::Arc;

use crate::core::framework::render::{
    AdvancedProfileRuntimePlan, AdvancedProviderReport, AntiAliasFallbackReport, AntiAliasMode,
    FrameHistoryInvalidationReason, PostProcessPassGraph, RenderAmbientLightSnapshot,
    RenderCameraOrderReport, RenderCameraTargetResolutionReport, RenderCapabilitySummary,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderHybridGiExtract,
    RenderHybridGiPayloadSource, RenderMeshSnapshot, RenderPipelineHandle,
    RenderPointLightSnapshot, RenderPostProcessEffectStackSettings, RenderRectLightSnapshot,
    RenderSpotLightSnapshot, RenderVirtualGeometryBvhVisualizationInstance,
    RenderVirtualGeometryCpuReferenceInstance, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPagePayload, RenderVirtualGeometryPayloadSource, ShaderQualityTier,
    SolariRuntimeReport, TemporalJitterSample, TemporalJitterSequence, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::FrameHistoryValidationKey;
use crate::graphics::{ViewVisibilityContext, ViewportRenderOutputTarget, VisibilityViewKey};

use crate::graphics::{
    CompiledRenderPipeline, VisibilityContext, VisibilityHybridGiFeedback,
    VisibilityHybridGiUpdatePlan, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};

use super::super::viewport_record::ViewportCameraHistoryKey;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct UiSubmissionStats {
    command_count: usize,
    quad_count: usize,
    text_payload_count: usize,
    image_payload_count: usize,
    clipped_command_count: usize,
}

pub(super) struct FrameSubmissionContext {
    // Presentation/imported target extent. Internal graph resources may use a smaller render size.
    target_size: UVec2,
    render_size: UVec2,
    pipeline_handle: RenderPipelineHandle,
    viewport_generation: u64,
    quality_profile: Option<String>,
    shader_quality: ShaderQualityTier,
    global_material_mip_bias: f32,
    compiled_pipeline: Arc<CompiledRenderPipeline>,
    capabilities: RenderCapabilitySummary,
    visibility_context: VisibilityContext,
    previous_motion_vector_camera: Option<ViewportCameraSnapshot>,
    camera_history_key: ViewportCameraHistoryKey,
    history_validation_key: Arc<FrameHistoryValidationKey>,
    history_invalidation_reason: Option<FrameHistoryInvalidationReason>,
    output_target: ViewportRenderOutputTarget,
    camera_target_resolution: RenderCameraTargetResolutionReport,
    scene_camera_order_report: Option<RenderCameraOrderReport>,
    ui_stats: UiSubmissionStats,
    post_process_effect_stack: RenderPostProcessEffectStackSettings,
    anti_alias_fallback: AntiAliasFallbackReport,
    advanced_runtime_plan: AdvancedProfileRuntimePlan,
    solari_runtime_report: SolariRuntimeReport,
    post_process_graph: PostProcessPassGraph,
    hybrid_gi_enabled: bool,
    virtual_geometry_enabled: bool,
    hybrid_gi_extract: Option<RenderHybridGiExtract>,
    hybrid_gi_payload_source: RenderHybridGiPayloadSource,
    hybrid_gi_update_plan: Option<VisibilityHybridGiUpdatePlan>,
    hybrid_gi_feedback: Option<VisibilityHybridGiFeedback>,
    // Owns the viewport-sized extract once so submit/prepare/stat readers can borrow heavy scene
    // payload slices without cloning meshes, lights, or particle previous-state vectors.
    source_extract: Arc<RenderFrameExtract>,
    particle_sprite_count: usize,
    particle_previous_state_sprite_count: usize,
    particle_anonymous_stream_ambiguity_sprite_count: usize,
    virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
    virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
    virtual_geometry_cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    virtual_geometry_bvh_visualization_instances:
        Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    virtual_geometry_resident_page_payloads: Vec<RenderVirtualGeometryPagePayload>,
    virtual_geometry_page_upload_plan: Option<VisibilityVirtualGeometryPageUploadPlan>,
    virtual_geometry_feedback: Option<VisibilityVirtualGeometryFeedback>,
    predicted_generation: u64,
}

impl FrameSubmissionContext {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        target_size: UVec2,
        render_size: UVec2,
        pipeline_handle: RenderPipelineHandle,
        viewport_generation: u64,
        quality_profile: Option<String>,
        shader_quality: ShaderQualityTier,
        compiled_pipeline: Arc<CompiledRenderPipeline>,
        capabilities: RenderCapabilitySummary,
        visibility_context: VisibilityContext,
        previous_motion_vector_camera: Option<ViewportCameraSnapshot>,
        camera_history_key: ViewportCameraHistoryKey,
        history_validation_key: FrameHistoryValidationKey,
        history_invalidation_reason: Option<FrameHistoryInvalidationReason>,
        output_target: ViewportRenderOutputTarget,
        camera_target_resolution: RenderCameraTargetResolutionReport,
        scene_camera_order_report: Option<RenderCameraOrderReport>,
        ui_stats: UiSubmissionStats,
        post_process_effect_stack: RenderPostProcessEffectStackSettings,
        anti_alias_fallback: AntiAliasFallbackReport,
        advanced_runtime_plan: AdvancedProfileRuntimePlan,
        solari_runtime_report: SolariRuntimeReport,
        post_process_graph: PostProcessPassGraph,
        hybrid_gi_enabled: bool,
        virtual_geometry_enabled: bool,
        hybrid_gi_extract: Option<RenderHybridGiExtract>,
        hybrid_gi_payload_source: RenderHybridGiPayloadSource,
        hybrid_gi_update_plan: Option<VisibilityHybridGiUpdatePlan>,
        hybrid_gi_feedback: Option<VisibilityHybridGiFeedback>,
        source_extract: Arc<RenderFrameExtract>,
        particle_sprite_count: usize,
        particle_previous_state_sprite_count: usize,
        particle_anonymous_stream_ambiguity_sprite_count: usize,
        virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
        virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
        virtual_geometry_cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
        virtual_geometry_bvh_visualization_instances: Vec<
            RenderVirtualGeometryBvhVisualizationInstance,
        >,
        virtual_geometry_resident_page_payloads: Vec<RenderVirtualGeometryPagePayload>,
        virtual_geometry_page_upload_plan: Option<VisibilityVirtualGeometryPageUploadPlan>,
        virtual_geometry_feedback: Option<VisibilityVirtualGeometryFeedback>,
        predicted_generation: u64,
    ) -> Self {
        // Degraded or descriptor-disabled advanced features must not carry stale runtime payloads forward.
        let hybrid_gi_enabled =
            hybrid_gi_enabled && advanced_runtime_plan.hybrid_global_illumination_enabled();
        let virtual_geometry_enabled =
            virtual_geometry_enabled && advanced_runtime_plan.virtual_geometry_enabled();
        let hybrid_gi_extract = hybrid_gi_enabled.then_some(hybrid_gi_extract).flatten();
        let hybrid_gi_payload_source = if hybrid_gi_enabled && hybrid_gi_extract.is_some() {
            hybrid_gi_payload_source
        } else {
            RenderHybridGiPayloadSource::None
        };
        let hybrid_gi_update_plan = hybrid_gi_enabled.then_some(hybrid_gi_update_plan).flatten();
        let hybrid_gi_feedback = hybrid_gi_enabled.then_some(hybrid_gi_feedback).flatten();
        let virtual_geometry_extract = virtual_geometry_enabled
            .then_some(virtual_geometry_extract)
            .flatten();
        let virtual_geometry_payload_source =
            if virtual_geometry_enabled && virtual_geometry_extract.is_some() {
                virtual_geometry_payload_source
            } else {
                RenderVirtualGeometryPayloadSource::None
            };
        let virtual_geometry_cpu_reference_instances = virtual_geometry_enabled
            .then_some(virtual_geometry_cpu_reference_instances)
            .unwrap_or_default();
        let virtual_geometry_bvh_visualization_instances = virtual_geometry_enabled
            .then_some(virtual_geometry_bvh_visualization_instances)
            .unwrap_or_default();
        let virtual_geometry_resident_page_payloads = virtual_geometry_enabled
            .then_some(virtual_geometry_resident_page_payloads)
            .unwrap_or_default();
        let virtual_geometry_page_upload_plan = virtual_geometry_enabled
            .then_some(virtual_geometry_page_upload_plan)
            .flatten();
        let virtual_geometry_feedback = virtual_geometry_enabled
            .then_some(virtual_geometry_feedback)
            .flatten();

        Self {
            target_size,
            render_size,
            pipeline_handle,
            viewport_generation,
            quality_profile,
            shader_quality,
            global_material_mip_bias: 0.0,
            compiled_pipeline,
            capabilities,
            visibility_context,
            previous_motion_vector_camera,
            camera_history_key,
            history_validation_key: Arc::new(history_validation_key),
            history_invalidation_reason,
            output_target,
            camera_target_resolution,
            scene_camera_order_report,
            ui_stats,
            post_process_effect_stack,
            anti_alias_fallback,
            advanced_runtime_plan,
            solari_runtime_report,
            post_process_graph,
            hybrid_gi_enabled,
            virtual_geometry_enabled,
            hybrid_gi_extract,
            hybrid_gi_payload_source,
            hybrid_gi_update_plan,
            hybrid_gi_feedback,
            source_extract,
            particle_sprite_count,
            particle_previous_state_sprite_count,
            particle_anonymous_stream_ambiguity_sprite_count,
            virtual_geometry_extract,
            virtual_geometry_payload_source,
            virtual_geometry_cpu_reference_instances,
            virtual_geometry_bvh_visualization_instances,
            virtual_geometry_resident_page_payloads,
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            predicted_generation,
        }
    }

    pub(super) fn size(&self) -> UVec2 {
        self.target_size
    }

    pub(super) fn render_size(&self) -> UVec2 {
        self.render_size
    }

    pub(super) fn pipeline_handle(&self) -> RenderPipelineHandle {
        self.pipeline_handle
    }

    pub(super) fn viewport_generation(&self) -> u64 {
        self.viewport_generation
    }

    pub(super) fn quality_profile(&self) -> Option<&str> {
        self.quality_profile.as_deref()
    }

    pub(super) fn shader_quality(&self) -> ShaderQualityTier {
        self.shader_quality
    }

    pub(super) fn with_global_material_mip_bias(mut self, mip_bias: f32) -> Self {
        self.global_material_mip_bias = mip_bias;
        self
    }

    pub(super) fn global_material_mip_bias(&self) -> f32 {
        self.global_material_mip_bias
    }

    pub(super) fn compiled_pipeline(&self) -> &CompiledRenderPipeline {
        &self.compiled_pipeline
    }

    pub(super) fn compiled_pipeline_shared(&self) -> Arc<CompiledRenderPipeline> {
        Arc::clone(&self.compiled_pipeline)
    }

    pub(super) fn capabilities(&self) -> &RenderCapabilitySummary {
        &self.capabilities
    }

    pub(super) fn visibility_context(&self) -> &VisibilityContext {
        &self.visibility_context
    }

    pub(super) fn source_extract(&self) -> Arc<RenderFrameExtract> {
        Arc::clone(&self.source_extract)
    }

    pub(super) fn source_world(&self) -> crate::core::framework::render::RenderWorldSnapshotHandle {
        self.source_extract.world
    }

    pub(super) fn view_visibility(
        &self,
        key: &VisibilityViewKey,
    ) -> Option<&ViewVisibilityContext> {
        self.visibility_context.frame_visibility.view(key)
    }

    pub(super) fn previous_motion_vector_camera(&self) -> Option<&ViewportCameraSnapshot> {
        self.previous_motion_vector_camera.as_ref()
    }

    pub(super) fn camera_history_key(&self) -> &ViewportCameraHistoryKey {
        &self.camera_history_key
    }

    pub(super) fn history_validation_key_shared(&self) -> Arc<FrameHistoryValidationKey> {
        Arc::clone(&self.history_validation_key)
    }

    pub(super) fn history_invalidation_reason(&self) -> Option<FrameHistoryInvalidationReason> {
        self.history_invalidation_reason
    }

    pub(super) fn output_target(&self) -> ViewportRenderOutputTarget {
        self.output_target
    }

    pub(super) fn camera_target_resolution(&self) -> RenderCameraTargetResolutionReport {
        self.camera_target_resolution
    }

    pub(super) fn scene_camera_order_report(&self) -> Option<&RenderCameraOrderReport> {
        self.scene_camera_order_report.as_ref()
    }

    pub(super) fn ui_stats(&self) -> &UiSubmissionStats {
        &self.ui_stats
    }

    pub(super) fn post_process_effect_stack(&self) -> RenderPostProcessEffectStackSettings {
        self.post_process_effect_stack
    }

    pub(super) fn anti_alias_fallback(&self) -> AntiAliasFallbackReport {
        self.anti_alias_fallback
    }

    pub(super) fn advanced_provider_reports(&self) -> &[AdvancedProviderReport] {
        &self.advanced_runtime_plan.reports
    }

    pub(super) fn solari_runtime_report(&self) -> &SolariRuntimeReport {
        &self.solari_runtime_report
    }

    pub(super) fn post_process_graph(&self) -> &PostProcessPassGraph {
        &self.post_process_graph
    }

    pub(super) fn hybrid_gi_enabled(&self) -> bool {
        self.hybrid_gi_enabled
    }

    pub(super) fn virtual_geometry_enabled(&self) -> bool {
        self.virtual_geometry_enabled
    }

    pub(super) fn hybrid_gi_extract(&self) -> Option<&RenderHybridGiExtract> {
        self.hybrid_gi_extract.as_ref()
    }

    pub(super) fn hybrid_gi_payload_source(&self) -> RenderHybridGiPayloadSource {
        self.hybrid_gi_payload_source
    }

    pub(super) fn hybrid_gi_update_plan(&self) -> Option<&VisibilityHybridGiUpdatePlan> {
        self.hybrid_gi_update_plan.as_ref()
    }

    pub(super) fn hybrid_gi_feedback(&self) -> Option<&VisibilityHybridGiFeedback> {
        self.hybrid_gi_feedback.as_ref()
    }

    pub(super) fn scene_meshes(&self) -> &[RenderMeshSnapshot] {
        &self.source_extract.geometry.meshes
    }

    pub(super) fn scene_directional_lights(&self) -> &[RenderDirectionalLightSnapshot] {
        &self.source_extract.lighting.directional_lights
    }

    pub(super) fn scene_point_lights(&self) -> &[RenderPointLightSnapshot] {
        &self.source_extract.lighting.point_lights
    }

    pub(super) fn scene_spot_lights(&self) -> &[RenderSpotLightSnapshot] {
        &self.source_extract.lighting.spot_lights
    }

    pub(super) fn scene_baked_lighting(
        &self,
    ) -> Option<&crate::core::framework::render::LightmapConsumeContract> {
        self.source_extract.environment.baked_lighting()
    }

    pub(super) fn scene_has_baked_probe_grid(&self) -> bool {
        self.source_extract.environment.light_probe_grid().is_some()
    }

    pub(super) fn scene_ambient_lights(&self) -> &[RenderAmbientLightSnapshot] {
        &self.source_extract.lighting.ambient_lights
    }

    pub(super) fn scene_rect_lights(&self) -> &[RenderRectLightSnapshot] {
        &self.source_extract.lighting.rect_lights
    }

    pub(super) fn particle_sprite_count(&self) -> usize {
        self.particle_sprite_count
    }

    pub(super) fn particle_previous_state_sprite_count(&self) -> usize {
        self.particle_previous_state_sprite_count
    }

    pub(super) fn particle_anonymous_stream_ambiguity_sprite_count(&self) -> usize {
        self.particle_anonymous_stream_ambiguity_sprite_count
    }

    pub(super) fn virtual_geometry_extract(&self) -> Option<&RenderVirtualGeometryExtract> {
        self.virtual_geometry_extract.as_ref()
    }

    pub(super) fn virtual_geometry_payload_source(&self) -> RenderVirtualGeometryPayloadSource {
        self.virtual_geometry_payload_source
    }

    pub(super) fn virtual_geometry_page_upload_plan(
        &self,
    ) -> Option<&VisibilityVirtualGeometryPageUploadPlan> {
        self.virtual_geometry_page_upload_plan.as_ref()
    }

    pub(super) fn virtual_geometry_feedback(&self) -> Option<&VisibilityVirtualGeometryFeedback> {
        self.virtual_geometry_feedback.as_ref()
    }

    pub(super) fn virtual_geometry_cpu_reference_instances(
        &self,
    ) -> &[RenderVirtualGeometryCpuReferenceInstance] {
        &self.virtual_geometry_cpu_reference_instances
    }

    pub(super) fn virtual_geometry_bvh_visualization_instances(
        &self,
    ) -> &[RenderVirtualGeometryBvhVisualizationInstance] {
        &self.virtual_geometry_bvh_visualization_instances
    }

    pub(super) fn virtual_geometry_resident_page_payloads(
        &self,
    ) -> &[RenderVirtualGeometryPagePayload] {
        &self.virtual_geometry_resident_page_payloads
    }

    pub(super) fn predicted_generation(&self) -> u64 {
        self.predicted_generation
    }
}

const DEFAULT_TAA_JITTER_PERIOD: u32 = 8;

pub(super) fn temporal_jitter_for_submission(
    anti_alias_fallback: AntiAliasFallbackReport,
    temporal_frame_index: u64,
) -> TemporalJitterSample {
    if anti_alias_fallback.effective_mode == AntiAliasMode::Taa {
        TemporalJitterSequence::new(DEFAULT_TAA_JITTER_PERIOD).sample(temporal_frame_index)
    } else {
        TemporalJitterSample::default()
    }
}

impl UiSubmissionStats {
    pub(super) fn record_command(&mut self) {
        self.command_count += 1;
    }

    pub(super) fn record_quad(&mut self) {
        self.quad_count += 1;
    }

    pub(super) fn record_text_payload(&mut self) {
        self.text_payload_count += 1;
    }

    pub(super) fn record_image_payload(&mut self) {
        self.image_payload_count += 1;
    }

    pub(super) fn record_clipped_command(&mut self) {
        self.clipped_command_count += 1;
    }

    pub(super) fn command_count(&self) -> usize {
        self.command_count
    }

    pub(super) fn quad_count(&self) -> usize {
        self.quad_count
    }

    pub(super) fn text_payload_count(&self) -> usize {
        self.text_payload_count
    }

    pub(super) fn image_payload_count(&self) -> usize {
        self.image_payload_count
    }

    pub(super) fn clipped_command_count(&self) -> usize {
        self.clipped_command_count
    }
}

#[cfg(test)]
mod tests;
