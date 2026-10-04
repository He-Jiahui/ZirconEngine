use crate::core::framework::render::RenderPluginRendererOutputs;

#[derive(Clone, Debug, Default, PartialEq)]
/// 虚拟几何准备结果；可淘汰页候选与 renderer 输出成对交给提交阶段，后者只消费一次。
pub struct VirtualGeometryRuntimePrepareOutput {
    evictable_page_ids: Vec<u32>,
    renderer_outputs: RenderPluginRendererOutputs,
}

impl VirtualGeometryRuntimePrepareOutput {
    pub fn new(evictable_page_ids: Vec<u32>) -> Self {
        Self {
            evictable_page_ids,
            renderer_outputs: RenderPluginRendererOutputs::default(),
        }
    }

    pub fn with_renderer_outputs(mut self, renderer_outputs: RenderPluginRendererOutputs) -> Self {
        self.renderer_outputs = renderer_outputs;
        self
    }

    pub fn renderer_outputs(&self) -> &RenderPluginRendererOutputs {
        &self.renderer_outputs
    }

    pub fn into_evictable_page_ids(self) -> Vec<u32> {
        self.evictable_page_ids
    }

    pub(crate) fn into_parts(self) -> (Vec<u32>, RenderPluginRendererOutputs) {
        (self.evictable_page_ids, self.renderer_outputs)
    }
}

#[cfg(test)]
#[path = "tests/prepare_output.rs"]
mod tests;
