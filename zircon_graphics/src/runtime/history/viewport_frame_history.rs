use zircon_math::UVec2;
use zircon_render_server::{FrameHistoryHandle, RenderPipelineHandle};

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
