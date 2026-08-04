use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::core::framework::render::RenderCaptureReport;
use zr_rhi_wgpu::ReadbackError;

pub(crate) struct AsyncViewportCaptureRequest {
    callback: Box<dyn FnOnce(Result<Vec<u8>, ReadbackError>) + Send + 'static>,
    admission: Arc<AtomicBool>,
}

impl AsyncViewportCaptureRequest {
    pub(crate) fn new(
        callback: Box<dyn FnOnce(Result<Vec<u8>, ReadbackError>) + Send + 'static>,
    ) -> Self {
        Self {
            callback,
            admission: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn admission_state(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.admission)
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Box<dyn FnOnce(Result<Vec<u8>, ReadbackError>) + Send + 'static>,
        Arc<AtomicBool>,
    ) {
        (self.callback, self.admission)
    }
}

pub(crate) struct ViewportAsyncCaptureSubmission {
    pub(crate) generation: u64,
    pub(crate) capture_size: crate::core::math::UVec2,
    pub(crate) capture_report: RenderCaptureReport,
    pub(crate) capture_admitted: bool,
}

impl ViewportAsyncCaptureSubmission {
    pub(crate) const fn new(
        generation: u64,
        capture_size: crate::core::math::UVec2,
        capture_report: RenderCaptureReport,
        capture_admitted: bool,
    ) -> Self {
        Self {
            generation,
            capture_size,
            capture_report,
            capture_admitted,
        }
    }
}

pub(crate) fn capture_request_was_admitted(admission: &AtomicBool) -> bool {
    admission.load(Ordering::Acquire)
}
