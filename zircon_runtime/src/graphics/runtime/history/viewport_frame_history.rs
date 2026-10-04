//! 视口历史绑定上一帧图资源租约、尺寸和相机结构键；兼容性确认后才交给下一帧。
use crate::core::framework::render::{FrameHistoryHandle, RenderPipelineHandle};
use crate::core::math::UVec2;
use std::sync::Arc;

use crate::graphics::visibility::VisibilityStaticIndex;
use crate::graphics::{FrameHistoryBinding, VisibilityHistorySnapshot};

use super::FrameHistoryValidationKey;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportFrameHistory {
    pub(super) handle: FrameHistoryHandle,
    pub(super) target_size: UVec2,
    pub(super) render_size: UVec2,
    pub(super) pipeline: RenderPipelineHandle,
    pub(super) generation: u64,
    pub(super) bindings: Vec<FrameHistoryBinding>,
    pub(super) visibility: VisibilityHistorySnapshot,
    pub(super) static_index: VisibilityStaticIndex,
    pub(super) dynamic_index: VisibilityStaticIndex,
    pub(super) validation_key: Arc<FrameHistoryValidationKey>,
}

#[cfg(test)]
#[path = "tests/viewport_frame_history.rs"]
mod tests;
