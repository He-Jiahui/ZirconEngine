use super::event_sink::JobEventSink;
use super::{CancellationToken, JobError, JobEventKind};

#[derive(Clone, Debug)]
pub struct JobContext {
    cancel: CancellationToken,
    events: JobEventSink,
}

impl JobContext {
    pub(super) fn new(cancel: CancellationToken, events: JobEventSink) -> Self {
        Self { cancel, events }
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancel
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    pub fn check_cancelled(&self) -> Result<(), JobError> {
        if self.is_cancelled() {
            Err(JobError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub fn report_progress(&self, completed: u32, total: u32, message: impl Into<String>) {
        self.events.emit(JobEventKind::Progress {
            completed,
            total,
            message: message.into(),
        });
    }

    pub(super) fn emit(&self, kind: JobEventKind) {
        self.events.emit(kind);
    }
}
