use super::scene_renderer::SceneRenderer;
use crate::graphics::scene::scene_renderer::environment::{
    RealtimeIblCompiledGraphCacheStats, RealtimeIblCpuTimingReport, RealtimeIblGpuTimingReport,
    RealtimeIblStatusReport,
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

    pub(in crate::graphics) fn realtime_ibl_status_report(&self) -> RealtimeIblStatusReport {
        self.core.realtime_ibl.status_report()
    }

    pub(in crate::graphics) fn take_realtime_ibl_gpu_timing_reports(
        &mut self,
    ) -> Vec<RealtimeIblGpuTimingReport> {
        self.core.realtime_ibl.take_gpu_timing_reports()
    }

    pub(in crate::graphics) fn take_realtime_ibl_cpu_timing_reports(
        &mut self,
    ) -> Vec<RealtimeIblCpuTimingReport> {
        self.core.realtime_ibl.take_cpu_timing_reports()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn resolved_gpu_timing_reports_do_not_borrow_the_native_device() {
        let caller = include_str!("scene_renderer_realtime_ibl_diagnostics.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("realtime IBL diagnostic facade test boundary");
        let runtime = include_str!("../environment/realtime_ibl_runtime.rs");
        let owner = runtime
            .split("pub(in crate::graphics) fn take_gpu_timing_reports(")
            .nth(1)
            .and_then(|source| {
                source
                    .split("pub(in crate::graphics) fn take_cpu_timing_reports(")
                    .next()
            })
            .expect("realtime IBL GPU timing report owner");

        assert!(caller.contains(".take_gpu_timing_reports()"));
        assert!(!caller.contains("backend.device"));
        assert!(!owner.contains("wgpu::Device"));
        assert!(owner.contains("timestamp_collector.take_completed()"));
    }
}
