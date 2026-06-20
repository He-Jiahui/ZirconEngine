use crate::core::framework::render::{FrameHistoryHandle, RenderPipelineHandle};
use crate::core::math::UVec2;

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
    pub(super) validation_key: FrameHistoryValidationKey,
}
