use crate::core::framework::render::RenderFrameSubmissionReceipt;
use zr_rhi_wgpu::WgpuSubmissionMetricsSnapshot;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn last_frame_submission_receipt(&self) -> Option<RenderFrameSubmissionReceipt> {
        self.last_frame_submission_receipt.clone()
    }

    /// Returns native WGPU submission facts without advancing the render timeline.
    pub(crate) fn submission_metrics(&self) -> WgpuSubmissionMetricsSnapshot {
        self.backend.submission_metrics()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn scene_renderer_forwards_submission_metrics_without_owning_queue_work() {
        let source = include_str!("scene_renderer_submission_metrics.rs");

        assert!(source.contains("self.backend.submission_metrics()"));
        assert!(!source.contains("queue.submit"));
        assert!(!source.contains("queue.write_"));
    }
}
