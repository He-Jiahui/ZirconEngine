use std::sync::{Arc, Mutex, MutexGuard};

use super::pump::JobEventQueue;
use super::{EditorJobProgressSource, JobCategory, JobEvent, JobEventKind, JobId};

#[derive(Clone, Debug)]
pub(super) struct JobEventSink {
    id: JobId,
    label: String,
    category: JobCategory,
    queue: JobEventQueue,
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
        label: String,
        category: JobCategory,
        queue: JobEventQueue,
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
            self.label.clone(),
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
