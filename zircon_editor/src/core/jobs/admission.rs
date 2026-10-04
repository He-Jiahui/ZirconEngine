//! 区分带语义键的接收和合并结果，并允许调用端在物化工作资源前先申请轻量容量；合并不会创建第二张票据，最终任务规格必须兑现预留字段。
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
#[path = "tests/admission.rs"]
mod tests;

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
