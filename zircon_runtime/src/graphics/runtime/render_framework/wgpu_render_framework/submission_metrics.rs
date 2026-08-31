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
mod tests {
    #[test]
    fn submission_metrics_sampling_never_flushes_or_waits_for_a_runtime_frame() {
        let sampler = include_str!("submission_metrics.rs")
            .split("pub fn try_submission_metrics_snapshot")
            .nth(1)
            .and_then(|source| source.split("#[cfg(test)]").next())
            .expect("submission metrics sampler");

        assert!(sampler.contains("self.core.state.try_lock()"));
        assert!(sampler.contains("Err(TryLockError::WouldBlock) => return None"));
        assert!(sampler.contains("state.renderer.submission_metrics()"));
        assert!(!sampler.contains("finish_submission"));
        assert!(!sampler.contains("lock_operation"));
        assert!(!sampler.contains("queue.submit"));
    }
}
