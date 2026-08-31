use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::core::framework::render::{RenderCaptureReport, RenderFrameSubmissionReceipt};

pub(crate) struct AsyncViewportCaptureRequest {
    callback: Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>,
    admission: Arc<AtomicBool>,
}

impl AsyncViewportCaptureRequest {
    pub(crate) fn new(callback: Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>) -> Self {
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
        Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>,
        Arc<AtomicBool>,
    ) {
        (self.callback, self.admission)
    }
}

pub(crate) struct ViewportAsyncCaptureSubmission {
    submission_receipt: RenderFrameSubmissionReceipt,
    viewport_product_copy: Option<zr_rhi_wgpu::WgpuUiExternalImageCopyReceipt>,
    pub(crate) capture_size: crate::core::math::UVec2,
    pub(crate) capture_report: RenderCaptureReport,
    pub(crate) capture_admitted: bool,
}

impl ViewportAsyncCaptureSubmission {
    pub(crate) fn new(
        submission_receipt: RenderFrameSubmissionReceipt,
        viewport_product_copy: Option<zr_rhi_wgpu::WgpuUiExternalImageCopyReceipt>,
        capture_size: crate::core::math::UVec2,
        capture_report: RenderCaptureReport,
        capture_admitted: bool,
    ) -> Self {
        Self {
            submission_receipt,
            viewport_product_copy,
            capture_size,
            capture_report,
            capture_admitted,
        }
    }

    pub(crate) const fn generation(&self) -> u64 {
        self.submission_receipt.frame_generation()
    }

    pub(crate) const fn submission_receipt(&self) -> &RenderFrameSubmissionReceipt {
        &self.submission_receipt
    }

    pub(crate) fn take_viewport_product_copy(
        &mut self,
    ) -> Option<zr_rhi_wgpu::WgpuUiExternalImageCopyReceipt> {
        self.viewport_product_copy.take()
    }
}

pub(crate) fn capture_request_was_admitted(admission: &AtomicBool) -> bool {
    admission.load(Ordering::Acquire)
}
