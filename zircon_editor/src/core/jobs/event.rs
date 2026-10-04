//! 提供从工作生命周期到主循环消息泵的稳定事件载荷；日志序号描述保留顺序，任务身份和共享标签贯穿进度及终态，保留字节估算服务日志预算。
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
#[path = "tests/event.rs"]
mod tests;

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
