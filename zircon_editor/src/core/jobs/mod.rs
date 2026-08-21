mod admission;
mod cancellation_token;
mod category;
mod context;
mod error;
mod event;
mod event_sink;
mod id;
mod job;
mod limits;
mod mutex_group;
mod progress;
mod pump;
mod quota_settings;
mod shutdown;
mod spec;
mod system;
#[cfg(test)]
mod test_support;
mod ticket;

pub use admission::{EditorJobAdmission, EditorJobAdmissionKey, EditorJobAdmissionRequest};
pub use cancellation_token::CancellationToken;
pub use category::{JobCategory, JobPriority};
pub use context::JobContext;
pub use error::{JobAdmissionKeyError, JobError, JobFailure, JobSubmitError, MutexGroupError};
pub use event::{JobEvent, JobEventKind};
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
pub use shutdown::UnfinishedEditorJob;
pub use spec::EditorJobSpec;
pub use system::{EditorJobAdmissionWindow, EditorJobBatchAdmissionReservation, EditorJobSystem};
pub use ticket::JobTicket;

#[cfg(test)]
pub(crate) use test_support::{
    test_job_scheduler, test_job_system, test_job_system_with_bus, test_job_system_with_limits,
};

#[cfg(test)]
mod tests;
