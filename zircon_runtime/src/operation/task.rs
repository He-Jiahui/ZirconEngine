use std::sync::Arc;
use std::time::Instant;

use zircon_runtime_interface::{
    ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle, ZrRuntimeOperationPhase,
    ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
};

use super::RuntimeOperationHandler;

pub(super) struct RuntimeOperationTask {
    pub(super) handle: ZrRuntimeOperationHandle,
    pub(super) operation_id: String,
    pub(super) phase: ZrRuntimeOperationPhase,
    pub(super) detail_kind: ZrRuntimeOperationDetailKindV2,
    pub(super) detail_value: u64,
    pub(super) handler: Arc<dyn RuntimeOperationHandler>,
    pub(super) payload: Option<serde_json::Value>,
    pub(super) prepared_command: Option<serde_json::Value>,
    pub(super) prepared_result: Option<serde_json::Value>,
    pub(super) prepared_command_bytes: usize,
    pub(super) prepared_result_bytes: usize,
    pub(super) retained_bytes: usize,
    pub(super) result: Option<ZrRuntimeOperationResultV1>,
    pub(super) deadline: Option<Instant>,
    pub(super) deadline_armed: bool,
    pub(super) terminal_at: Option<Instant>,
    pub(super) harvest_in_flight: bool,
    pub(super) snapshot_claimed: bool,
    pub(super) prepare_in_flight: bool,
    pub(super) apply_claimed: bool,
}

impl RuntimeOperationTask {
    pub(super) fn status(&self) -> ZrRuntimeOperationStatusV2 {
        ZrRuntimeOperationStatusV2::new(
            self.handle,
            self.phase,
            if self.phase.is_terminal() { 1 } else { 0 },
            1,
            self.detail_kind,
            self.detail_value,
        )
    }
}
