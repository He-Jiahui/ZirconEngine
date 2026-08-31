use std::sync::mpsc::{Receiver, SyncSender};

use zircon_runtime_interface::{ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle};

pub(super) enum RuntimeOperationPrepareCompletion {
    Prepared {
        handle: ZrRuntimeOperationHandle,
        command: serde_json::Value,
        result: serde_json::Value,
        command_bytes: usize,
        result_bytes: usize,
    },
    Failed {
        handle: ZrRuntimeOperationHandle,
        error: String,
        detail_kind: ZrRuntimeOperationDetailKindV2,
    },
}

pub(super) struct RuntimeOperationCompletionBatch {
    pub(super) sender: SyncSender<RuntimeOperationPrepareCompletion>,
    pub(super) receiver: Receiver<RuntimeOperationPrepareCompletion>,
    pub(super) handles: Vec<ZrRuntimeOperationHandle>,
}

pub(super) struct RuntimeOperationCompletionReceiver {
    pub(super) receiver: Receiver<RuntimeOperationPrepareCompletion>,
    pub(super) handles: Vec<ZrRuntimeOperationHandle>,
}
