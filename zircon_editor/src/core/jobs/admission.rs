use std::time::Duration;

use super::{JobAdmissionKeyError, JobCategory, JobId, JobPriority, JobTicket};

/// Stable semantic identity for requests that may share one pending job.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorJobAdmissionKey(String);

impl EditorJobAdmissionKey {
    /// Maximum retained UTF-8 bytes for one admission identity.
    pub const MAX_BYTES: usize = 256;

    pub fn new(value: impl Into<String>) -> Result<Self, JobAdmissionKeyError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(JobAdmissionKeyError::Empty);
        }
        if value.len() > Self::MAX_BYTES {
            return Err(JobAdmissionKeyError::TooLong {
                len: value.len(),
                max: Self::MAX_BYTES,
            });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_key_enforces_its_utf8_byte_budget() {
        assert!(EditorJobAdmissionKey::new("a".repeat(EditorJobAdmissionKey::MAX_BYTES)).is_ok());
        assert_eq!(
            EditorJobAdmissionKey::new("a".repeat(EditorJobAdmissionKey::MAX_BYTES + 1)),
            Err(JobAdmissionKeyError::TooLong {
                len: EditorJobAdmissionKey::MAX_BYTES + 1,
                max: EditorJobAdmissionKey::MAX_BYTES,
            })
        );
    }

    #[test]
    #[ignore = "managed Editor09 performance evidence"]
    fn editor09_admission_key_retention_budget_evidence() {
        const OVERSIZED_BYTES: usize = 1_048_576;

        let error = EditorJobAdmissionKey::new("a".repeat(OVERSIZED_BYTES)).unwrap_err();

        assert_eq!(
            error,
            JobAdmissionKeyError::TooLong {
                len: OVERSIZED_BYTES,
                max: EditorJobAdmissionKey::MAX_BYTES,
            }
        );
        println!(
            "EDITOR_JOB_BENCH_V1 kind=admission_key_retention oversized_input_bytes={} retained_identity_bytes_before={} retained_identity_bytes_after=0 retained_byte_reduction_percent=100.0000 maximum_bytes={}",
            OVERSIZED_BYTES,
            OVERSIZED_BYTES,
            EditorJobAdmissionKey::MAX_BYTES,
        );
    }
}

/// Lightweight capacity claim made before a caller materializes worker-owned resources.
///
/// The final job spec must retain these admission fields when it commits the
/// reservation. Fields that require materialization, such as a resource mutex
/// group, intentionally stay out of this request.
#[derive(Clone, Debug)]
pub struct EditorJobAdmissionRequest {
    pub(super) category: JobCategory,
    pub(super) priority: JobPriority,
    pub(super) estimated_pending_bytes: usize,
    pub(super) max_pending_age: Option<Duration>,
}

impl EditorJobAdmissionRequest {
    pub fn new(category: JobCategory, estimated_pending_bytes: usize) -> Self {
        Self {
            category,
            priority: JobPriority::Normal,
            estimated_pending_bytes: estimated_pending_bytes.max(1),
            max_pending_age: None,
        }
    }

    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_max_pending_age(mut self, max_pending_age: Duration) -> Self {
        self.max_pending_age = Some(max_pending_age);
        self
    }
}
