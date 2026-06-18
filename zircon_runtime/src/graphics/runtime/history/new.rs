use crate::core::framework::render::{FrameHistoryHandle, RenderPipelineHandle};
use crate::core::math::UVec2;

use crate::graphics::{FrameHistoryBinding, VisibilityHistorySnapshot};

use super::{FrameHistoryValidationKey, ViewportFrameHistory};

impl ViewportFrameHistory {
    pub(crate) fn new(
        handle: FrameHistoryHandle,
        target_size: UVec2,
        render_size: UVec2,
        pipeline: RenderPipelineHandle,
        generation: u64,
        bindings: Vec<FrameHistoryBinding>,
        visibility: VisibilityHistorySnapshot,
        validation_key: FrameHistoryValidationKey,
    ) -> Self {
        Self {
            handle,
            target_size,
            render_size,
            pipeline,
            generation,
            bindings,
            visibility,
            validation_key,
        }
    }
}
