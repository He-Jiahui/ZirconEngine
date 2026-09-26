use super::{TaskDiagnosticCursor, TaskDiagnosticObservation};

/// 一页任务终结诊断及游标恢复信息；编辑器日志桥接器据此报告保留窗口缺口。
/// 遇到 source_changed 或 dropped_count 时先处理缺口，再用 next_cursor 继续翻页。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskDiagnosticBatch {
    observations: Vec<TaskDiagnosticObservation>,
    recovery_cursor: TaskDiagnosticCursor,
    next_cursor: TaskDiagnosticCursor,
    dropped_count: u64,
    source_changed: bool,
    has_more: bool,
}

impl TaskDiagnosticBatch {
    pub(super) fn new(
        observations: Vec<TaskDiagnosticObservation>,
        recovery_cursor: TaskDiagnosticCursor,
        next_cursor: TaskDiagnosticCursor,
        dropped_count: u64,
        source_changed: bool,
        has_more: bool,
    ) -> Self {
        Self {
            observations,
            recovery_cursor,
            next_cursor,
            dropped_count,
            source_changed,
            has_more,
        }
    }

    pub fn observations(&self) -> &[TaskDiagnosticObservation] {
        &self.observations
    }

    pub const fn recovery_cursor(&self) -> TaskDiagnosticCursor {
        self.recovery_cursor
    }

    pub const fn next_cursor(&self) -> TaskDiagnosticCursor {
        self.next_cursor
    }

    pub const fn dropped_count(&self) -> u64 {
        self.dropped_count
    }

    pub const fn source_changed(&self) -> bool {
        self.source_changed
    }

    pub const fn has_more(&self) -> bool {
        self.has_more
    }
}
