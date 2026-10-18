#[path = "admission.rs"]
mod admission;
#[path = "cancellation_token.rs"]
mod cancellation_token;
#[path = "category.rs"]
mod category;
#[path = "context.rs"]
mod context;
#[path = "error.rs"]
mod error;
#[path = "event.rs"]
mod event;
#[path = "event_journal/mod.rs"]
pub(super) mod event_journal;
#[path = "event_sink.rs"]
mod event_sink;
#[path = "id.rs"]
mod id;
#[path = "job.rs"]
mod job;
#[path = "limits.rs"]
mod limits;
#[path = "mutex_group.rs"]
mod mutex_group;
#[path = "progress.rs"]
mod progress;
#[path = "pump.rs"]
mod pump;
#[path = "quota_settings.rs"]
mod quota_settings;
#[path = "shutdown.rs"]
mod shutdown;
#[path = "spec.rs"]
mod spec;
#[path = "system/mod.rs"]
mod system;
#[cfg(test)]
#[path = "tests/test_support.rs"]
mod test_support;
#[path = "ticket.rs"]
mod ticket;

pub use admission::{EditorJobAdmission, EditorJobAdmissionKey, EditorJobAdmissionRequest};
pub use cancellation_token::CancellationToken;
pub use category::{JobCategory, JobPriority};
pub use context::JobContext;
pub use error::{JobAdmissionKeyError, JobError, JobFailure, JobSubmitError, MutexGroupError};
pub use event::{JobEvent, JobEventKind};
pub use event_journal::{
    EditorJobEventJournalGap, EditorJobEventJournalLimits, EditorJobEventJournalSnapshot,
};
pub use id::JobId;
pub use job::EditorJob;
pub use limits::{EditorJobAdmissionLimits, EditorJobAdmissionSnapshot, EditorJobLimits};
pub use mutex_group::MutexGroup;
pub use progress::{
    EditorJobPrimaryProgressSnapshot, EditorJobProgress, EditorJobProgressObserver,
    EditorJobProgressSnapshot, EditorJobProgressSource,
};
pub use pump::{JobEventPumpBudget, DEFAULT_JOB_EVENT_PUMP_BUDGET};
pub use quota_settings::{
    register_editor_job_quota_settings, resolve_editor_job_limits, EditorJobQuotaSettingsError,
    EDITOR_JOB_EXPORT_QUOTA_KEY, EDITOR_JOB_INTERACTIVE_SAVE_QUOTA_KEY, EDITOR_JOB_PLAY_QUOTA_KEY,
    EDITOR_JOB_THUMBNAIL_QUOTA_KEY,
};
pub use shutdown::{
    EditorJobShutdownFailure, EditorJobShutdownProgress, EditorJobShutdownState,
    UnfinishedEditorJob,
};
pub use spec::EditorJobSpec;
pub use system::{EditorJobAdmissionWindow, EditorJobBatchAdmissionReservation, EditorJobSystem};
pub use ticket::JobTicket;

#[cfg(test)]
pub(crate) use test_support::{
    test_job_scheduler, test_job_system, test_job_system_with_bus, test_job_system_with_limits,
};

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
