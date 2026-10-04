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
            crate::profile_counter!(
                "runtime",
                "operation.completion_lost_rows",
                lost_batches.iter().map(Vec::len).sum::<usize>()
            );
            for handles in lost_batches {
                terminal_transition |= self.fail_worker_completion_channel(&handles);
            }
            let Some(completion) = completion else {
                if !has_lost_batches {
                    break;
                }
                continue;
            };
            crate::profile_counter!("runtime", "operation.completion_rows", 1);
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
        crate::profile_counter!(
            "runtime",
            "operation.completion_receiver_rows",
            receivers.len()
        );
        let mut lost_batches = self
            .lost_completion_batches
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .drain(..)
            .collect::<Vec<_>>();
        let mut index = 0;
        while index < receivers.len() {
            crate::profile_counter!("runtime", "operation.completion_receiver_probe", 1);
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
                owner_state,
                owner_bytes,
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
                let Some(prepared_bytes) = command_bytes
                    .checked_add(result_bytes)
                    .and_then(|bytes| bytes.checked_add(owner_bytes))
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
                    task.prepared_owner_state = owner_state;
                    task.snapshot_owner_bytes = 0;
                    task.prepared_command_bytes = command_bytes;
                    task.prepared_result_bytes = result_bytes;
                    task.prepared_owner_bytes = owner_bytes;
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
                let should_finish = state
                    .tasks
                    .get(&handle)
                    .is_some_and(|task| task.phase == ZrRuntimeOperationPhase::Preparing);
                if should_finish {
                    self.finish_failed_task(&mut state, handle, error, detail_kind, 0);
                }
                should_finish
            }
        }
    }

    fn release_prepare_slot(
        state: &mut super::RuntimeOperationTaskState,
        handle: ZrRuntimeOperationHandle,
    ) -> bool {
        let released_bytes = {
            let Some(task) = state.tasks.get_mut(&handle) else {
                return false;
            };
            if !std::mem::replace(&mut task.prepare_in_flight, false) {
                return false;
            }
            let released_bytes = std::mem::replace(&mut task.in_flight_owner_bytes, 0);
            task.snapshot_owner_bytes = 0;
            released_bytes
        };
        state.in_flight_prepares = state
            .in_flight_prepares
            .checked_sub(1)
            .expect("operation prepare completion must have an in-flight slot");
        state.retained_bytes = state
            .retained_bytes
            .checked_sub(released_bytes)
            .expect("worker input bytes must remain accounted until completion cleanup");
        true
    }

    fn fail_worker_completion_channel(&self, handles: &[ZrRuntimeOperationHandle]) -> bool {
        let mut state = self.lock_state();
        let mut terminal_transition = false;
        for handle in handles {
            if !Self::release_prepare_slot(&mut state, *handle) {
                continue;
            }
            let should_fail = state
                .tasks
                .get(handle)
                .is_some_and(|task| task.phase == ZrRuntimeOperationPhase::Preparing);
            if should_fail {
                self.finish_failed_task(
                    &mut state,
                    *handle,
                    "runtime operation worker completion channel closed",
                    ZrRuntimeOperationDetailKindV2::WorkerChannelLost,
                    0,
                );
                terminal_transition = true;
            }
        }
        terminal_transition
    }
}

#[cfg(test)]
#[path = "tests/completion.rs"]
mod tests;
