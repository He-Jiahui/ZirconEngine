use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use super::{
    CancellationToken, EditorJobSpec, JobCategory, JobEventKind, JobId, UnfinishedEditorJob,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorJobProgress {
    completed: u32,
    total: u32,
    message: String,
}

impl EditorJobProgress {
    pub fn new(completed: u32, total: u32, message: impl Into<String>) -> Self {
        Self {
            completed,
            total,
            message: message.into(),
        }
    }

    pub fn completed(&self) -> u32 {
        self.completed
    }

    pub fn total(&self) -> u32 {
        self.total
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorJobProgressSnapshot {
    id: JobId,
    label: String,
    category: JobCategory,
    progress: Option<EditorJobProgress>,
    cancellable: bool,
}

impl EditorJobProgressSnapshot {
    pub fn new(
        id: JobId,
        label: impl Into<String>,
        category: JobCategory,
        progress: Option<EditorJobProgress>,
        cancellable: bool,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            category,
            progress,
            cancellable,
        }
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

    pub fn progress(&self) -> Option<&EditorJobProgress> {
        self.progress.as_ref()
    }

    pub fn cancellable(&self) -> bool {
        self.cancellable
    }
}

#[derive(Clone, Debug, Default)]
pub struct EditorJobProgressSource {
    active: Arc<Mutex<BTreeMap<JobId, ActiveJobEntry>>>,
}

#[derive(Debug)]
struct ActiveJobEntry {
    snapshot: EditorJobProgressSnapshot,
    cancel: CancellationToken,
    terminal: bool,
}

impl EditorJobProgressSource {
    pub fn primary_snapshot(&self) -> Option<EditorJobProgressSnapshot> {
        self.lock_active()
            .values()
            .find(|entry| !entry.terminal)
            .map(|entry| entry.snapshot.clone())
    }

    pub fn snapshot(&self) -> Vec<EditorJobProgressSnapshot> {
        self.lock_active()
            .values()
            .filter(|entry| !entry.terminal)
            .map(|entry| entry.snapshot.clone())
            .collect()
    }

    pub(super) fn register(&self, id: JobId, spec: &EditorJobSpec) {
        self.lock_active().insert(
            id,
            ActiveJobEntry {
                snapshot: EditorJobProgressSnapshot::new(
                    id,
                    spec.label.clone(),
                    spec.category,
                    None,
                    true,
                ),
                cancel: spec.cancel.clone(),
                terminal: false,
            },
        );
    }

    pub(super) fn request_cancel(&self, id: JobId) -> bool {
        let active = self.lock_active();
        let Some(entry) = active.get(&id) else {
            return false;
        };
        if entry.terminal {
            return false;
        }
        entry.cancel.cancel();
        true
    }

    pub(super) fn cancel_all(&self) {
        for entry in self.lock_active().values() {
            if !entry.terminal {
                entry.cancel.cancel();
            }
        }
    }

    pub(super) fn has_active(&self) -> bool {
        !self.lock_active().is_empty()
    }

    pub(super) fn unfinished_jobs(&self) -> Vec<UnfinishedEditorJob> {
        self.lock_active()
            .values()
            .map(|entry| {
                UnfinishedEditorJob::new(
                    entry.snapshot.id,
                    entry.snapshot.label.clone(),
                    entry.snapshot.category,
                )
            })
            .collect()
    }

    pub(super) fn apply_event(&self, id: JobId, kind: &JobEventKind) {
        let mut active = self.lock_active();
        match kind {
            JobEventKind::Progress {
                completed,
                total,
                message,
            } => {
                if let Some(entry) = active.get_mut(&id) {
                    if !entry.terminal {
                        entry.snapshot.progress =
                            Some(EditorJobProgress::new(*completed, *total, message.clone()));
                    }
                }
            }
            JobEventKind::Completed | JobEventKind::Failed { .. } | JobEventKind::Cancelled => {
                if let Some(entry) = active.get_mut(&id) {
                    entry.terminal = true;
                }
            }
            JobEventKind::Started => {}
        }
    }

    pub(super) fn complete(&self, id: JobId) {
        self.lock_active().remove(&id);
    }

    fn lock_active(&self) -> MutexGuard<'_, BTreeMap<JobId, ActiveJobEntry>> {
        self.active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{EditorJobSpec, JobCategory, JobEventKind, JobId};
    use super::EditorJobProgressSource;

    #[test]
    fn terminal_visibility_hides_ui_before_lifecycle_completion_removes_the_entry() {
        let progress = EditorJobProgressSource::default();
        let id = JobId::new(7);
        progress.register(id, &EditorJobSpec::new("terminal", JobCategory::Compile));

        progress.apply_event(id, &JobEventKind::Completed);

        assert!(progress.snapshot().is_empty());
        assert!(progress.has_active());
        assert_eq!(progress.unfinished_jobs()[0].id(), id);
        assert!(!progress.request_cancel(id));

        progress.complete(id);
        assert!(!progress.has_active());
        assert!(progress.unfinished_jobs().is_empty());
    }

    #[test]
    fn primary_snapshot_clones_only_the_smallest_visible_job() {
        let progress = EditorJobProgressSource::default();
        let first = JobId::new(2);
        let hidden = JobId::new(1);
        let later = JobId::new(9);
        progress.register(later, &EditorJobSpec::new("later", JobCategory::Thumbnail));
        progress.register(first, &EditorJobSpec::new("first", JobCategory::Compile));
        progress.register(hidden, &EditorJobSpec::new("hidden", JobCategory::Import));
        progress.apply_event(hidden, &JobEventKind::Completed);

        let primary = progress.primary_snapshot().unwrap();

        assert_eq!(primary.id(), first);
        assert_eq!(primary.label(), "first");
        assert_eq!(progress.snapshot().len(), 2);
    }
}
