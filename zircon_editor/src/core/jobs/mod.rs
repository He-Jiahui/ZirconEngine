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
mod shutdown;
mod spec;
mod system;
#[cfg(test)]
mod test_support;
mod ticket;

pub use cancellation_token::CancellationToken;
pub use category::{JobCategory, JobPriority};
pub use context::JobContext;
pub use error::{JobError, JobFailure, JobSubmitError, MutexGroupError};
pub use event::{JobEvent, JobEventKind};
pub use id::JobId;
pub use job::EditorJob;
pub use limits::EditorJobLimits;
pub use mutex_group::MutexGroup;
pub use progress::{EditorJobProgress, EditorJobProgressSnapshot, EditorJobProgressSource};
pub use shutdown::UnfinishedEditorJob;
pub use spec::EditorJobSpec;
pub use system::EditorJobSystem;
pub use ticket::JobTicket;

#[cfg(test)]
pub(crate) use test_support::{
    test_job_scheduler, test_job_system, test_job_system_with_bus, test_job_system_with_limits,
};

#[cfg(test)]
mod tests;
