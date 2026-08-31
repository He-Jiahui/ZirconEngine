use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use super::{
    TaskDiagnosticBatch, TaskDiagnosticCursor, TaskDiagnosticJournal,
    TASK_DIAGNOSTIC_MAX_BATCH_ENTRIES,
};

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
