use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{JobCategory, JobId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobEvent {
    #[serde(default)]
    journal_sequence: u64,
    id: JobId,
    label: Arc<str>,
    category: JobCategory,
    kind: JobEventKind,
}

impl JobEvent {
    pub(super) fn new(
        id: JobId,
        label: Arc<str>,
        category: JobCategory,
        kind: JobEventKind,
    ) -> Self {
        Self {
            journal_sequence: 0,
            id,
            label,
            category,
            kind,
        }
    }

    pub fn journal_sequence(&self) -> u64 {
        self.journal_sequence
    }

    pub fn id(&self) -> JobId {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn category(&self) -> JobCategory {
        self.category
    }

    pub fn kind(&self) -> &JobEventKind {
        &self.kind
    }

    pub(super) fn with_journal_sequence(mut self, journal_sequence: u64) -> Self {
        self.journal_sequence = journal_sequence;
        self
    }

    pub(super) fn estimated_retained_bytes(&self) -> usize {
        let detail_bytes = match &self.kind {
            JobEventKind::Progress { message, .. } | JobEventKind::Failed { message } => {
                message.len()
            }
            JobEventKind::Started | JobEventKind::Completed | JobEventKind::Cancelled => 0,
        };
        std::mem::size_of::<Self>()
            .saturating_add(self.label.len())
            .saturating_add(detail_bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{JobEvent, JobEventKind};
    use crate::core::jobs::{JobCategory, JobId};

    #[test]
    fn cloned_events_share_the_job_stable_label_allocation() {
        let event = JobEvent::new(
            JobId::new(7),
            Arc::<str>::from("thumbnail-stable-label"),
            JobCategory::Thumbnail,
            JobEventKind::Started,
        );

        let cloned = event.clone();

        assert_eq!(event.label(), "thumbnail-stable-label");
        assert_eq!(cloned.label(), event.label());
        assert!(Arc::ptr_eq(&event.label, &cloned.label));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobEventKind {
    Started,
    Progress {
        completed: u32,
        total: u32,
        message: String,
    },
    Completed,
    Failed {
        message: String,
    },
    Cancelled,
}
