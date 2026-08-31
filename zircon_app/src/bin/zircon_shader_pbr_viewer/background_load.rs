use std::any::Any;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, TryRecvError},
    Arc,
};
use std::thread::JoinHandle;
use std::time::Duration;

use zircon_runtime::core::runtime::tasks::spawn_named_thread;

pub(crate) enum BackgroundTaskPoll<T> {
    Pending,
    Completed(Result<T, String>),
}

#[derive(Clone)]
pub(crate) struct BackgroundTaskCancellation {
    cancelled: Arc<AtomicBool>,
}

impl BackgroundTaskCancellation {
    pub(crate) fn is_cancel_requested(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BackgroundTaskShutdown {
    CompletedAndJoined,
    CancelledAndJoined,
    TimedOut,
    JoinPanicked,
}

pub(crate) struct BackgroundTask<T> {
    receiver: Receiver<Result<T, String>>,
    cancellation: BackgroundTaskCancellation,
    join_handle: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> BackgroundTask<T> {
    pub(crate) fn spawn(
        thread_name: &str,
        job: impl FnOnce(BackgroundTaskCancellation) -> Result<T, String> + Send + 'static,
        wake_event_loop: impl FnOnce() + Send + 'static,
    ) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let cancellation = BackgroundTaskCancellation {
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let worker_cancellation = cancellation.clone();
        let join_handle = spawn_named_thread(thread_name, move || {
            let result = if worker_cancellation.is_cancel_requested() {
                Err(background_cancellation_message())
            } else {
                catch_unwind(AssertUnwindSafe(|| job(worker_cancellation.clone())))
                    .map_err(background_panic_message)
                    .and_then(|result| result)
            };
            let result = if worker_cancellation.is_cancel_requested() {
                Err(background_cancellation_message())
            } else {
                result
            };
            let _ = sender.send(result);
            let _ = catch_unwind(AssertUnwindSafe(wake_event_loop));
        })
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;
        Ok(Self {
            receiver,
            cancellation,
            join_handle: Some(join_handle),
        })
    }

    pub(crate) fn request_cancel(&self) -> bool {
        !self.cancellation.cancelled.swap(true, Ordering::AcqRel)
    }

    pub(crate) fn is_cancellation_requested(&self) -> bool {
        self.cancellation.is_cancel_requested()
    }

    pub(crate) fn try_take(&mut self) -> BackgroundTaskPoll<T> {
        match self.receiver.try_recv() {
            Ok(result) => {
                BackgroundTaskPoll::Completed(self.join_worker().map_or_else(Err, |_| result))
            }
            Err(TryRecvError::Empty) => BackgroundTaskPoll::Pending,
            Err(TryRecvError::Disconnected) => {
                BackgroundTaskPoll::Completed(self.join_worker().map_or_else(Err, |_| {
                    Err(
                        "background scene loader disconnected before returning a result"
                            .to_string(),
                    )
                }))
            }
        }
    }

    pub(crate) fn cancel_and_join(mut self, timeout: Duration) -> BackgroundTaskShutdown {
        match self.receiver.try_recv() {
            Ok(_) => return self.join_shutdown_outcome(),
            Err(TryRecvError::Disconnected) => return self.join_shutdown_outcome(),
            Err(TryRecvError::Empty) => {}
        }

        self.request_cancel();
        match self.receiver.recv_timeout(timeout) {
            Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => self.join_shutdown_outcome(),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if self
                    .join_handle
                    .as_ref()
                    .is_some_and(JoinHandle::is_finished)
                {
                    self.join_shutdown_outcome()
                } else {
                    BackgroundTaskShutdown::TimedOut
                }
            }
        }
    }

    fn join_shutdown_outcome(&mut self) -> BackgroundTaskShutdown {
        match self.join_worker() {
            Ok(()) if self.is_cancellation_requested() => {
                BackgroundTaskShutdown::CancelledAndJoined
            }
            Ok(()) => BackgroundTaskShutdown::CompletedAndJoined,
            Err(_) => BackgroundTaskShutdown::JoinPanicked,
        }
    }

    fn join_worker(&mut self) -> Result<(), String> {
        let Some(join_handle) = self.join_handle.take() else {
            return Ok(());
        };
        join_handle.join().map_err(background_join_panic_message)
    }
}

fn background_panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        format!("background scene loader panicked: {message}")
    } else if let Some(message) = payload.downcast_ref::<String>() {
        format!("background scene loader panicked: {message}")
    } else {
        "background scene loader panicked with a non-string payload".to_string()
    }
}

fn background_join_panic_message(payload: Box<dyn Any + Send>) -> String {
    format!(
        "background scene loader panicked outside its job boundary: {}",
        background_panic_message(payload)
    )
}

fn background_cancellation_message() -> String {
    "background scene loader was cancelled before publication".to_string()
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::{BackgroundTask, BackgroundTaskPoll, BackgroundTaskShutdown};

    #[test]
    fn completed_background_load_wakes_and_returns_value() {
        let (wake_sender, wake_receiver) = mpsc::channel();
        let mut task = BackgroundTask::spawn(
            "viewer-background-load-success-test",
            |_| Ok(42_u32),
            move || {
                let _ = wake_sender.send(());
            },
        )
        .expect("background task should start");

        wake_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("background task should wake the event loop");
        match task.try_take() {
            BackgroundTaskPoll::Completed(Ok(value)) => assert_eq!(value, 42),
            _ => panic!("background task did not return its completed value"),
        }
    }

    #[test]
    fn panicking_background_load_wakes_and_returns_error() {
        let (wake_sender, wake_receiver) = mpsc::channel();
        let mut task = BackgroundTask::<u32>::spawn(
            "viewer-background-load-panic-test",
            |_| panic!("test panic"),
            move || {
                let _ = wake_sender.send(());
            },
        )
        .expect("background task should start");

        wake_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("panicking task should still wake the event loop");
        match task.try_take() {
            BackgroundTaskPoll::Completed(Err(message)) => {
                assert!(message.contains("test panic"));
            }
            _ => panic!("background task panic was not returned as an error"),
        }
    }

    #[test]
    fn cancellation_signal_is_observed_and_the_loader_is_joined() {
        let (started_sender, started_receiver) = mpsc::sync_channel(1);
        let (wake_sender, wake_receiver) = mpsc::channel();
        let task = BackgroundTask::spawn(
            "viewer-background-load-cancellation-test",
            move |cancellation| {
                started_sender
                    .send(())
                    .expect("test should observe the task start");
                while !cancellation.is_cancel_requested() {
                    std::thread::yield_now();
                }
                Ok(42_u32)
            },
            move || {
                let _ = wake_sender.send(());
            },
        )
        .expect("background task should start");

        started_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("background task should begin before cancellation");
        assert!(task.request_cancel());
        assert!(task.is_cancellation_requested());
        assert_eq!(
            task.cancel_and_join(Duration::from_secs(2)),
            BackgroundTaskShutdown::CancelledAndJoined
        );
        wake_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("cancelled task should wake the event loop after joining");
    }

    #[test]
    fn shutdown_timeout_is_distinct_from_a_joined_cancellation() {
        let (started_sender, started_receiver) = mpsc::sync_channel(1);
        let (release_sender, release_receiver) = mpsc::sync_channel(1);
        let (wake_sender, wake_receiver) = mpsc::channel();
        let task = BackgroundTask::spawn(
            "viewer-background-load-timeout-test",
            move |_| {
                started_sender
                    .send(())
                    .expect("test should observe the task start");
                release_receiver
                    .recv()
                    .expect("test should release the non-cooperative task");
                Ok(())
            },
            move || {
                let _ = wake_sender.send(());
            },
        )
        .expect("background task should start");

        started_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("background task should begin before cancellation");
        assert!(task.request_cancel());
        assert_eq!(
            task.cancel_and_join(Duration::ZERO),
            BackgroundTaskShutdown::TimedOut
        );
        release_sender
            .send(())
            .expect("test should release the timed-out task");
        wake_receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("timed-out task should eventually finish after release");
    }
}
