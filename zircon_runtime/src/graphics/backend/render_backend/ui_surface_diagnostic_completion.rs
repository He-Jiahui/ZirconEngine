//! Reuses the backend's existing device timeline and bounded delivery router.
use super::product_diagnostic_delivery_router::ProductDiagnosticDeliveryRouter;
use std::sync::{Arc, Mutex};
use zr_rhi::{
    DiagnosticReadbackAdmission, DiagnosticReadbackRequestId, RenderDevice, RhiError,
    SubmissionPollReceipt,
};
use zr_rhi_wgpu::{
    WgpuRenderDevice, WgpuUiSurfaceDiagnosticCompletionOwner,
    WgpuUiSurfaceDiagnosticReceiptCallback,
};

pub(super) struct UiSurfaceDiagnosticCompletionOwner {
    device: Arc<WgpuRenderDevice>,
    router: Arc<Mutex<ProductDiagnosticDeliveryRouter>>,
}
impl UiSurfaceDiagnosticCompletionOwner {
    pub(super) fn new(
        device: Arc<WgpuRenderDevice>,
        router: Arc<Mutex<ProductDiagnosticDeliveryRouter>>,
    ) -> Self {
        Self { device, router }
    }
}
pub(super) fn dispatch_deliveries(
    device: &WgpuRenderDevice,
    router: &Mutex<ProductDiagnosticDeliveryRouter>,
) {
    let dispatches = {
        let mut router = router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        router.collect_query_results(device);
        router.collect_dispatches(device)
    };
    run_dispatches(router, dispatches);
}

/// Executes detached production deliveries outside the router lock.
pub(super) fn run_dispatches(
    router: &Mutex<ProductDiagnosticDeliveryRouter>,
    dispatches: Vec<super::product_diagnostic_delivery_router::ProductDiagnosticDispatch>,
) {
    let panics = dispatches
        .into_iter()
        .map(|dispatch| usize::from(dispatch.run()))
        .sum::<usize>();
    if panics > 0 {
        let mut router = router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for _ in 0..panics {
            router.record_callback_panic();
        }
    }
}
pub(super) fn drain_completion_owner_once(
    device: &WgpuRenderDevice,
    router: &Mutex<ProductDiagnosticDeliveryRouter>,
) -> Result<SubmissionPollReceipt, RhiError> {
    let receipt = device.poll_submissions()?;
    dispatch_deliveries(device, router);
    Ok(receipt)
}
impl WgpuUiSurfaceDiagnosticCompletionOwner for UiSurfaceDiagnosticCompletionOwner {
    fn register_receipt_callback(
        &self,
        admission: DiagnosticReadbackAdmission,
        callback: WgpuUiSurfaceDiagnosticReceiptCallback,
    ) -> Result<bool, RhiError> {
        self.router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .register_with_receipt(admission, callback)
            .map_err(|reason| RhiError::SurfaceUnavailable(reason))
    }
    fn cancel_receipt_callback(&self, request: DiagnosticReadbackRequestId) {
        self.router
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .cancel_receipt_callback(request);
    }
    fn drain_once(&self) -> Result<SubmissionPollReceipt, RhiError> {
        drain_completion_owner_once(self.device.as_ref(), self.router.as_ref())
    }
}
