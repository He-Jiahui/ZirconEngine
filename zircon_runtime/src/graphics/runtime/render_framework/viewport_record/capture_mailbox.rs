use std::sync::Arc;

use crate::core::framework::render::{CapturedFrame, RenderCaptureReport};
use crate::core::math::UVec2;
use crate::graphics::scene::AsyncViewportCaptureRequest;
use crate::graphics::CompiledRenderPipeline;
use zr_rhi_wgpu::GpuReadbackQueue;

use super::viewport_record::{
    PendingViewportCapture, ReadyViewportCapture, ViewportAsyncCaptureMailbox, ViewportRecord,
};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn async_capture_request(
        &self,
        generation: u64,
    ) -> AsyncViewportCaptureRequest {
        let mailbox = Arc::clone(&self.capture_mailbox);
        AsyncViewportCaptureRequest::new(Box::new(move |result| {
            let mut mailbox = mailbox
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            mailbox.complete(generation, result.map_err(|error| error.to_string()));
        }))
    }

    pub(in crate::graphics::runtime::render_framework) fn register_async_capture(
        &mut self,
        generation: u64,
        size: UVec2,
        capture_report: RenderCaptureReport,
        pipeline: Arc<CompiledRenderPipeline>,
    ) {
        self.compiled_pipeline = Some(Arc::clone(&pipeline));
        let mut mailbox = self
            .capture_mailbox
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        mailbox.register(
            generation,
            PendingViewportCapture {
                size,
                capture_report,
                pipeline,
            },
        );
    }

    pub(in crate::graphics::runtime::render_framework) fn promote_completed_async_capture(
        &mut self,
    ) -> bool {
        let ready = self
            .capture_mailbox
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take_ready();
        let Some(ready) = ready else {
            return false;
        };
        if !super::capture::capture_generation_is_newer(
            self.last_promoted_capture_generation,
            ready.capture.generation,
        ) {
            return false;
        }
        let mut capture = ready.capture;
        if let Some(profiles) = self.pending_capture_profiles.remove(&capture.generation) {
            for profile in &profiles {
                super::capture::attach_profile_to_matching_capture(&mut capture, profile);
            }
        }
        self.last_capture_pipeline = Some(Arc::clone(&ready.pipeline));
        self.last_promoted_capture_generation = Some(capture.generation);
        self.last_capture = Some(capture);
        true
    }
}

impl ViewportAsyncCaptureMailbox {
    pub(super) fn register(&mut self, generation: u64, pending: PendingViewportCapture) {
        self.pending.insert(generation, pending);
        self.trim_to_readback_ring();
        self.promote(generation);
    }

    pub(super) fn complete(&mut self, generation: u64, result: Result<Vec<u8>, String>) {
        self.completed.insert(generation, result);
        self.promote(generation);
    }

    pub(super) fn take_ready(&mut self) -> Option<ReadyViewportCapture> {
        self.ready.take()
    }

    pub(super) fn has_pending(&self, generation: u64) -> bool {
        self.pending.contains_key(&generation)
    }

    fn promote(&mut self, generation: u64) {
        let Some(result) = self.completed.remove(&generation) else {
            return;
        };
        let Some(pending) = self.pending.remove(&generation) else {
            self.completed.insert(generation, result);
            return;
        };
        let Ok(rgba) = result else {
            return;
        };
        let capture = CapturedFrame::with_capture_report(
            pending.size.x,
            pending.size.y,
            rgba,
            generation,
            pending.capture_report,
        );
        if self
            .ready
            .as_ref()
            .map_or(true, |ready| ready.capture.generation <= generation)
        {
            self.ready = Some(ReadyViewportCapture {
                capture,
                pipeline: pending.pipeline,
            });
        }
    }

    fn trim_to_readback_ring(&mut self) {
        while self.pending.len() > GpuReadbackQueue::FRAME_SLOTS {
            let Some(generation) = self.pending.keys().next().copied() else {
                return;
            };
            self.pending.remove(&generation);
            self.completed.remove(&generation);
        }
    }
}

impl Default for ViewportAsyncCaptureMailbox {
    fn default() -> Self {
        Self {
            pending: Default::default(),
            completed: Default::default(),
            ready: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::capture::capture_generation_is_newer;

    #[test]
    fn completed_capture_generation_never_moves_backwards() {
        assert!(capture_generation_is_newer(None, 8));
        assert!(capture_generation_is_newer(Some(8), 9));
        assert!(!capture_generation_is_newer(Some(8), 8));
        assert!(!capture_generation_is_newer(Some(8), 7));
    }
}
