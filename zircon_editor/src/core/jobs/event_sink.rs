use std::sync::{Arc, Mutex, MutexGuard};

use super::pump::JobEventQueue;
use super::{EditorJobProgressSource, JobCategory, JobEvent, JobEventKind, JobId};

#[derive(Clone, Debug)]
pub(super) struct JobEventSink {
    id: JobId,
    label: Arc<str>,
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
        label: Arc<str>,
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
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn sink_reuses_the_spec_stable_label_allocation() {
        let spec = super::super::EditorJobSpec::new("stable-job-label", JobCategory::Index);
        let sink = JobEventSink::new(
            JobId::new(1),
            Arc::clone(&spec.label),
            JobCategory::Index,
            JobEventQueue::default(),
            EditorJobProgressSource::default(),
        );

        assert!(Arc::ptr_eq(&spec.label, &sink.label));
    }
}
