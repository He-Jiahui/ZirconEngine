use super::scene_renderer::SceneRenderer;
use crate::graphics::scene::scene_renderer::environment::{
    RealtimeIblCompiledGraphCacheStats, RealtimeIblGpuTimingReport,
};

impl SceneRenderer {
    pub(in crate::graphics) fn realtime_ibl_gpu_timing_supported(&self) -> bool {
        self.core.realtime_ibl.gpu_timestamps_supported()
    }

    pub(in crate::graphics) fn realtime_ibl_compiled_graph_cache_stats(
        &self,
    ) -> RealtimeIblCompiledGraphCacheStats {
        self.core.realtime_ibl.compiled_graph_cache_stats()
    }

    pub(in crate::graphics) fn take_realtime_ibl_gpu_timing_reports(
        &mut self,
    ) -> Vec<RealtimeIblGpuTimingReport> {
        self.core
            .realtime_ibl
            .take_gpu_timing_reports(&self.backend.device)
    }
}
