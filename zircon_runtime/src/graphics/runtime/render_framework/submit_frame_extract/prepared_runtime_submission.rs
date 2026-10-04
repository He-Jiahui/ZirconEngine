//! 已准备的 provider 输出只与当前相机提交配对，淘汰列表和诊断不可跨帧混用。
use crate::core::framework::render::{
    RenderHybridGiPreparedFrame, RenderPluginRendererOutputs, RenderPreparedRuntimeSidebands,
};

#[derive(Default)]
pub(super) struct PreparedRuntimeSubmission {
    hybrid_gi_evictable_probe_ids: Vec<u32>,
    hybrid_gi_prepared_frame: Option<RenderHybridGiPreparedFrame>,
    virtual_geometry_evictable_page_ids: Vec<u32>,
    plugin_renderer_outputs: RenderPluginRendererOutputs,
}

impl PreparedRuntimeSubmission {
    pub(super) fn new(
        hybrid_gi_evictable_probe_ids: Vec<u32>,
        hybrid_gi_prepared_frame: Option<RenderHybridGiPreparedFrame>,
        virtual_geometry_evictable_page_ids: Vec<u32>,
        plugin_renderer_outputs: RenderPluginRendererOutputs,
    ) -> Self {
        Self {
            hybrid_gi_evictable_probe_ids,
            hybrid_gi_prepared_frame,
            virtual_geometry_evictable_page_ids,
            plugin_renderer_outputs,
        }
    }

    pub(super) fn into_prepared_runtime_sidebands(self) -> RenderPreparedRuntimeSidebands {
        RenderPreparedRuntimeSidebands::new(
            self.plugin_renderer_outputs,
            self.hybrid_gi_evictable_probe_ids,
            self.virtual_geometry_evictable_page_ids,
        )
        .with_hybrid_gi_prepared_frame(self.hybrid_gi_prepared_frame)
    }
}

#[cfg(test)]
#[path = "tests/prepared_runtime_submission.rs"]
mod tests;
