use super::scene_renderer::SceneRenderer;
use crate::graphics::scene::scene_renderer::environment::RealtimeIblGpuTimingReport;

impl SceneRenderer {
    pub fn realtime_ibl_gpu_timing_supported(&self) -> bool {
        self.core.realtime_ibl.gpu_timestamps_supported()
    }

    pub fn take_realtime_ibl_gpu_timing_reports(&mut self) -> Vec<RealtimeIblGpuTimingReport> {
        self.core
            .realtime_ibl
            .take_gpu_timing_reports(&self.backend.device)
    }
}
