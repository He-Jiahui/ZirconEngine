use crate::core::framework::render::{FrameHistoryInvalidationReason, RenderPipelineHandle};
use crate::core::math::UVec2;

use crate::graphics::FrameHistoryBinding;

use super::{FrameHistoryValidationKey, ViewportFrameHistory};

impl ViewportFrameHistory {
    pub(crate) fn is_compatible(
        &self,
        target_size: UVec2,
        render_size: UVec2,
        pipeline: RenderPipelineHandle,
        bindings: &[FrameHistoryBinding],
        validation_key: &FrameHistoryValidationKey,
    ) -> bool {
        self.incompatibility_reason(target_size, render_size, pipeline, bindings, validation_key)
            .is_none()
    }

    pub(crate) fn incompatibility_reason(
        &self,
        target_size: UVec2,
        render_size: UVec2,
        pipeline: RenderPipelineHandle,
        bindings: &[FrameHistoryBinding],
        validation_key: &FrameHistoryValidationKey,
    ) -> Option<FrameHistoryInvalidationReason> {
        if self.target_size != target_size {
            return Some(FrameHistoryInvalidationReason::ViewportResized);
        }
        if self.render_size != render_size {
            return Some(FrameHistoryInvalidationReason::RenderSizeChanged);
        }
        if self.pipeline != pipeline {
            return Some(FrameHistoryInvalidationReason::PipelineChanged);
        }
        if self.bindings != bindings {
            return Some(FrameHistoryInvalidationReason::HistoryBindingChanged);
        }
        if self.validation_key.as_ref() != validation_key {
            return Some(FrameHistoryInvalidationReason::FrameInputsChanged);
        }
        None
    }
}
