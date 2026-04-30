use crate::core::framework::render::CapturedFrame;

use crate::CompiledRenderPipeline;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn last_capture(
        &self,
    ) -> Option<&CapturedFrame> {
        self.last_capture.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn store_capture(
        &mut self,
        compiled_pipeline: CompiledRenderPipeline,
        capture: CapturedFrame,
    ) {
        self.compiled_pipeline = Some(compiled_pipeline);
        self.last_capture = Some(capture);
    }
}
