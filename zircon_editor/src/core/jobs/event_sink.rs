use std::sync::mpsc::Sender;

use super::{EditorJobProgressSource, JobCategory, JobEvent, JobEventKind, JobId};

#[derive(Clone, Debug)]
pub(super) struct JobEventSink {
    id: JobId,
    label: String,
    category: JobCategory,
    sender: Sender<JobEvent>,
    progress: EditorJobProgressSource,
}

impl JobEventSink {
    pub(super) fn new(
        id: JobId,
        label: String,
        category: JobCategory,
        sender: Sender<JobEvent>,
        progress: EditorJobProgressSource,
    ) -> Self {
        Self {
            id,
            label,
            category,
            sender,
            progress,
        }
    }

    pub(super) fn emit(&self, kind: JobEventKind) {
        self.progress.apply_event(self.id, &kind);
        let _ = self.sender.send(JobEvent::new(
            self.id,
            self.label.clone(),
            self.category,
            kind,
        ));
    }
}
