//! 把截止时仍未完成的任务身份、分类及标签交给关闭边界，供宿主报告或继续处理；该诊断结果本身不是已停止工作的确认。
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
