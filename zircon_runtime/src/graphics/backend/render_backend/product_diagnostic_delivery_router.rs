use std::collections::{HashMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};

use zr_rhi::{
    DiagnosticQueryPlan, DiagnosticReadbackAdmission, DiagnosticReadbackReceipt,
    DiagnosticReadbackRequestId, DiagnosticReadbackTerminal,
};
use zr_rhi_wgpu::{
    GpuDiagnosticQueryFramePlanSnapshot, WgpuDiagnosticQueryDelivery,
    WgpuDiagnosticReadbackDelivery, WgpuRenderDevice,
};

pub(super) type ProductDiagnosticReadbackCallback =
    Box<dyn FnOnce(Result<Vec<u8>, String>) + Send + 'static>;

pub(super) struct ProductDiagnosticDeliveryRouter {
    pending: HashMap<DiagnosticReadbackRequestId, ProductDiagnosticReadbackCallback>,
    pending_order: VecDeque<DiagnosticReadbackRequestId>,
    pending_limit: usize,
    scratch: Vec<WgpuDiagnosticReadbackDelivery>,
    routed_delivery_count: u64,
    rejected_request_count: u64,
    orphan_delivery_count: u64,
    callback_panic_count: u64,
    next_query_frame_index: u64,
    pending_query_frames: HashMap<u64, PendingProductDiagnosticQueryFrame>,
    pending_query_order: VecDeque<u64>,
    query_scratch: Vec<WgpuDiagnosticQueryDelivery>,
    ready_query_results: VecDeque<ProductDiagnosticQueryResult>,
    orphan_query_delivery_count: u64,
    dropped_query_result_count: u64,
}

impl ProductDiagnosticDeliveryRouter {
    pub(super) fn new(pending_limit: usize) -> Self {
        Self {
            pending: HashMap::new(),
            pending_order: VecDeque::new(),
            pending_limit,
            scratch: Vec::new(),
            routed_delivery_count: 0,
            rejected_request_count: 0,
            orphan_delivery_count: 0,
            callback_panic_count: 0,
            next_query_frame_index: 1,
            pending_query_frames: HashMap::new(),
            pending_query_order: VecDeque::new(),
            query_scratch: Vec::new(),
            ready_query_results: VecDeque::new(),
            orphan_query_delivery_count: 0,
            dropped_query_result_count: 0,
        }
    }

    pub(super) fn register(
        &mut self,
        admission: DiagnosticReadbackAdmission,
        callback: ProductDiagnosticReadbackCallback,
    ) -> Result<bool, String> {
        let (request, admitted) = match admission {
            DiagnosticReadbackAdmission::Admitted(request) => (request, true),
            DiagnosticReadbackAdmission::Rejected(receipt) => {
                self.rejected_request_count = self.rejected_request_count.saturating_add(1);
                (receipt.request(), false)
            }
        };
        if self.pending.contains_key(&request) {
            return Err(format!(
                "product diagnostic request {} was registered more than once",
                request.sequence()
            ));
        }
        if self.pending_limit == 0 {
            return Err("product diagnostic delivery router is disabled by its budget".to_string());
        }
        if self.pending.len() >= self.pending_limit {
            return Err(
                "product diagnostic delivery router exhausted its pending-request budget"
                    .to_string(),
            );
        }
        self.pending.insert(request, callback);
        self.pending_order.push_back(request);
        self.compact_pending_order_if_needed();
        Ok(admitted)
    }

    /// Drains the device delivery ring once and detaches callbacks for execution outside locks.
    pub(super) fn collect_dispatches(
        &mut self,
        device: &WgpuRenderDevice,
    ) -> Vec<ProductDiagnosticDispatch> {
        debug_assert!(self.scratch.is_empty());
        device.append_diagnostic_readback_deliveries(&mut self.scratch);
        if self.scratch.is_empty() {
            return Vec::new();
        }
        let mut dispatches = Vec::with_capacity(self.scratch.len());
        for delivery in self.scratch.drain(..) {
            let receipt = delivery.receipt();
            let Some(callback) = self.pending.remove(&receipt.request()) else {
                self.orphan_delivery_count = self.orphan_delivery_count.saturating_add(1);
                continue;
            };
            self.routed_delivery_count = self.routed_delivery_count.saturating_add(1);
            dispatches.push(ProductDiagnosticDispatch::new(
                callback,
                receipt,
                delivery.into_bytes(),
            ));
        }
        dispatches
    }

    pub(super) fn record_callback_panic(&mut self) {
        self.callback_panic_count = self.callback_panic_count.saturating_add(1);
    }

    pub(super) fn reserve_query_route(
        &mut self,
        renderer_frame_generation: u64,
    ) -> Result<u64, String> {
        if self.pending_limit == 0 {
            return Err("product diagnostic query router is disabled by its budget".to_string());
        }
        if self.pending_query_frames.len() >= self.pending_limit {
            return Err(
                "product diagnostic query router exhausted its pending-frame budget".to_string(),
            );
        }
        let query_frame_index = self.next_query_frame_index;
        self.next_query_frame_index =
            self.next_query_frame_index.checked_add(1).ok_or_else(|| {
                "product diagnostic query frame identity space is exhausted".to_string()
            })?;
        self.pending_query_frames.insert(
            query_frame_index,
            PendingProductDiagnosticQueryFrame {
                renderer_frame_generation,
                snapshot: None,
            },
        );
        self.pending_query_order.push_back(query_frame_index);
        self.compact_pending_query_order_if_needed();
        Ok(query_frame_index)
    }

    pub(super) fn register_query_plan(
        &mut self,
        query_frame_index: u64,
        snapshot: GpuDiagnosticQueryFramePlanSnapshot,
    ) -> Result<(), String> {
        let pending = self
            .pending_query_frames
            .get_mut(&query_frame_index)
            .ok_or_else(|| {
                format!("product diagnostic query frame {query_frame_index} has no pending route")
            })?;
        pending.snapshot = Some(snapshot);
        Ok(())
    }

    pub(super) fn cancel_query_route(&mut self, query_frame_index: u64) {
        self.pending_query_frames.remove(&query_frame_index);
    }

    pub(super) fn collect_query_results(&mut self, device: &WgpuRenderDevice) {
        debug_assert!(self.query_scratch.is_empty());
        device.append_diagnostic_query_deliveries(&mut self.query_scratch);
        let mut deliveries = std::mem::take(&mut self.query_scratch);
        for delivery in deliveries.drain(..) {
            let Some(pending) = self.pending_query_frames.remove(&delivery.frame_index) else {
                self.orphan_query_delivery_count =
                    self.orphan_query_delivery_count.saturating_add(1);
                continue;
            };
            let Some(snapshot) = pending.snapshot else {
                self.orphan_query_delivery_count =
                    self.orphan_query_delivery_count.saturating_add(1);
                continue;
            };
            let (plan, pass_names) = snapshot.into_parts();
            self.push_query_result(ProductDiagnosticQueryResult {
                renderer_frame_generation: pending.renderer_frame_generation,
                plan,
                pass_names,
                delivery,
            });
        }
        self.query_scratch = deliveries;
        self.compact_pending_query_order_if_needed();
    }

    pub(super) fn drain_query_results(&mut self) -> Vec<ProductDiagnosticQueryResult> {
        self.ready_query_results.drain(..).collect()
    }

    fn compact_pending_order_if_needed(&mut self) {
        let compact_threshold = self.pending_limit.saturating_mul(2);
        if self.pending_order.len() <= compact_threshold {
            return;
        }
        self.pending_order
            .retain(|request| self.pending.contains_key(request));
    }

    fn compact_pending_query_order_if_needed(&mut self) {
        let compact_threshold = self.pending_limit.saturating_mul(2);
        if self.pending_query_order.len() <= compact_threshold {
            return;
        }
        self.pending_query_order
            .retain(|frame| self.pending_query_frames.contains_key(frame));
    }

    fn push_query_result(&mut self, result: ProductDiagnosticQueryResult) {
        if self.pending_limit == 0 {
            self.dropped_query_result_count = self.dropped_query_result_count.saturating_add(1);
            return;
        }
        if self.ready_query_results.len() >= self.pending_limit {
            self.ready_query_results.pop_front();
            self.dropped_query_result_count = self.dropped_query_result_count.saturating_add(1);
        }
        self.ready_query_results.push_back(result);
    }
}

struct PendingProductDiagnosticQueryFrame {
    renderer_frame_generation: u64,
    snapshot: Option<GpuDiagnosticQueryFramePlanSnapshot>,
}

pub(crate) struct ProductDiagnosticQueryResult {
    pub(crate) renderer_frame_generation: u64,
    pub(crate) plan: DiagnosticQueryPlan,
    pub(crate) pass_names: Vec<String>,
    pub(crate) delivery: WgpuDiagnosticQueryDelivery,
}

pub(super) struct ProductDiagnosticDispatch {
    callback: ProductDiagnosticReadbackCallback,
    receipt: DiagnosticReadbackReceipt,
    bytes: Option<Vec<u8>>,
}

impl ProductDiagnosticDispatch {
    fn new(
        callback: ProductDiagnosticReadbackCallback,
        receipt: DiagnosticReadbackReceipt,
        bytes: Option<Vec<u8>>,
    ) -> Self {
        Self {
            callback,
            receipt,
            bytes,
        }
    }

    pub(super) fn run(self) -> bool {
        let result = product_diagnostic_result(self.receipt, self.bytes);
        catch_unwind(AssertUnwindSafe(|| (self.callback)(result))).is_err()
    }
}

fn product_diagnostic_result(
    receipt: DiagnosticReadbackReceipt,
    bytes: Option<Vec<u8>>,
) -> Result<Vec<u8>, String> {
    match (receipt.terminal(), bytes) {
        (DiagnosticReadbackTerminal::Succeeded, Some(bytes)) => Ok(bytes),
        (DiagnosticReadbackTerminal::Succeeded, None) => Err(format!(
            "diagnostic request {} succeeded without a payload",
            receipt.request().sequence()
        )),
        (terminal, _) => Err(format!(
            "diagnostic request {} terminated as {terminal:?}",
            receipt.request().sequence()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zr_rhi::{
        DeviceGeneration, DeviceId, DiagnosticReadbackBudget, DiagnosticReadbackKind,
        DiagnosticReadbackTracker,
    };

    #[test]
    fn product_router_moves_delivery_bytes_and_runs_callbacks_outside_the_router() {
        let source = include_str!("product_diagnostic_delivery_router.rs");
        let collection = source
            .split("pub(super) fn collect_dispatches")
            .nth(1)
            .and_then(|source| source.split("pub(super) fn record_callback_panic").next())
            .expect("router collection method");

        assert!(collection.contains("append_diagnostic_readback_deliveries"));
        assert!(collection.contains("self.pending.remove(&receipt.request())"));
        assert!(collection.contains("delivery.into_bytes()"));
        assert!(!collection.contains("callback)("));
    }

    #[test]
    fn full_router_rejects_new_registration_without_orphaning_existing_owner() {
        let budget = DiagnosticReadbackBudget::default();
        let mut tracker =
            DiagnosticReadbackTracker::new(DeviceId::new(7), DeviceGeneration::initial(), budget);
        tracker.begin_frame(1).unwrap();
        let first = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
        let second = tracker.admit(DiagnosticReadbackKind::Buffer, 4).unwrap();
        let first_request = match first {
            DiagnosticReadbackAdmission::Admitted(request) => request,
            DiagnosticReadbackAdmission::Rejected(_) => panic!("first request must be admitted"),
        };
        let second_request = match second {
            DiagnosticReadbackAdmission::Admitted(request) => request,
            DiagnosticReadbackAdmission::Rejected(_) => panic!("second request must be admitted"),
        };
        let mut router = ProductDiagnosticDeliveryRouter::new(1);

        router.register(first, Box::new(|_| {})).unwrap();
        let error = router
            .register(second, Box::new(|_| {}))
            .expect_err("full router must fail instead of dropping an unresolved callback");

        assert!(error.contains("pending-request budget"));
        assert!(router.pending.contains_key(&first_request));
        assert!(!router.pending.contains_key(&second_request));
        assert_eq!(router.orphan_delivery_count, 0);
    }
}
