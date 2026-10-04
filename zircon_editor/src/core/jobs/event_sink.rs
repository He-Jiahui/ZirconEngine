//! 串行化每张任务的事件生命周期，使逃逸的上下文不能在终态后复活进度；同一事件先更新权威进度，再进入有界日志等待主循环投递。
use std::sync::{Arc, Mutex, MutexGuard};

use super::event_journal::EditorJobEventJournal;
use super::{EditorJobProgressSource, JobCategory, JobEvent, JobEventKind, JobId};

#[derive(Clone, Debug)]
pub(super) struct JobEventSink {
    id: JobId,
    label: Arc<str>,
    category: JobCategory,
    queue: EditorJobEventJournal,
    progress: EditorJobProgressSource,
    lifecycle: Arc<Mutex<JobEventLifecycle>>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum JobEventLifecycle {
    #[default]
    Pending,
    Running,
    Terminal,
}

impl JobEventSink {
    pub(super) fn new(
        id: JobId,
        label: Arc<str>,
        category: JobCategory,
        queue: EditorJobEventJournal,
        progress: EditorJobProgressSource,
    ) -> Self {
        Self {
            id,
            label,
            category,
            queue,
            progress,
            lifecycle: Arc::new(Mutex::new(JobEventLifecycle::Pending)),
        }
    }

    pub(super) fn emit(&self, kind: JobEventKind) {
        let mut lifecycle = self.lock_lifecycle();
        let next = match (&*lifecycle, &kind) {
            (JobEventLifecycle::Pending, JobEventKind::Started) => JobEventLifecycle::Running,
            (JobEventLifecycle::Running, JobEventKind::Progress { .. }) => {
                JobEventLifecycle::Running
            }
            (
                JobEventLifecycle::Pending | JobEventLifecycle::Running,
                JobEventKind::Completed | JobEventKind::Failed { .. } | JobEventKind::Cancelled,
            ) => JobEventLifecycle::Terminal,
            _ => return,
        };
        self.progress.apply_event(self.id, &kind);
        self.queue.push(JobEvent::new(
            self.id,
            Arc::clone(&self.label),
            self.category,
            kind,
        ));
        *lifecycle = next;
    }

    fn lock_lifecycle(&self) -> MutexGuard<'_, JobEventLifecycle> {
        self.lifecycle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
#[path = "tests/event_sink.rs"]
mod tests;
