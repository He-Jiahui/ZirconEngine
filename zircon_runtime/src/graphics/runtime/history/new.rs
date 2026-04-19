use crate::core::framework::render::{FrameHistoryHandle, RenderPipelineHandle};
use crate::core::math::UVec2;

use crate::{FrameHistoryBinding, VisibilityHistorySnapshot};

use super::viewport_frame_history::ViewportFrameHistory;

impl ViewportFrameHistory {
    pub(crate) fn new(
        handle: FrameHistoryHandle,
        viewport_size: UVec2,
        pipeline: RenderPipelineHandle,
        generation: u64,
        bindings: Vec<FrameHistoryBinding>,
        visibility: VisibilityHistorySnapshot,
    ) -> Self {
        Self {
            handle,
            viewport_size,
            pipeline,
            generation,
            bindings,
            visibility,
        }
    }
}
