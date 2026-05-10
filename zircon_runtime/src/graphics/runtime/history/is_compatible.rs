use crate::core::framework::render::RenderPipelineHandle;
use crate::core::math::UVec2;

use crate::FrameHistoryBinding;

use super::{FrameHistoryValidationKey, ViewportFrameHistory};

impl ViewportFrameHistory {
    pub(crate) fn is_compatible(
        &self,
        viewport_size: UVec2,
        pipeline: RenderPipelineHandle,
        bindings: &[FrameHistoryBinding],
        validation_key: &FrameHistoryValidationKey,
    ) -> bool {
        self.viewport_size == viewport_size
            && self.pipeline == pipeline
            && self.bindings == bindings
            && self.validation_key == *validation_key
    }
}
