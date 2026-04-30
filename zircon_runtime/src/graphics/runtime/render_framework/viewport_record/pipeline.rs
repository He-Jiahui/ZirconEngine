use crate::core::framework::render::RenderPipelineHandle;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn pipeline(
        &self,
    ) -> Option<RenderPipelineHandle> {
        self.pipeline
    }

    pub(in crate::graphics::runtime::render_framework) fn effective_pipeline(
        &self,
        default_pipeline: RenderPipelineHandle,
    ) -> RenderPipelineHandle {
        self.pipeline
            .or_else(|| {
                self.quality_profile
                    .as_ref()
                    .and_then(|profile| profile.pipeline_override)
            })
            .unwrap_or(default_pipeline)
    }

    pub(in crate::graphics::runtime::render_framework) fn set_pipeline(
        &mut self,
        pipeline: RenderPipelineHandle,
    ) {
        self.pipeline = Some(pipeline);
    }
}
