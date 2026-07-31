use std::sync::Arc;

use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationPhase, ZrRuntimeOperationProgressV1,
    ZrRuntimeOperationResultV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::RuntimeOperationHandler;

pub(super) struct RuntimeOperationTask {
    pub(super) handle: ZrRuntimeOperationHandle,
    pub(super) operation_id: String,
    pub(super) phase: ZrRuntimeOperationPhase,
    pub(super) handler: Arc<dyn RuntimeOperationHandler>,
    pub(super) payload: Option<serde_json::Value>,
    pub(super) prepared: Option<serde_json::Value>,
    pub(super) retained_bytes: usize,
    pub(super) result: Option<ZrRuntimeOperationResultV1>,
}

impl RuntimeOperationTask {
    pub(super) fn progress(&self) -> ZrRuntimeOperationProgressV1 {
        let (completed_work, message) = match self.phase {
            ZrRuntimeOperationPhase::Queued => (0, "queued"),
            ZrRuntimeOperationPhase::Running => (0, "preparing"),
            ZrRuntimeOperationPhase::Completed => (1, "completed"),
            ZrRuntimeOperationPhase::Failed => (1, "failed"),
        };
        ZrRuntimeOperationProgressV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.handle,
            self.phase,
            completed_work,
            1,
            message,
        )
    }
}
