use crate::core::framework::render::{
    AdvancedProfileRuntimePlan, AdvancedProviderReport, AntiAliasFallbackReport,
    PostProcessPassGraph, PostProcessStackDescriptor, RenderAmbientLightSnapshot,
    RenderBloomSettings, RenderColorGradingSettings, RenderDirectionalLightSnapshot,
    RenderHybridGiExtract, RenderHybridGiPayloadSource, RenderMeshSnapshot, RenderPipelineHandle,
    RenderPointLightSnapshot, RenderRectLightSnapshot, RenderSpotLightSnapshot,
    RenderVirtualGeometryBvhVisualizationInstance, RenderVirtualGeometryCpuReferenceInstance,
    RenderVirtualGeometryExtract, RenderVirtualGeometryPayloadSource, SolariRuntimeReport,
};
use crate::core::math::UVec2;
use crate::graphics::runtime::FrameHistoryValidationKey;

use crate::{
    CompiledRenderPipeline, VisibilityContext, VisibilityHybridGiFeedback,
    VisibilityHybridGiUpdatePlan, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct UiSubmissionStats {
    command_count: usize,
    quad_count: usize,
    text_payload_count: usize,
    image_payload_count: usize,
    clipped_command_count: usize,
}

pub(super) struct FrameSubmissionContext {
    size: UVec2,
    pipeline_handle: RenderPipelineHandle,
    viewport_generation: u64,
    quality_profile: Option<String>,
    compiled_pipeline: CompiledRenderPipeline,
    visibility_context: VisibilityContext,
    history_validation_key: FrameHistoryValidationKey,
    ui_stats: UiSubmissionStats,
    post_process_bloom: RenderBloomSettings,
    post_process_color_grading: RenderColorGradingSettings,
    anti_alias_fallback: AntiAliasFallbackReport,
    advanced_runtime_plan: AdvancedProfileRuntimePlan,
    solari_runtime_report: SolariRuntimeReport,
    post_process_stack: PostProcessStackDescriptor,
    post_process_graph: PostProcessPassGraph,
    hybrid_gi_enabled: bool,
    virtual_geometry_enabled: bool,
    hybrid_gi_extract: Option<RenderHybridGiExtract>,
    hybrid_gi_payload_source: RenderHybridGiPayloadSource,
    hybrid_gi_update_plan: Option<VisibilityHybridGiUpdatePlan>,
    hybrid_gi_feedback: Option<VisibilityHybridGiFeedback>,
    scene_meshes: Vec<RenderMeshSnapshot>,
    scene_directional_lights: Vec<RenderDirectionalLightSnapshot>,
    scene_point_lights: Vec<RenderPointLightSnapshot>,
    scene_spot_lights: Vec<RenderSpotLightSnapshot>,
    scene_ambient_lights: Vec<RenderAmbientLightSnapshot>,
    scene_rect_lights: Vec<RenderRectLightSnapshot>,
    virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
    virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
    virtual_geometry_cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
    virtual_geometry_bvh_visualization_instances:
        Vec<RenderVirtualGeometryBvhVisualizationInstance>,
    virtual_geometry_page_upload_plan: Option<VisibilityVirtualGeometryPageUploadPlan>,
    virtual_geometry_feedback: Option<VisibilityVirtualGeometryFeedback>,
    predicted_generation: u64,
}

impl FrameSubmissionContext {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        size: UVec2,
        pipeline_handle: RenderPipelineHandle,
        viewport_generation: u64,
        quality_profile: Option<String>,
        compiled_pipeline: CompiledRenderPipeline,
        visibility_context: VisibilityContext,
        history_validation_key: FrameHistoryValidationKey,
        ui_stats: UiSubmissionStats,
        post_process_bloom: RenderBloomSettings,
        post_process_color_grading: RenderColorGradingSettings,
        anti_alias_fallback: AntiAliasFallbackReport,
        advanced_runtime_plan: AdvancedProfileRuntimePlan,
        solari_runtime_report: SolariRuntimeReport,
        post_process_stack: PostProcessStackDescriptor,
        post_process_graph: PostProcessPassGraph,
        hybrid_gi_enabled: bool,
        virtual_geometry_enabled: bool,
        hybrid_gi_extract: Option<RenderHybridGiExtract>,
        hybrid_gi_payload_source: RenderHybridGiPayloadSource,
        hybrid_gi_update_plan: Option<VisibilityHybridGiUpdatePlan>,
        hybrid_gi_feedback: Option<VisibilityHybridGiFeedback>,
        scene_meshes: Vec<RenderMeshSnapshot>,
        scene_directional_lights: Vec<RenderDirectionalLightSnapshot>,
        scene_point_lights: Vec<RenderPointLightSnapshot>,
        scene_spot_lights: Vec<RenderSpotLightSnapshot>,
        scene_ambient_lights: Vec<RenderAmbientLightSnapshot>,
        scene_rect_lights: Vec<RenderRectLightSnapshot>,
        virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
        virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
        virtual_geometry_cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
        virtual_geometry_bvh_visualization_instances: Vec<
            RenderVirtualGeometryBvhVisualizationInstance,
        >,
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
        let virtual_geometry_page_upload_plan = virtual_geometry_enabled
            .then_some(virtual_geometry_page_upload_plan)
            .flatten();
        let virtual_geometry_feedback = virtual_geometry_enabled
            .then_some(virtual_geometry_feedback)
            .flatten();

        Self {
            size,
            pipeline_handle,
            viewport_generation,
            quality_profile,
            compiled_pipeline,
            visibility_context,
            history_validation_key,
            ui_stats,
            post_process_bloom,
            post_process_color_grading,
            anti_alias_fallback,
            advanced_runtime_plan,
            solari_runtime_report,
            post_process_stack,
            post_process_graph,
            hybrid_gi_enabled,
            virtual_geometry_enabled,
            hybrid_gi_extract,
            hybrid_gi_payload_source,
            hybrid_gi_update_plan,
            hybrid_gi_feedback,
            scene_meshes,
            scene_directional_lights,
            scene_point_lights,
            scene_spot_lights,
            scene_ambient_lights,
            scene_rect_lights,
            virtual_geometry_extract,
            virtual_geometry_payload_source,
            virtual_geometry_cpu_reference_instances,
            virtual_geometry_bvh_visualization_instances,
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            predicted_generation,
        }
    }

    pub(super) fn size(&self) -> UVec2 {
        self.size
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

    pub(super) fn compiled_pipeline(&self) -> &CompiledRenderPipeline {
        &self.compiled_pipeline
    }

    pub(super) fn visibility_context(&self) -> &VisibilityContext {
        &self.visibility_context
    }

    pub(super) fn history_validation_key(&self) -> &FrameHistoryValidationKey {
        &self.history_validation_key
    }

    pub(super) fn ui_stats(&self) -> &UiSubmissionStats {
        &self.ui_stats
    }

    pub(super) fn post_process_bloom(&self) -> RenderBloomSettings {
        self.post_process_bloom
    }

    pub(super) fn post_process_color_grading(&self) -> RenderColorGradingSettings {
        self.post_process_color_grading
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

    pub(super) fn post_process_stack(&self) -> &PostProcessStackDescriptor {
        &self.post_process_stack
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
        &self.scene_meshes
    }

    pub(super) fn scene_directional_lights(&self) -> &[RenderDirectionalLightSnapshot] {
        &self.scene_directional_lights
    }

    pub(super) fn scene_point_lights(&self) -> &[RenderPointLightSnapshot] {
        &self.scene_point_lights
    }

    pub(super) fn scene_spot_lights(&self) -> &[RenderSpotLightSnapshot] {
        &self.scene_spot_lights
    }

    pub(super) fn scene_ambient_lights(&self) -> &[RenderAmbientLightSnapshot] {
        &self.scene_ambient_lights
    }

    pub(super) fn scene_rect_lights(&self) -> &[RenderRectLightSnapshot] {
        &self.scene_rect_lights
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

    pub(super) fn predicted_generation(&self) -> u64 {
        self.predicted_generation
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
mod tests {
    use crate::core::framework::render::{
        AdvancedProviderAvailability, AdvancedRenderFeature, RenderCapabilitySummary,
        RenderFrameExtract, RenderProfileBundle, RenderWorldSnapshotHandle,
    };
    use crate::core::math::UVec2;
    use crate::graphics::{CompiledRenderPipeline, RenderPassStage};
    use crate::render_graph::RenderGraphBuilder;
    use crate::scene::world::World;

    use super::*;

    #[test]
    fn advanced_runtime_plan_gates_provider_missing_feature_payloads() {
        let context = context_with_advanced_plan(AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new().with_hybrid_gi_provider("hgi"),
        ));

        assert!(context.hybrid_gi_enabled());
        assert!(!context.virtual_geometry_enabled());

        let virtual_geometry = context
            .advanced_provider_reports()
            .iter()
            .find(|report| report.feature == AdvancedRenderFeature::VirtualGeometry)
            .expect("virtual geometry report");
        assert_eq!(
            virtual_geometry.degradation_reason_labels(),
            vec!["provider-missing"]
        );
    }

    #[test]
    fn advanced_runtime_plan_keeps_provider_backed_features_enabled() {
        let context = context_with_advanced_plan(AdvancedProfileRuntimePlan::from_profile_bundle(
            &RenderProfileBundle::advanced_render(),
            &advanced_capabilities(),
            &AdvancedProviderAvailability::new()
                .with_virtual_geometry_provider("vg")
                .with_hybrid_gi_provider("hgi"),
        ));

        assert!(context.hybrid_gi_enabled());
        assert!(context.virtual_geometry_enabled());
        assert!(context
            .advanced_provider_reports()
            .iter()
            .all(|report| report.degradations.is_empty()));
    }

    #[test]
    fn virtual_geometry_payload_source_clears_when_plan_degrades_feature() {
        let context = context_with_advanced_plan_and_virtual_geometry(
            AdvancedProfileRuntimePlan::from_profile_bundle(
                &RenderProfileBundle::advanced_render(),
                &advanced_capabilities(),
                &AdvancedProviderAvailability::new().with_hybrid_gi_provider("hgi"),
            ),
            Some(RenderVirtualGeometryExtract::default()),
            RenderVirtualGeometryPayloadSource::Authored,
        );

        assert!(!context.virtual_geometry_enabled());
        assert!(context.virtual_geometry_extract().is_none());
        assert_eq!(
            context.virtual_geometry_payload_source(),
            RenderVirtualGeometryPayloadSource::None
        );
    }

    #[test]
    fn virtual_geometry_payload_source_survives_for_provider_backed_extract() {
        let context = context_with_advanced_plan_and_virtual_geometry(
            AdvancedProfileRuntimePlan::from_profile_bundle(
                &RenderProfileBundle::advanced_render(),
                &advanced_capabilities(),
                &AdvancedProviderAvailability::new()
                    .with_virtual_geometry_provider("vg")
                    .with_hybrid_gi_provider("hgi"),
            ),
            Some(RenderVirtualGeometryExtract::default()),
            RenderVirtualGeometryPayloadSource::Authored,
        );

        assert!(context.virtual_geometry_enabled());
        assert!(context.virtual_geometry_extract().is_some());
        assert_eq!(
            context.virtual_geometry_payload_source(),
            RenderVirtualGeometryPayloadSource::Authored
        );
    }

    #[test]
    fn hybrid_gi_payload_source_clears_when_plan_degrades_feature() {
        let context = context_with_advanced_plan_and_payloads(
            AdvancedProfileRuntimePlan::from_profile_bundle(
                &RenderProfileBundle::advanced_render(),
                &advanced_capabilities(),
                &AdvancedProviderAvailability::new().with_virtual_geometry_provider("vg"),
            ),
            Some(RenderHybridGiExtract::default()),
            RenderHybridGiPayloadSource::Authored,
            None,
            RenderVirtualGeometryPayloadSource::None,
        );

        assert!(!context.hybrid_gi_enabled());
        assert!(context.hybrid_gi_extract().is_none());
        assert_eq!(
            context.hybrid_gi_payload_source(),
            RenderHybridGiPayloadSource::None
        );
    }

    #[test]
    fn hybrid_gi_payload_source_survives_for_provider_backed_extract() {
        let context = context_with_advanced_plan_and_payloads(
            AdvancedProfileRuntimePlan::from_profile_bundle(
                &RenderProfileBundle::advanced_render(),
                &advanced_capabilities(),
                &AdvancedProviderAvailability::new()
                    .with_virtual_geometry_provider("vg")
                    .with_hybrid_gi_provider("hgi"),
            ),
            Some(RenderHybridGiExtract::default()),
            RenderHybridGiPayloadSource::Authored,
            None,
            RenderVirtualGeometryPayloadSource::None,
        );

        assert!(context.hybrid_gi_enabled());
        assert!(context.hybrid_gi_extract().is_some());
        assert_eq!(
            context.hybrid_gi_payload_source(),
            RenderHybridGiPayloadSource::Authored
        );
    }

    fn context_with_advanced_plan(
        advanced_runtime_plan: AdvancedProfileRuntimePlan,
    ) -> FrameSubmissionContext {
        context_with_advanced_plan_and_virtual_geometry(
            advanced_runtime_plan,
            None,
            RenderVirtualGeometryPayloadSource::None,
        )
    }

    fn context_with_advanced_plan_and_virtual_geometry(
        advanced_runtime_plan: AdvancedProfileRuntimePlan,
        virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
        virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
    ) -> FrameSubmissionContext {
        context_with_advanced_plan_and_payloads(
            advanced_runtime_plan,
            None,
            RenderHybridGiPayloadSource::None,
            virtual_geometry_extract,
            virtual_geometry_payload_source,
        )
    }

    fn context_with_advanced_plan_and_payloads(
        advanced_runtime_plan: AdvancedProfileRuntimePlan,
        hybrid_gi_extract: Option<RenderHybridGiExtract>,
        hybrid_gi_payload_source: RenderHybridGiPayloadSource,
        virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
        virtual_geometry_payload_source: RenderVirtualGeometryPayloadSource,
    ) -> FrameSubmissionContext {
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        );
        FrameSubmissionContext::new(
            UVec2::new(64, 64),
            RenderPipelineHandle::new(1),
            0,
            None,
            empty_pipeline(),
            VisibilityContext::from_extract(&extract),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            advanced_runtime_plan,
            Default::default(),
            Default::default(),
            Default::default(),
            true,
            true,
            hybrid_gi_extract,
            hybrid_gi_payload_source,
            None,
            None,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            virtual_geometry_extract,
            virtual_geometry_payload_source,
            Vec::new(),
            Vec::new(),
            None,
            None,
            1,
        )
    }

    fn advanced_capabilities() -> RenderCapabilitySummary {
        RenderCapabilitySummary {
            virtual_geometry_supported: true,
            hybrid_global_illumination_supported: true,
            supports_storage_buffers: true,
            supports_indirect_draw: true,
            supports_buffer_readback: true,
            ..RenderCapabilitySummary::default()
        }
    }

    fn empty_pipeline() -> CompiledRenderPipeline {
        let graph = RenderGraphBuilder::new("advanced-runtime-plan-context-test")
            .compile()
            .unwrap();
        CompiledRenderPipeline {
            handle: RenderPipelineHandle::new(1),
            name: "empty".to_string(),
            renderer_name: "empty".to_string(),
            stages: vec![RenderPassStage::Opaque3d],
            pass_stages: Vec::new(),
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph,
        }
    }
}
