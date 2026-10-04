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
#[path = "tests/scene_renderer_submission_metrics.rs"]
mod tests;
