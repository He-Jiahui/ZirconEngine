use super::{JobAdmissionKeyError, JobId, JobTicket};

/// Stable semantic identity for requests that may share one pending job.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorJobAdmissionKey(String);

impl EditorJobAdmissionKey {
    pub fn new(value: impl Into<String>) -> Result<Self, JobAdmissionKeyError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(JobAdmissionKeyError::Empty);
        }
        Ok(Self(value))
    }
}

/// Explicit result of a keyed submission.
///
/// A merged request owns no second ticket or queue reservation. The accepted
/// job keeps its ticket while the pending task is updated with the latest
/// lightweight payload.
#[derive(Debug)]
pub enum EditorJobAdmission<T> {
    Accepted(JobTicket<T>),
    Merged { existing_job: JobId },
}
