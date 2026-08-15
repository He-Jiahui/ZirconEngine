use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::SyncSender;

use crate::core::jobs::{EditorJob, JobContext, JobError, JobEventKind};

pub(super) type PendingCancelTask = Box<dyn FnOnce(JobContext) + Send + 'static>;

pub(super) trait PendingTask: Any + Send {
    fn run(self: Box<Self>, context: JobContext);
    fn replace_with(&mut self, latest: Box<dyn PendingTask>) -> bool;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

impl<F> PendingTask for F
where
    F: FnOnce(JobContext) + Send + 'static,
{
    fn run(self: Box<Self>, context: JobContext) {
        (*self)(context);
    }

    fn replace_with(&mut self, _latest: Box<dyn PendingTask>) -> bool {
        false
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

pub(super) struct LatestPendingTask<J>
where
    J: EditorJob,
{
    job: J,
    sender: SyncSender<Result<J::Output, JobError>>,
}

impl<J> LatestPendingTask<J>
where
    J: EditorJob,
{
    pub(super) fn new(job: J, sender: SyncSender<Result<J::Output, JobError>>) -> Self {
        Self { job, sender }
    }
}

impl<J> PendingTask for LatestPendingTask<J>
where
    J: EditorJob,
{
    fn run(self: Box<Self>, context: JobContext) {
        let Self { job, sender } = *self;
        let event_context = context.clone();
        let result = if context.is_cancelled() {
            Err(JobError::Cancelled)
        } else {
            catch_unwind(AssertUnwindSafe(|| job.run(context)))
                .unwrap_or_else(|payload| Err(JobError::Panicked(panic_message(payload))))
        };
        let kind = match &result {
            Ok(_) => JobEventKind::Completed,
            Err(JobError::Cancelled) => JobEventKind::Cancelled,
            Err(error) => JobEventKind::Failed {
                message: error.to_string(),
            },
        };
        event_context.emit(kind);
        let _ = sender.send(result);
    }

    fn replace_with(&mut self, latest: Box<dyn PendingTask>) -> bool {
        let Ok(latest) = latest.into_any().downcast::<Self>() else {
            return false;
        };
        self.job = latest.job;
        true
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}
