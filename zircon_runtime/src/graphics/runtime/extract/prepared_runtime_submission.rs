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
    virtual_geometry_prepared_cpu_frame: Option<
        std::sync::Arc<crate::core::framework::render::RenderVirtualGeometryPreparedCpuFrame>,
    >,
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
            virtual_geometry_prepared_cpu_frame: None,
        }
    }

    pub(super) fn with_virtual_geometry_prepared_cpu_frame(
        mut self,
        frame: Option<
            std::sync::Arc<crate::core::framework::render::RenderVirtualGeometryPreparedCpuFrame>,
        >,
    ) -> Self {
        self.virtual_geometry_prepared_cpu_frame = frame;
        self
    }
    pub(super) fn virtual_geometry_prepared_cpu_frame(
        &self,
    ) -> Option<std::sync::Arc<crate::core::framework::render::RenderVirtualGeometryPreparedCpuFrame>>
    {
        self.virtual_geometry_prepared_cpu_frame.clone()
    }
    pub(super) fn into_prepared_runtime_sidebands(self) -> RenderPreparedRuntimeSidebands {
        RenderPreparedRuntimeSidebands::new(
            self.plugin_renderer_outputs,
            self.hybrid_gi_evictable_probe_ids,
            self.virtual_geometry_evictable_page_ids,
        )
        .with_hybrid_gi_prepared_frame(self.hybrid_gi_prepared_frame)
        .with_virtual_geometry_prepared_cpu_frame(self.virtual_geometry_prepared_cpu_frame)
    }
}

#[cfg(test)]
#[path = "tests/prepared_runtime_submission.rs"]
mod tests;
