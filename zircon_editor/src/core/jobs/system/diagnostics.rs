//! 把任务系统已有事件日志的压力快照交给宿主诊断，避免诊断端直接接触队列或干预消息泵生命周期。
use super::EditorJobSystem;

impl EditorJobSystem {
    pub fn event_journal_snapshot(&self) -> super::super::EditorJobEventJournalSnapshot {
        self.inner.event_queue.snapshot()
    }
}
