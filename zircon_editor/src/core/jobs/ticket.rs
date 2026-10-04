//! 提供任务结果的一次消费及显式截止等待，供主循环轮询和关闭流程使用；丢弃票据不取消工作，并发等待期间另一调用可能暂时无法取得结果接收权。
use std::fmt;
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use super::{JobError, JobId};

/// 单次结果接收权；共享引用允许轮询，取走结果后不能再次消费。
pub struct JobTicket<T> {
    id: JobId,
    result: Mutex<Option<Receiver<Result<T, JobError>>>>,
}

impl<T> JobTicket<T> {
    pub(super) fn new(id: JobId, result: Receiver<Result<T, JobError>>) -> Self {
        Self {
            id,
            result: Mutex::new(Some(result)),
        }
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

#[cfg(test)]
#[path = "tests/ticket.rs"]
mod tests;
