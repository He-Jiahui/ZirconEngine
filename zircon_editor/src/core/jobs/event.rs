use serde::{Deserialize, Serialize};

use super::{JobCategory, JobId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobEvent {
    id: JobId,
    label: String,
    category: JobCategory,
    kind: JobEventKind,
}

impl JobEvent {
    pub(super) fn new(id: JobId, label: String, category: JobCategory, kind: JobEventKind) -> Self {
        Self {
            id,
            label,
            category,
            kind,
        }
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
