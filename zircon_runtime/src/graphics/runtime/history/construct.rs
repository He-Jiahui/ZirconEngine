use crate::core::framework::render::{FrameHistoryHandle, RenderPipelineHandle};
use crate::core::math::UVec2;
use std::sync::Arc;

use crate::graphics::visibility::VisibilityStaticIndex;
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
        static_index: VisibilityStaticIndex,
        validation_key: Arc<FrameHistoryValidationKey>,
    ) -> Self {
        Self {
            handle,
            target_size,
            render_size,
            pipeline,
            generation,
            bindings,
            visibility,
            static_index,
            validation_key,
        }
    }
}
