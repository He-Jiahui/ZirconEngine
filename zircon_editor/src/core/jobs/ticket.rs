use std::fmt;
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

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

    /// Waits no later than `deadline` while preserving a pending result for a later retry.
    pub fn wait_until(&self, deadline: Instant) -> Option<Result<T, JobError>> {
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
mod tests {
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::JobTicket;
    use crate::core::jobs::{JobError, JobId};

    #[test]
    fn deadline_wait_consumes_a_ready_result_once() {
        let (sender, receiver) = mpsc::channel();
        sender.send(Ok(7_u32)).unwrap();
        let ticket = JobTicket::new(JobId::new(1), receiver);

        assert_eq!(
            ticket.wait_until(Instant::now() + Duration::from_secs(1)),
            Some(Ok(7))
        );
        assert_eq!(ticket.try_take(), None);
    }

    #[test]
    fn deadline_wait_timeout_preserves_the_result_receiver_for_retry() {
        let (sender, receiver) = mpsc::channel();
        let ticket = JobTicket::new(JobId::new(2), receiver);
        let started = Instant::now();

        assert_eq!(ticket.wait_until(started + Duration::from_millis(10)), None);
        assert!(started.elapsed() < Duration::from_millis(250));

        sender.send(Ok::<_, JobError>(11_u32)).unwrap();
        assert_eq!(ticket.try_take(), Some(Ok(11)));
    }

    #[test]
    fn deadline_wait_reports_a_disconnected_result_channel() {
        let (sender, receiver) = mpsc::channel::<Result<u32, JobError>>();
        drop(sender);
        let ticket = JobTicket::new(JobId::new(3), receiver);

        assert_eq!(
            ticket.wait_until(Instant::now() + Duration::from_secs(1)),
            Some(Err(JobError::ResultChannelClosed))
        );
        assert_eq!(ticket.try_take(), None);
    }

    #[test]
    fn one_deadline_waiter_does_not_hold_the_ticket_lock_until_its_deadline() {
        let (sender, receiver) = mpsc::channel();
        let ticket = Arc::new(JobTicket::new(JobId::new(4), receiver));
        let waiter_ticket = Arc::clone(&ticket);
        let waiter = std::thread::spawn(move || {
            waiter_ticket.wait_until(Instant::now() + Duration::from_secs(1))
        });
        while ticket.lock_result().is_some() {
            std::thread::yield_now();
        }

        let probe_started = Instant::now();
        assert_eq!(ticket.wait_until(Instant::now()), None);
        assert!(probe_started.elapsed() < Duration::from_millis(50));

        sender.send(Ok::<_, JobError>(13_u32)).unwrap();
        assert_eq!(waiter.join().unwrap(), Some(Ok(13)));
    }
}
