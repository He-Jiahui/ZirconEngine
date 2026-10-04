use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::{
    CancellationToken, EditorJobSpec, JobCategory, JobEventKind, JobId, JobPriority,
    UnfinishedEditorJob,
};

/// Observes the authoritative job lifecycle without retaining ticket receivers.
///
/// Consumers may derive bounded read models from a `JobId`, but the progress
/// source remains the sole owner of active job state.
pub trait EditorJobProgressObserver: Send + Sync {
    fn job_admitted(&self, job: JobId, source: &EditorJobProgressSource);
    fn job_finished(&self, job: JobId, source: &EditorJobProgressSource);
    fn jobs_resynchronized(&self, source: &EditorJobProgressSource);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorJobProgress {
    completed: u32,
    total: u32,
    message: Arc<str>,
}

impl EditorJobProgress {
    pub fn new(completed: u32, total: u32, message: impl Into<String>) -> Self {
        Self {
            completed,
            total,
            message: Arc::from(message.into()),
        }
    }

    fn from_borrowed(completed: u32, total: u32, message: &str) -> Self {
        Self {
            completed,
            total,
            message: Arc::from(message),
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
    label: Arc<str>,
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
            label: Arc::from(label.into()),
            category,
            progress,
            cancellable,
        }
    }

    fn with_shared_label(
        id: JobId,
        label: Arc<str>,
        category: JobCategory,
        progress: Option<EditorJobProgress>,
        cancellable: bool,
    ) -> Self {
        Self {
            id,
            label,
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

/// An atomically observed primary-progress generation and optional snapshot.
///
/// A retained consumer keeps the last observed generation. Equal generations
/// deliberately return no snapshot, so stable frames do not clone job labels
/// or progress messages before they can bypass presentation work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorJobPrimaryProgressSnapshot {
    generation: u64,
    primary: Option<EditorJobProgressSnapshot>,
}

impl EditorJobPrimaryProgressSnapshot {
    fn new(generation: u64, primary: Option<EditorJobProgressSnapshot>) -> Self {
        Self {
            generation,
            primary,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn primary(&self) -> Option<&EditorJobProgressSnapshot> {
        self.primary.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct EditorJobProgressSource {
    state: Arc<Mutex<ProgressState>>,
    published_primary_generation: Arc<AtomicU64>,
}

impl Default for EditorJobProgressSource {
    fn default() -> Self {
        Self {
            state: Arc::default(),
            published_primary_generation: Arc::new(AtomicU64::new(0)),
        }
    }
}

/// The authoritative job map and its retained primary projection share one
/// lock so a returned generation can never describe a different snapshot.
#[derive(Debug, Default)]
struct ProgressState {
    active: BTreeMap<JobId, ActiveJobEntry>,
    visible_by_priority: BTreeSet<(u8, JobId)>,
    primary_generation: u64,
}

#[derive(Debug)]
struct ActiveJobEntry {
    snapshot: EditorJobProgressSnapshot,
    cancel: CancellationToken,
    priority: JobPriority,
    terminal: bool,
}

impl EditorJobProgressSource {
    pub fn primary_snapshot(&self) -> Option<EditorJobProgressSnapshot> {
        self.lock_state().primary_snapshot()
    }

    /// Returns a primary snapshot only when it differs from the retained
    /// consumer's observed generation.
    ///
    /// Passing `None` performs the initial observation, including the empty
    /// primary state. Passing the same generation clones nothing.
    pub fn primary_snapshot_if_changed(
        &self,
        observed_generation: Option<u64>,
    ) -> Option<EditorJobPrimaryProgressSnapshot> {
        if observed_generation.is_some_and(|observed| {
            observed == self.published_primary_generation.load(Ordering::Acquire)
        }) {
            return None;
        }

        let state = self.lock_state();
        if observed_generation == Some(state.primary_generation) {
            return None;
        }

        Some(EditorJobPrimaryProgressSnapshot::new(
            state.primary_generation,
            state.primary_snapshot(),
        ))
    }

    pub fn snapshot(&self) -> Vec<EditorJobProgressSnapshot> {
        let state = self.lock_state();
        let mut snapshots = Vec::with_capacity(state.active.len());
        snapshots.extend(
            state
                .active
                .values()
                .filter(|entry| !entry.terminal)
                .map(|entry| entry.snapshot.clone()),
        );
        snapshots
    }

    /// Returns at most `limit` visible snapshots in stable job-id order.
    ///
    /// Lifecycle consumers use this only to refill a bounded presentation
    /// capacity after an entry retires; frame consumers should use
    /// [`Self::snapshot_for_ids`] instead.
    pub fn snapshot_limit(&self, limit: usize) -> Vec<EditorJobProgressSnapshot> {
        let state = self.lock_state();
        let mut snapshots = Vec::with_capacity(limit.min(state.active.len()));
        snapshots.extend(
            state
                .active
                .values()
                .filter(|entry| !entry.terminal)
                .take(limit)
                .map(|entry| entry.snapshot.clone()),
        );
        snapshots
    }

    /// Returns only the visible snapshots explicitly requested by a consumer.
    ///
    /// Activity notification projection uses this instead of scanning every active
    /// job on each frame when it tracks only a bounded set of notification ids.
    pub fn snapshot_for_ids(
        &self,
        ids: impl IntoIterator<Item = JobId>,
    ) -> Vec<EditorJobProgressSnapshot> {
        let ids = ids.into_iter().collect::<BTreeSet<_>>();
        if ids.is_empty() {
            return Vec::new();
        }
        let state = self.lock_state();
        let active = &state.active;
        let mut snapshots = Vec::with_capacity(ids.len());
        snapshots.extend(ids.into_iter().filter_map(|id| {
            let entry = active.get(&id)?;
            (!entry.terminal).then(|| entry.snapshot.clone())
        }));
        snapshots
    }

    /// Returns visible snapshots for a bounded caller-owned set of unique job IDs.
    ///
    /// Unlike [`Self::snapshot_for_ids`], this avoids allocating a deduplication index because
    /// the caller maintains the one-notification-per-job invariant. It preserves caller order.
    pub(crate) fn snapshot_for_unique_ids(
        &self,
        ids: impl IntoIterator<Item = JobId>,
    ) -> Vec<EditorJobProgressSnapshot> {
        let ids = ids.into_iter();
        let (lower_bound, upper_bound) = ids.size_hint();
        let mut snapshots = Vec::with_capacity(lower_bound);
        if upper_bound == Some(0) {
            return snapshots;
        }
        let state = self.lock_state();
        let active = &state.active;
        for id in ids {
            let Some(entry) = active.get(&id) else {
                continue;
            };
            if !entry.terminal {
                snapshots.push(entry.snapshot.clone());
            }
        }
        snapshots
    }

    pub(super) fn register(&self, id: JobId, spec: &EditorJobSpec) {
        let mut state = self.lock_state();
        let previous_primary = state.primary_id();
        let next_entry = ActiveJobEntry {
            snapshot: EditorJobProgressSnapshot::with_shared_label(
                id,
                Arc::clone(&spec.label),
                spec.category,
                None,
                true,
            ),
            cancel: spec.cancel.clone(),
            priority: spec.priority,
            terminal: false,
        };
        let primary_projection_changes = match previous_primary {
            None => true,
            Some(primary) if id == primary => state.active.get(&id).is_some_and(|previous| {
                previous.terminal != next_entry.terminal
                    || previous.snapshot != next_entry.snapshot
                    || previous.priority != next_entry.priority
            }),
            Some(primary) => state
                .active
                .get(&primary)
                .is_some_and(|current| next_entry.primary_key() < current.primary_key()),
        };
        let next_generation = primary_projection_changes.then(|| state.next_primary_generation());

        if let Some(previous) = state.active.insert(id, next_entry) {
            if !previous.terminal {
                state.visible_by_priority.remove(&previous.primary_key());
            }
        }
        let primary_key = state
            .active
            .get(&id)
            .expect("registered progress entry must remain active")
            .primary_key();
        state.visible_by_priority.insert(primary_key);
        if let Some(next_generation) = next_generation {
            self.publish_primary_generation(&mut state, next_generation);
        }
    }

    pub(super) fn request_cancel(&self, id: JobId) -> bool {
        let state = self.lock_state();
        let Some(entry) = state.active.get(&id) else {
            return false;
        };
        if entry.terminal {
            return false;
        }
        entry.cancel.cancel();
        true
    }

    pub(super) fn cancel_all(&self) {
        for entry in self.lock_state().active.values() {
            if !entry.terminal {
                entry.cancel.cancel();
            }
        }
    }

    pub(super) fn has_active(&self) -> bool {
        !self.lock_state().active.is_empty()
    }

    pub(super) fn unfinished_jobs(&self) -> Vec<UnfinishedEditorJob> {
        let state = self.lock_state();
        let mut unfinished = Vec::with_capacity(state.active.len());
        unfinished.extend(state.active.values().map(|entry| {
            UnfinishedEditorJob::new(
                entry.snapshot.id,
                entry.snapshot.label.to_string(),
                entry.snapshot.category,
            )
        }));
        unfinished
    }

    pub(super) fn apply_event(&self, id: JobId, kind: &JobEventKind) {
        let mut state = self.lock_state();
        let previous_primary = state.primary_id();
        match kind {
            JobEventKind::Progress {
                completed,
                total,
                message,
            } => {
                let next = EditorJobProgress::from_borrowed(*completed, *total, message.as_str());
                let Some(entry) = state.active.get(&id) else {
                    return;
                };
                if entry.terminal || entry.snapshot.progress.as_ref() == Some(&next) {
                    return;
                }
                let next_generation =
                    (previous_primary == Some(id)).then(|| state.next_primary_generation());

                if let Some(entry) = state.active.get_mut(&id) {
                    entry.snapshot.progress = Some(next);
                }
                if let Some(next_generation) = next_generation {
                    self.publish_primary_generation(&mut state, next_generation);
                }
            }
            JobEventKind::Completed | JobEventKind::Failed { .. } | JobEventKind::Cancelled => {
                let Some(entry) = state.active.get(&id) else {
                    return;
                };
                if entry.terminal {
                    return;
                }
                let primary_key = entry.primary_key();
                let next_generation =
                    (previous_primary == Some(id)).then(|| state.next_primary_generation());

                if let Some(entry) = state.active.get_mut(&id) {
                    entry.terminal = true;
                }
                state.visible_by_priority.remove(&primary_key);
                if let Some(next_generation) = next_generation {
                    self.publish_primary_generation(&mut state, next_generation);
                }
            }
            JobEventKind::Started => {}
        }
    }

    pub(super) fn complete(&self, id: JobId) {
        let mut state = self.lock_state();
        let previous_primary = state.primary_id();
        let next_generation =
            (previous_primary == Some(id)).then(|| state.next_primary_generation());

        if let Some(entry) = state.active.remove(&id) {
            if !entry.terminal {
                state.visible_by_priority.remove(&entry.primary_key());
            }
        }
        if let Some(next_generation) = next_generation {
            self.publish_primary_generation(&mut state, next_generation);
        }
    }

    fn publish_primary_generation(&self, state: &mut ProgressState, next_generation: u64) {
        state.primary_generation = next_generation;
        self.published_primary_generation
            .store(next_generation, Ordering::Release);
    }

    fn lock_state(&self) -> MutexGuard<'_, ProgressState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl ProgressState {
    fn primary_entry(&self) -> Option<&ActiveJobEntry> {
        let (_, id) = self.visible_by_priority.first()?;
        self.active.get(id)
    }

    fn primary_id(&self) -> Option<JobId> {
        self.primary_entry().map(|entry| entry.snapshot.id())
    }

    fn primary_snapshot(&self) -> Option<EditorJobProgressSnapshot> {
        self.primary_entry().map(|entry| entry.snapshot.clone())
    }

    fn next_primary_generation(&self) -> u64 {
        self.primary_generation
            .checked_add(1)
            .expect("primary progress generation cannot overflow")
    }
}

impl ActiveJobEntry {
    fn primary_key(&self) -> (u8, JobId) {
        (self.priority.admission_rank(), self.snapshot.id())
    }
}

#[cfg(test)]
#[path = "progress/tests/primary_generation_tests.rs"]
mod primary_generation_tests;

#[cfg(test)]
#[path = "tests/progress.rs"]
mod tests;
