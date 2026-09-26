use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use super::{
    TaskDiagnosticBatch, TaskDiagnosticCursor, TaskDiagnosticJournal,
    TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES,
};

/// 调度器对外提供的诊断读取句柄；可在不启用完整生命周期采样的情况下读取终结事件。
#[derive(Clone)]
pub struct TaskDiagnosticSource {
    journal: Arc<TaskDiagnosticJournal>,
}

impl TaskDiagnosticSource {
    pub(in crate::core::runtime::tasks) fn new(journal: Arc<TaskDiagnosticJournal>) -> Self {
        Self { journal }
    }

    pub fn initial_cursor(&self) -> TaskDiagnosticCursor {
        self.journal.initial_cursor()
    }

    /// 从游标取有界的一页；消费方须处理丢失和换源，再以返回的游标继续读取。
    pub fn read_after(
        &self,
        cursor: TaskDiagnosticCursor,
        max_entries: usize,
    ) -> TaskDiagnosticBatch {
        self.journal
            .read_after(cursor, max_entries.min(TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES))
    }
}

impl Debug for TaskDiagnosticSource {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TaskDiagnosticSource")
            .field("source_id", &self.journal.source_id())
            .finish_non_exhaustive()
    }
}
