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

    pub(crate) fn is_compatible(
        &self,
        viewport_size: UVec2,
        pipeline: RenderPipelineHandle,
        bindings: &[FrameHistoryBinding],
    ) -> bool {
        self.viewport_size == viewport_size
            && self.pipeline == pipeline
            && self.bindings == bindings
    }

    pub(crate) fn update(
        &mut self,
        generation: u64,
        bindings: Vec<FrameHistoryBinding>,
        visibility: VisibilityHistorySnapshot,
    ) {
        self.generation = generation;
        self.bindings = bindings;
        self.visibility = visibility;
    }
}
