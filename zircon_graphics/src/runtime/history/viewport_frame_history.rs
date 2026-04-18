use zircon_framework::render::{FrameHistoryHandle, RenderPipelineHandle};
use zircon_math::UVec2;

use crate::{FrameHistoryBinding, VisibilityHistorySnapshot};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportFrameHistory {
    pub(crate) handle: FrameHistoryHandle,
    pub(crate) viewport_size: UVec2,
    pub(crate) pipeline: RenderPipelineHandle,
    pub(crate) generation: u64,
    pub(crate) bindings: Vec<FrameHistoryBinding>,
    pub(crate) visibility: VisibilityHistorySnapshot,
}
