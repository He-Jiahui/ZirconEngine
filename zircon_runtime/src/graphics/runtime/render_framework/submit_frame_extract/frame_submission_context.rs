use crate::core::framework::render::{
    RenderHybridGiExtract, RenderPipelineHandle, RenderVirtualGeometryBvhVisualizationInstance,
    RenderVirtualGeometryCpuReferenceInstance, RenderVirtualGeometryExtract,
};
use crate::core::math::UVec2;

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
    quality_profile: Option<String>,
    compiled_pipeline: CompiledRenderPipeline,
    visibility_context: VisibilityContext,
    ui_stats: UiSubmissionStats,
    hybrid_gi_enabled: bool,
    virtual_geometry_enabled: bool,
    hybrid_gi_extract: Option<RenderHybridGiExtract>,
    hybrid_gi_update_plan: Option<VisibilityHybridGiUpdatePlan>,
    hybrid_gi_feedback: Option<VisibilityHybridGiFeedback>,
    virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
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
        quality_profile: Option<String>,
        compiled_pipeline: CompiledRenderPipeline,
        visibility_context: VisibilityContext,
        ui_stats: UiSubmissionStats,
        hybrid_gi_enabled: bool,
        virtual_geometry_enabled: bool,
        hybrid_gi_extract: Option<RenderHybridGiExtract>,
        hybrid_gi_update_plan: Option<VisibilityHybridGiUpdatePlan>,
        hybrid_gi_feedback: Option<VisibilityHybridGiFeedback>,
        virtual_geometry_extract: Option<RenderVirtualGeometryExtract>,
        virtual_geometry_cpu_reference_instances: Vec<RenderVirtualGeometryCpuReferenceInstance>,
        virtual_geometry_bvh_visualization_instances: Vec<
            RenderVirtualGeometryBvhVisualizationInstance,
        >,
        virtual_geometry_page_upload_plan: Option<VisibilityVirtualGeometryPageUploadPlan>,
        virtual_geometry_feedback: Option<VisibilityVirtualGeometryFeedback>,
        predicted_generation: u64,
    ) -> Self {
        // Descriptor-disabled advanced features must not carry stale runtime payloads forward.
        let hybrid_gi_extract = hybrid_gi_enabled.then_some(hybrid_gi_extract).flatten();
        let hybrid_gi_update_plan = hybrid_gi_enabled.then_some(hybrid_gi_update_plan).flatten();
        let hybrid_gi_feedback = hybrid_gi_enabled.then_some(hybrid_gi_feedback).flatten();
        let virtual_geometry_extract = virtual_geometry_enabled
            .then_some(virtual_geometry_extract)
            .flatten();
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
            quality_profile,
            compiled_pipeline,
            visibility_context,
            ui_stats,
            hybrid_gi_enabled,
            virtual_geometry_enabled,
            hybrid_gi_extract,
            hybrid_gi_update_plan,
            hybrid_gi_feedback,
            virtual_geometry_extract,
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

    pub(super) fn quality_profile(&self) -> Option<&str> {
        self.quality_profile.as_deref()
    }

    pub(super) fn compiled_pipeline(&self) -> &CompiledRenderPipeline {
        &self.compiled_pipeline
    }

    pub(super) fn visibility_context(&self) -> &VisibilityContext {
        &self.visibility_context
    }

    pub(super) fn ui_stats(&self) -> &UiSubmissionStats {
        &self.ui_stats
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

    pub(super) fn hybrid_gi_update_plan(&self) -> Option<&VisibilityHybridGiUpdatePlan> {
        self.hybrid_gi_update_plan.as_ref()
    }

    pub(super) fn hybrid_gi_feedback(&self) -> Option<&VisibilityHybridGiFeedback> {
        self.hybrid_gi_feedback.as_ref()
    }

    pub(super) fn virtual_geometry_extract(&self) -> Option<&RenderVirtualGeometryExtract> {
        self.virtual_geometry_extract.as_ref()
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
