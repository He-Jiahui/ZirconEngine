//! 工作端通过上下文观察合作式取消并报告进度，事件由任务生命周期入口进入日志与权威进度源；调用端应选择可安全取消的边界。
use super::event_sink::JobEventSink;
use super::{CancellationToken, JobError, JobEventKind};

#[derive(Clone, Debug)]
pub struct JobContext {
    cancel: CancellationToken,
    events: JobEventSink,
}

impl JobContext {
    pub(super) fn new(cancel: CancellationToken, events: JobEventSink) -> Self {
        Self { cancel, events }
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancel
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    /// 仅在允许放弃结果或尚未提交副作用的边界使用；此检查无法回滚已完成的外部事务。
    pub fn check_cancelled(&self) -> Result<(), JobError> {
        if self.is_cancelled() {
            Err(JobError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub fn report_progress(&self, completed: u32, total: u32, message: impl Into<String>) {
        self.events.emit(JobEventKind::Progress {
            completed,
            total,
            message: message.into(),
        });
    }

    pub(super) fn emit(&self, kind: JobEventKind) {
        self.events.emit(kind);
    }
}
