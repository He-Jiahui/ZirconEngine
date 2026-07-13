use super::{JobCategory, JobId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnfinishedEditorJob {
    id: JobId,
    label: String,
    category: JobCategory,
}

impl UnfinishedEditorJob {
    pub(super) fn new(id: JobId, label: String, category: JobCategory) -> Self {
        Self {
            id,
            label,
            category,
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
}
