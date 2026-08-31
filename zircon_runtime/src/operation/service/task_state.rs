use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use zircon_runtime_interface::{ZrRuntimeOperationHandle, ZrRuntimeOperationPhase};

use crate::core::runtime::tasks::TaskTimerSubscription;

use super::super::task::RuntimeOperationTask;
use super::super::RuntimeOperationServiceError;

#[derive(Default)]
pub(in crate::operation) struct RuntimeOperationTaskState {
    pub(in crate::operation) next_handle: u64,
    pub(in crate::operation) tasks: HashMap<ZrRuntimeOperationHandle, RuntimeOperationTask>,
    pub(in crate::operation) queued_snapshot_tasks: VecDeque<ZrRuntimeOperationHandle>,
    pub(in crate::operation) ready_apply_tasks: VecDeque<ZrRuntimeOperationHandle>,
    pub(in crate::operation) retained_bytes: usize,
    pub(in crate::operation) pending_admissions: usize,
    pub(in crate::operation) pending_admission_bytes: usize,
    pub(in crate::operation) in_flight_prepares: usize,
    pub(in crate::operation) maintenance_subscription: Option<TaskTimerSubscription>,
    pub(in crate::operation) maintenance_deadline: Option<Instant>,
    pub(in crate::operation) maintenance_generation: u64,
}

impl RuntimeOperationTaskState {
    pub(super) fn compact_phase_indexes(&mut self, maximum_tasks: usize) {
        if self.queued_snapshot_tasks.len() >= maximum_tasks {
            let tasks = &self.tasks;
            self.queued_snapshot_tasks.retain(|handle| {
                tasks.get(handle).is_some_and(|task| {
                    task.phase == ZrRuntimeOperationPhase::Queued && !task.snapshot_claimed
                })
            });
        }
        if self.ready_apply_tasks.len() >= maximum_tasks {
            let tasks = &self.tasks;
            self.ready_apply_tasks.retain(|handle| {
                tasks.get(handle).is_some_and(|task| {
                    task.phase == ZrRuntimeOperationPhase::ReadyToApply && !task.apply_claimed
                })
            });
        }
    }

    pub(super) fn allocate_handle(
        &mut self,
    ) -> Result<ZrRuntimeOperationHandle, RuntimeOperationServiceError> {
        if self.next_handle == 0 {
            self.next_handle = 1;
        }
        let handle = ZrRuntimeOperationHandle::new(self.next_handle);
        self.next_handle = self
            .next_handle
            .checked_add(1)
            .ok_or(RuntimeOperationServiceError::HandleExhausted)?;
        Ok(handle)
    }
}
