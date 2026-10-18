//! 提供任务结果的一次消费及显式截止等待，供主循环轮询和关闭流程使用；丢弃票据不取消工作，并发等待期间另一调用可能暂时无法取得结果接收权。
use std::fmt;
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard};

use std::time::Instant;
use zircon_runtime::core::runtime::tasks::{JobHandle, TaskState};

use super::{JobError, JobId};

/// 单次结果接收权；共享引用允许轮询，取走结果后不能再次消费。
pub struct JobTicket<T> {
    id: JobId,
    result: Mutex<Option<Receiver<Result<T, JobError>>>>,
    execution: Option<Arc<JobExecution>>,
}

impl<T> JobTicket<T> {
    pub(super) fn new(id: JobId, result: Receiver<Result<T, JobError>>) -> Self {
        Self {
            id,
            result: Mutex::new(Some(result)),
            execution: None,
        }
    }

    pub(super) fn new_owned(
        id: JobId,
        result: Receiver<Result<T, JobError>>,
        execution: Arc<JobExecution>,
    ) -> Self {
        Self {
            id,
            result: Mutex::new(Some(result)),
            execution: Some(execution),
        }
    }

    /// Unlike result availability, this includes runtime task and observer completion.
    pub(crate) fn is_terminal(&self) -> bool {
        self.execution
            .as_ref()
            .is_some_and(|execution| execution.terminal_result().is_some())
    }

    pub(crate) fn try_take_terminal(&self) -> Option<Result<T, JobError>> {
        let terminal = self.execution.as_ref()?.terminal_result()?;
        if let Err(error) = terminal {
            return self.lock_result().take().map(|_| Err(error));
        }
        self.try_take()
    }

    pub fn id(&self) -> JobId {
        self.id
    }

    /// 消耗票据并阻塞取得终态；主循环宜使用轮询或明确截止等待，避免停住界面消息泵。
    pub fn wait(self) -> Result<T, JobError> {
        let receiver = self
            .result
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        receiver
            .and_then(|receiver| receiver.recv().ok())
            .unwrap_or(Err(JobError::ResultChannelClosed))
    }

    pub fn try_take(&self) -> Option<Result<T, JobError>> {
        let mut slot = self.lock_result();
        let receiver = slot.as_ref()?;
        match receiver.try_recv() {
            Ok(result) => {
                slot.take();
                Some(result)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                slot.take();
                Some(Err(JobError::ResultChannelClosed))
            }
        }
    }

    /// Waits no later than `deadline` while preserving a pending result for a later retry.
    pub fn wait_until(&self, deadline: Instant) -> Option<Result<T, JobError>> {
        // 限时等待临时取走接收权，避免持锁阻塞其他调用；并发轮询此时的空值不证明工作已完成。
        let receiver = self.lock_result().take()?;
        let wait = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(wait) {
            Ok(result) => Some(result),
            Err(RecvTimeoutError::Timeout) => {
                *self.lock_result() = Some(receiver);
                None
            }
            Err(RecvTimeoutError::Disconnected) => Some(Err(JobError::ResultChannelClosed)),
        }
    }

    fn lock_result(&self) -> MutexGuard<'_, Option<Receiver<Result<T, JobError>>>> {
        self.result
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl<T> fmt::Debug for JobTicket<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JobTicket")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

pub(super) struct JobExecution {
    state: Mutex<JobExecutionState>,
}

pub(super) enum JobExecutionState {
    AwaitingSchedule,
    Running(JobHandle),
    CompletedWithoutTask,
    SubmissionFailed(String),
}

impl JobExecution {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(JobExecutionState::AwaitingSchedule),
        }
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, JobExecutionState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn bind_cleanup(&self, handle: &JobHandle) {
        let mut state = self.lock();
        if matches!(*state, JobExecutionState::AwaitingSchedule) {
            *state = JobExecutionState::Running(handle.clone());
        }
    }

    pub(super) fn fail_submission(&self, message: &str) {
        let mut state = self.lock();
        if matches!(*state, JobExecutionState::AwaitingSchedule) {
            *state = JobExecutionState::SubmissionFailed(message.to_owned());
        }
    }

    pub(super) fn complete_without_task(&self) {
        let mut state = self.lock();
        if matches!(*state, JobExecutionState::AwaitingSchedule) {
            *state = JobExecutionState::CompletedWithoutTask;
        }
    }

    pub(super) fn terminal_result(&self) -> Option<Result<(), JobError>> {
        match &*self.lock() {
            JobExecutionState::AwaitingSchedule => None,
            JobExecutionState::Running(handle) => {
                handle.terminal_state().map(|state| match state {
                    TaskState::Completed => Ok(()),
                    TaskState::Cancelled => Err(JobError::Cancelled),
                    _ => Err(JobError::Panicked(
                        "editor job runtime task failed before terminal completion".to_string(),
                    )),
                })
            }
            JobExecutionState::CompletedWithoutTask => Some(Ok(())),
            JobExecutionState::SubmissionFailed(message) => {
                Some(Err(JobError::Panicked(message.clone())))
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/ticket.rs"]
mod tests;
