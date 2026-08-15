use std::sync::Arc;
use std::time::Duration;

use super::{
    CancellationToken, EditorJobAdmissionKey, JobCategory, JobId, JobPriority, MutexGroup,
};

const DEFAULT_ESTIMATED_PENDING_BYTES: usize = 4 * 1024;

#[derive(Clone, Debug)]
pub struct EditorJobSpec {
    pub(super) label: Arc<str>,
    pub(super) category: JobCategory,
    pub(super) priority: JobPriority,
    pub(super) mutex_group: Option<MutexGroup>,
    pub(super) cancel: CancellationToken,
    pub(super) after: Vec<JobId>,
    pub(super) estimated_pending_bytes: usize,
    pub(super) admission_key: Option<EditorJobAdmissionKey>,
    pub(super) max_pending_age: Option<Duration>,
}

impl EditorJobSpec {
    pub fn new(label: impl Into<String>, category: JobCategory) -> Self {
        Self {
            label: Arc::from(label.into()),
            category,
            priority: JobPriority::Normal,
            mutex_group: None,
            cancel: CancellationToken::default(),
            after: Vec::new(),
            estimated_pending_bytes: DEFAULT_ESTIMATED_PENDING_BYTES,
            admission_key: None,
            max_pending_age: None,
        }
    }

    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_mutex_group(mut self, mutex_group: MutexGroup) -> Self {
        self.mutex_group = Some(mutex_group);
        self
    }

    pub fn with_cancel(mut self, cancel: CancellationToken) -> Self {
        self.cancel = cancel;
        self
    }

    pub fn with_estimated_bytes(mut self, estimated_pending_bytes: usize) -> Self {
        self.estimated_pending_bytes = estimated_pending_bytes.max(1);
        self
    }

    pub fn with_admission_key(mut self, admission_key: EditorJobAdmissionKey) -> Self {
        self.admission_key = Some(admission_key);
        self
    }

    pub fn with_max_pending_age(mut self, max_pending_age: Duration) -> Self {
        self.max_pending_age = Some(max_pending_age);
        self
    }

    pub fn after(mut self, dependency: JobId) -> Self {
        if !self.after.contains(&dependency) {
            self.after.push(dependency);
        }
        self
    }
}
