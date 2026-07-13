use std::fmt;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Mutex, MutexGuard};

use super::{JobError, JobId};

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
