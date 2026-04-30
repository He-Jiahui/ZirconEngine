use crate::core::framework::render::{FrameHistoryHandle, RenderPipelineHandle};
use crate::core::math::UVec2;

use crate::{FrameHistoryBinding, VisibilityHistorySnapshot};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportFrameHistory {
    pub(super) handle: FrameHistoryHandle,
    pub(super) viewport_size: UVec2,
    pub(super) pipeline: RenderPipelineHandle,
    pub(super) generation: u64,
    pub(super) bindings: Vec<FrameHistoryBinding>,
    pub(super) visibility: VisibilityHistorySnapshot,
}
