use std::sync::TryLockError;

use super::wgpu_render_framework::WgpuRenderFramework;

impl WgpuRenderFramework {
    /// Returns a monotonic WGPU submission snapshot without flushing queued frame work.
    ///
    /// Performance tooling samples this before and after a fixed workload, then computes deltas.
    /// It deliberately does not call finish_submission or wait for renderer state: callers skip a
    /// sample when an active frame owns the state lock.
    pub fn try_submission_metrics_snapshot(
        &self,
    ) -> Option<zr_rhi_wgpu::WgpuSubmissionMetricsSnapshot> {
        let state = match self.core.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => return None,
        };
        Some(state.renderer.submission_metrics())
    }
}

#[cfg(test)]
#[path = "tests/submission_metrics.rs"]
mod tests;
