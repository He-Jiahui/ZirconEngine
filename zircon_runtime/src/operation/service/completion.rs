use std::sync::mpsc::TryRecvError;

use zircon_runtime_interface::{
    ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle, ZrRuntimeOperationPhase,
};

use super::{RuntimeOperationPrepareCompletion, RuntimeOperationService};

impl RuntimeOperationService {
    pub(super) fn drain_prepare_completions(&self) {
        let mut terminal_transition = false;
        loop {
            let (completion, lost_batches) = self.take_prepare_completion();
            let has_lost_batches = !lost_batches.is_empty();
            for handles in lost_batches {
                terminal_transition |= self.fail_worker_completion_channel(&handles);
            }
            let Some(completion) = completion else {
                if !has_lost_batches {
                    break;
                }
                continue;
            };
            terminal_transition |= self.apply_prepare_completion(completion);
        }
        if terminal_transition {
            self.refresh_maintenance_after_transition();
        }
    }

    fn take_prepare_completion(
        &self,
    ) -> (
        Option<RuntimeOperationPrepareCompletion>,
        Vec<Vec<ZrRuntimeOperationHandle>>,
    ) {
        let mut receivers = self.lock_completion_receivers();
        let mut lost_batches = Vec::new();
        let mut index = 0;
        while index < receivers.len() {
            match receivers[index].receiver.try_recv() {
                Ok(completion) => return (Some(completion), lost_batches),
                Err(TryRecvError::Empty) => index += 1,
                Err(TryRecvError::Disconnected) => {
                    lost_batches.push(receivers.swap_remove(index).handles);
                }
            }
        }
        (None, lost_batches)
    }

    fn apply_prepare_completion(&self, completion: RuntimeOperationPrepareCompletion) -> bool {
        let mut state = self.lock_state();
        match completion {
            RuntimeOperationPrepareCompletion::Prepared {
                handle,
                command,
                result,
                command_bytes,
                result_bytes,
            } => {
                if !Self::release_prepare_slot(&mut state, handle) {
                    return false;
                }
                let Some(previous_bytes) = state.tasks.get(&handle).and_then(|task| {
                    (task.phase == ZrRuntimeOperationPhase::Preparing)
                        .then_some(task.retained_bytes)
                }) else {
                    return false;
                };
                let Some(prepared_bytes) = command_bytes.checked_add(result_bytes) else {
                    self.finish_failed_task(
                        &mut state,
                        handle,
                        "runtime operation prepared command and result exceed retained byte budget",
                        ZrRuntimeOperationDetailKindV2::AdmissionByteLimit,
                        self.limits.max_retained_bytes as u64,
                    );
                    return true;
                };
                let Some(retained_bytes) = state
                    .retained_bytes
                    .checked_sub(previous_bytes)
                    .expect("runtime operation task bytes must remain accounted")
                    .checked_add(prepared_bytes)
                else {
                    self.finish_failed_task(
                        &mut state,
                        handle,
                        "runtime operation prepared command and result exceed retained byte budget",
                        ZrRuntimeOperationDetailKindV2::AdmissionByteLimit,
                        self.limits.max_retained_bytes as u64,
                    );
                    return true;
                };
                if retained_bytes > self.limits.max_retained_bytes {
                    self.finish_failed_task(
                        &mut state,
                        handle,
                        "runtime operation prepared command and result exceed retained byte budget",
                        ZrRuntimeOperationDetailKindV2::AdmissionByteLimit,
                        self.limits.max_retained_bytes as u64,
                    );
                    return true;
                }
                state.retained_bytes = retained_bytes;
                {
                    let Some(task) = state.tasks.get_mut(&handle) else {
                        return false;
                    };
                    task.prepared_command = Some(command);
                    task.prepared_result = Some(result);
                    task.prepared_command_bytes = command_bytes;
                    task.prepared_result_bytes = result_bytes;
                    task.retained_bytes = prepared_bytes;
                    task.phase = ZrRuntimeOperationPhase::ReadyToApply;
                    task.detail_kind = ZrRuntimeOperationDetailKindV2::None;
                    task.detail_value = 0;
                }
                state.ready_apply_tasks.push_back(handle);
                false
            }
            RuntimeOperationPrepareCompletion::Failed {
                handle,
                error,
                detail_kind,
            } => {
                if !Self::release_prepare_slot(&mut state, handle) {
                    return false;
                }
                self.finish_failed_task(&mut state, handle, error, detail_kind, 0);
                true
            }
        }
    }

    fn release_prepare_slot(
        state: &mut super::RuntimeOperationTaskState,
        handle: ZrRuntimeOperationHandle,
    ) -> bool {
        let Some(task) = state.tasks.get_mut(&handle) else {
            return false;
        };
        if !std::mem::replace(&mut task.prepare_in_flight, false) {
            return false;
        }
        state.in_flight_prepares = state
            .in_flight_prepares
            .checked_sub(1)
            .expect("operation prepare completion must have an in-flight slot");
        true
    }

    fn fail_worker_completion_channel(&self, handles: &[ZrRuntimeOperationHandle]) -> bool {
        let mut state = self.lock_state();
        let lost_prepare_count = handles
            .iter()
            .filter(|handle| {
                state
                    .tasks
                    .get(handle)
                    .is_some_and(|task| task.prepare_in_flight)
            })
            .count();
        if lost_prepare_count == 0 {
            return false;
        }
        state.in_flight_prepares = state
            .in_flight_prepares
            .checked_sub(lost_prepare_count)
            .expect("lost worker prepares must retain their in-flight slots");
        for handle in handles {
            let should_fail = state.tasks.get_mut(handle).is_some_and(|task| {
                let was_in_flight = std::mem::replace(&mut task.prepare_in_flight, false);
                was_in_flight && task.phase == ZrRuntimeOperationPhase::Preparing
            });
            if should_fail {
                self.finish_failed_task(
                    &mut state,
                    *handle,
                    "runtime operation worker completion channel closed",
                    ZrRuntimeOperationDetailKindV2::WorkerChannelLost,
                    0,
                );
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
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
            prepared_command_bytes: 0,
            prepared_result_bytes: 0,
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
        service.lock_completion_receivers().push(
            super::super::RuntimeOperationCompletionReceiver {
                receiver,
                handles: vec![preparing_handle, cancelled_handle],
            },
        );
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
    }
}
