use crate::core::framework::render::{RenderHybridGiPreparedFrame, RenderPluginRendererOutputs};

#[derive(Clone, Debug, Default, PartialEq)]
/// provider 准备阶段移交给渲染提交的侧带；可淘汰探针候选、renderer 输出和已准备帧在此保持同一帧归属。
/// 框架通过 `into_parts` 一次消费，随后与虚拟几何侧带合并。
pub struct HybridGiRuntimePrepareOutput {
    evictable_probe_ids: Vec<u32>,
    renderer_outputs: RenderPluginRendererOutputs,
    prepared_frame: Option<RenderHybridGiPreparedFrame>,
}

impl HybridGiRuntimePrepareOutput {
    pub fn new(evictable_probe_ids: Vec<u32>) -> Self {
        Self {
            evictable_probe_ids,
            renderer_outputs: RenderPluginRendererOutputs::default(),
            prepared_frame: None,
        }
    }

    pub fn with_renderer_outputs(mut self, renderer_outputs: RenderPluginRendererOutputs) -> Self {
        self.renderer_outputs = renderer_outputs;
        self
    }

    pub fn with_prepared_frame(
        mut self,
        prepared_frame: Option<RenderHybridGiPreparedFrame>,
    ) -> Self {
        self.prepared_frame = prepared_frame;
        self
    }

    pub fn evictable_probe_ids(&self) -> &[u32] {
        &self.evictable_probe_ids
    }

    pub fn renderer_outputs(&self) -> &RenderPluginRendererOutputs {
        &self.renderer_outputs
    }

    pub fn prepared_frame(&self) -> Option<&RenderHybridGiPreparedFrame> {
        self.prepared_frame.as_ref()
    }

    pub fn into_evictable_probe_ids(self) -> Vec<u32> {
        self.evictable_probe_ids
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<u32>,
        RenderPluginRendererOutputs,
        Option<RenderHybridGiPreparedFrame>,
    ) {
        (
            self.evictable_probe_ids,
            self.renderer_outputs,
            self.prepared_frame,
        )
    }
}

#[cfg(test)]
#[path = "tests/prepare_output.rs"]
mod tests;
