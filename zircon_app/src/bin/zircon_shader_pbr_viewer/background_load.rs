use std::any::Any;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

pub(crate) enum BackgroundTaskPoll<T> {
    Pending,
    Completed(Result<T, String>),
}

pub(crate) struct BackgroundTask<T> {
    receiver: Receiver<Result<T, String>>,
}

impl<T: Send + 'static> BackgroundTask<T> {
    pub(crate) fn spawn(
        thread_name: &str,
        job: impl FnOnce() -> Result<T, String> + Send + 'static,
        wake_event_loop: impl FnOnce() + Send + 'static,
    ) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(move || {
                let result = catch_unwind(AssertUnwindSafe(job))
                    .map_err(background_panic_message)
                    .and_then(|result| result);
                let _ = sender.send(result);
                wake_event_loop();
            })?;
        Ok(Self { receiver })
    }

    pub(crate) fn try_take(&self) -> BackgroundTaskPoll<T> {
        match self.receiver.try_recv() {
            Ok(result) => BackgroundTaskPoll::Completed(result),
            Err(TryRecvError::Empty) => BackgroundTaskPoll::Pending,
            Err(TryRecvError::Disconnected) => BackgroundTaskPoll::Completed(Err(
                "background scene loader disconnected before returning a result".to_string(),
            )),
        }
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

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::{BackgroundTask, BackgroundTaskPoll};

    #[test]
    fn completed_background_load_wakes_and_returns_value() {
        let (wake_sender, wake_receiver) = mpsc::channel();
        let task = BackgroundTask::spawn(
            "viewer-background-load-success-test",
            || Ok(42_u32),
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
        let task = BackgroundTask::<u32>::spawn(
            "viewer-background-load-panic-test",
            || panic!("test panic"),
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
}
