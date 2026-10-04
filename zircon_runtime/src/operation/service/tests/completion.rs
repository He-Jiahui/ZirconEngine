use std::sync::Arc;

use zircon_runtime_interface::{
    ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle, ZrRuntimeOperationPhase,
};

use super::super::super::task::RuntimeOperationTask;
use super::super::super::{
    RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationHandlerError,
    RuntimeOperationPrepared,
};
use super::RuntimeOperationService;

struct NoopHandler;

impl RuntimeOperationHandler for NoopHandler {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        _payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        unreachable!("the channel-loss fixture never snapshots")
    }

    fn prepare(
        &self,
        _snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        unreachable!("the channel-loss fixture never prepares")
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        unreachable!("the channel-loss fixture never applies")
    }
}

fn worker_task(
    handle: ZrRuntimeOperationHandle,
    phase: ZrRuntimeOperationPhase,
    detail_kind: ZrRuntimeOperationDetailKindV2,
    retained_bytes: usize,
) -> RuntimeOperationTask {
    RuntimeOperationTask {
        handle,
        operation_id: "test.worker-channel-loss".to_owned(),
        phase,
        detail_kind,
        detail_value: 0,
        handler: Arc::new(NoopHandler),
        payload: None,
        prepared_command: None,
        prepared_result: None,
        prepared_owner_state: None,
        snapshot_owner_bytes: 0,
        prepared_command_bytes: 0,
        prepared_result_bytes: 0,
        prepared_owner_bytes: 0,
        in_flight_owner_bytes: retained_bytes,
        retained_bytes,
        result: None,
        deadline: None,
        deadline_armed: true,
        terminal_at: None,
        harvest_in_flight: false,
        snapshot_claimed: false,
        prepare_in_flight: true,
        apply_claimed: false,
    }
}

#[test]
fn worker_channel_loss_fails_only_its_preparing_batch_and_releases_capacity() {
    let service = RuntimeOperationService::new();
    let preparing_handle = ZrRuntimeOperationHandle::new(1);
    let cancelled_handle = ZrRuntimeOperationHandle::new(2);
    {
        let mut state = service.lock_state();
        state.in_flight_prepares = 2;
        state.retained_bytes = 8;
        state.tasks.insert(
            preparing_handle,
            worker_task(
                preparing_handle,
                ZrRuntimeOperationPhase::Preparing,
                ZrRuntimeOperationDetailKindV2::None,
                8,
            ),
        );
        state.tasks.insert(
            cancelled_handle,
            worker_task(
                cancelled_handle,
                ZrRuntimeOperationPhase::Cancelled,
                ZrRuntimeOperationDetailKindV2::Cancelled,
                0,
            ),
        );
    }

    let (sender, receiver) = std::sync::mpsc::sync_channel(2);
    drop(sender);
    service
        .lock_completion_receivers()
        .push(super::super::RuntimeOperationCompletionReceiver {
            receiver,
            handles: vec![preparing_handle, cancelled_handle],
        });
    service.drain_prepare_completions();

    let status = service
        .poll(preparing_handle)
        .expect("lost worker task remains observable");
    assert_eq!(status.phase(), Some(ZrRuntimeOperationPhase::Failed));
    assert_eq!(
        status.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::WorkerChannelLost)
    );
    let cancelled = service
        .poll(cancelled_handle)
        .expect("cancelled worker task remains observable");
    assert_eq!(cancelled.phase(), Some(ZrRuntimeOperationPhase::Cancelled));
    assert_eq!(
        cancelled.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::Cancelled)
    );
    assert_eq!(service.lock_state().in_flight_prepares, 0);
    assert_eq!(service.lock_state().retained_bytes, 0);
    service.drain_prepare_completions();
    assert_eq!(service.lock_state().in_flight_prepares, 0);
    assert_eq!(service.lock_state().retained_bytes, 0);
}
