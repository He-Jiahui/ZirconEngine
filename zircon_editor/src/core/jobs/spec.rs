use super::{CancellationToken, JobCategory, JobId, JobPriority, MutexGroup};

#[derive(Clone, Debug)]
pub struct EditorJobSpec {
    pub(super) label: String,
    pub(super) category: JobCategory,
    pub(super) priority: JobPriority,
    pub(super) mutex_group: Option<MutexGroup>,
    pub(super) cancel: CancellationToken,
    pub(super) after: Vec<JobId>,
}

impl EditorJobSpec {
    pub fn new(label: impl Into<String>, category: JobCategory) -> Self {
        Self {
            label: label.into(),
            category,
            priority: JobPriority::Normal,
            mutex_group: None,
            cancel: CancellationToken::default(),
            after: Vec::new(),
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

    pub fn after(mut self, dependency: JobId) -> Self {
        if !self.after.contains(&dependency) {
            self.after.push(dependency);
        }
        self
    }
}
