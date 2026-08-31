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
        self.lock_state()
            .active
            .values()
            .filter(|entry| !entry.terminal)
            .map(|entry| entry.snapshot.clone())
            .collect()
    }

    /// Returns at most `limit` visible snapshots in stable job-id order.
    ///
    /// Lifecycle consumers use this only to refill a bounded presentation
    /// capacity after an entry retires; frame consumers should use
    /// [`Self::snapshot_for_ids`] instead.
    pub fn snapshot_limit(&self, limit: usize) -> Vec<EditorJobProgressSnapshot> {
        self.lock_state()
            .active
            .values()
            .filter(|entry| !entry.terminal)
            .take(limit)
            .map(|entry| entry.snapshot.clone())
            .collect()
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
        ids.into_iter()
            .filter_map(|id| {
                let entry = active.get(&id)?;
                (!entry.terminal).then(|| entry.snapshot.clone())
            })
            .collect()
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
        state.visible_by_priority.insert(
            state
                .active
                .get(&id)
                .expect("registered progress entry must remain active")
                .primary_key(),
        );
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
        self.lock_state()
            .active
            .values()
            .map(|entry| {
                UnfinishedEditorJob::new(
                    entry.snapshot.id,
                    entry.snapshot.label.to_string(),
                    entry.snapshot.category,
                )
            })
            .collect()
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
#[path = "progress/primary_generation_tests.rs"]
mod primary_generation_tests;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::super::{EditorJobSpec, JobCategory, JobEventKind, JobId, JobPriority};
    use super::{EditorJobProgressSnapshot, EditorJobProgressSource};

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

    #[test]
    fn selected_snapshots_exclude_unrequested_and_terminal_jobs() {
        let progress = EditorJobProgressSource::default();
        let first = JobId::new(2);
        let later = JobId::new(9);
        let unrequested = JobId::new(3);
        let terminal = JobId::new(4);
        progress.register(first, &EditorJobSpec::new("first", JobCategory::Import));
        progress.register(later, &EditorJobSpec::new("later", JobCategory::Thumbnail));
        progress.register(
            unrequested,
            &EditorJobSpec::new("unrequested", JobCategory::Compile),
        );
        progress.register(
            terminal,
            &EditorJobSpec::new("terminal", JobCategory::Index),
        );
        progress.apply_event(terminal, &JobEventKind::Completed);

        let snapshots = progress.snapshot_for_ids([later, terminal, first, later]);

        assert_eq!(
            snapshots
                .iter()
                .map(EditorJobProgressSnapshot::id)
                .collect::<Vec<_>>(),
            vec![first, later]
        );
    }

    #[test]
    fn selected_snapshot_lookup_uses_requested_ids_without_active_scan() {
        let source = include_str!("progress.rs");
        let method_start = source
            .find("pub fn snapshot_for_ids(")
            .expect("snapshot_for_ids should remain available");
        let method_end = method_start
            + source[method_start..]
                .find("\n    pub(super) fn register")
                .expect("snapshot_for_ids should end before registration");
        let method = &source[method_start..method_end];

        assert!(method.contains("ids.into_iter()"));
        assert!(method.contains("active.get(&id)"));
        for active_scan in [
            "active.iter()",
            "active.values()",
            "active.keys()",
            "active.range(",
            "active.into_iter()",
        ] {
            assert!(
                !method.contains(active_scan),
                "selected snapshot lookup must not scan active entries through {active_scan}"
            );
        }
    }

    #[test]
    fn limited_snapshots_preserve_stable_order_without_cloning_the_tail() {
        let progress = EditorJobProgressSource::default();
        for id in [JobId::new(4), JobId::new(1), JobId::new(7)] {
            progress.register(id, &EditorJobSpec::new("limited", JobCategory::Import));
        }

        let snapshots = progress.snapshot_limit(2);

        assert_eq!(
            snapshots
                .iter()
                .map(EditorJobProgressSnapshot::id)
                .collect::<Vec<_>>(),
            vec![JobId::new(1), JobId::new(4)]
        );
    }

    #[test]
    fn repeated_snapshots_share_label_and_progress_message_allocations() {
        let progress = EditorJobProgressSource::default();
        let id = JobId::new(11);
        progress.register(id, &EditorJobSpec::new("shared-label", JobCategory::Import));
        progress.apply_event(
            id,
            &JobEventKind::Progress {
                completed: 1,
                total: 2,
                message: "shared-progress-message".to_string(),
            },
        );

        let first = progress.snapshot().pop().unwrap();
        let second = progress.snapshot().pop().unwrap();

        assert!(Arc::ptr_eq(&first.label, &second.label));
        assert!(Arc::ptr_eq(
            &first.progress.as_ref().unwrap().message,
            &second.progress.as_ref().unwrap().message,
        ));
    }

    #[test]
    fn interactive_progress_preempts_an_older_background_job() {
        let progress = EditorJobProgressSource::default();
        let background = JobId::new(1);
        let interactive = JobId::new(99);
        progress.register(
            background,
            &EditorJobSpec::new("background", JobCategory::Index)
                .with_priority(JobPriority::Background),
        );
        let observed = progress
            .primary_snapshot_if_changed(None)
            .unwrap()
            .generation();
        progress.register(
            interactive,
            &EditorJobSpec::new("interactive", JobCategory::InteractiveSave)
                .with_priority(JobPriority::Interactive),
        );

        let preempted = progress
            .primary_snapshot_if_changed(Some(observed))
            .expect("interactive registration must advance the retained primary");
        assert_eq!(preempted.primary().unwrap().id(), interactive);

        progress.apply_event(interactive, &JobEventKind::Completed);

        let resumed = progress
            .primary_snapshot_if_changed(Some(preempted.generation()))
            .expect("terminal interactive work must restore the background primary");
        assert_eq!(resumed.primary().unwrap().id(), background);
    }

    #[test]
    fn primary_lookup_uses_the_maintained_visibility_index() {
        let source = include_str!("progress.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("progress implementation");
        let primary_entry = implementation
            .split("fn primary_entry(&self)")
            .nth(1)
            .and_then(|source| source.split("fn primary_id(&self)").next())
            .expect("primary entry implementation");

        assert!(implementation.contains("visible_by_priority: BTreeSet<(u8, JobId)>"));
        assert!(primary_entry.contains("visible_by_priority.first()"));
        assert!(!primary_entry.contains("active.values()"));
        assert!(!primary_entry.contains("min_by_key"));
    }

    #[test]
    #[ignore = "managed Editor09 performance evidence"]
    fn editor09_progress_snapshot_shared_string_evidence() {
        const ACTIVE_JOBS: usize = 10_000;
        const REPEATS: usize = 8;
        const MAX_NON_PRIMARY_UPDATE_LATENCY: Duration = Duration::from_secs(2);

        let progress = EditorJobProgressSource::default();
        for index in 1..=ACTIVE_JOBS {
            let id = JobId::new(index as u64);
            progress.register(
                id,
                &EditorJobSpec::new(
                    format!("background-progress-job-{index:05}"),
                    JobCategory::Import,
                ),
            );
            progress.apply_event(
                id,
                &JobEventKind::Progress {
                    completed: 1,
                    total: 100,
                    message: format!("processing background artifact {index:05}"),
                },
            );
        }

        let update_started = Instant::now();
        for index in 2..=ACTIVE_JOBS {
            progress.apply_event(
                JobId::new(index as u64),
                &JobEventKind::Progress {
                    completed: 2,
                    total: 100,
                    message: format!("updated background artifact {index:05}"),
                },
            );
        }
        let update_elapsed = update_started.elapsed();
        assert!(update_elapsed <= MAX_NON_PRIMARY_UPDATE_LATENCY);
        let primary_candidates_scanned_before = (ACTIVE_JOBS - 1).saturating_mul(ACTIVE_JOBS);
        let primary_index_reads_after = ACTIVE_JOBS - 1;
        let primary_lookup_reduction_percent = (1.0
            - primary_index_reads_after as f64 / primary_candidates_scanned_before as f64)
            * 100.0;
        let baseline = progress.snapshot();
        let copied_string_bytes_before_per_snapshot = baseline
            .iter()
            .map(|snapshot| {
                snapshot.label().len()
                    + snapshot
                        .progress()
                        .map(|progress| progress.message().len())
                        .unwrap_or_default()
            })
            .sum::<usize>();
        let mut samples = Vec::with_capacity(REPEATS);
        for _ in 0..REPEATS {
            let started = Instant::now();
            let snapshot = progress.snapshot();
            samples.push(started.elapsed().as_nanos());
            assert_eq!(snapshot.len(), ACTIVE_JOBS);
            assert!(snapshot
                .iter()
                .zip(&baseline)
                .all(|(next, previous)| Arc::ptr_eq(&next.label, &previous.label)
                    && Arc::ptr_eq(
                        &next.progress.as_ref().unwrap().message,
                        &previous.progress.as_ref().unwrap().message,
                    )));
        }
        samples.sort_unstable();

        println!(
            "EDITOR_JOB_BENCH_V1 kind=progress_snapshot_shared_strings active_jobs={} repeats={} copied_string_bytes_before={} copied_string_bytes_after=0 copied_string_reduction_percent=100.0000 non_primary_updates={} primary_candidates_scanned_before={} primary_index_reads_after={} primary_lookup_reduction_percent={:.4} update_elapsed_ns={} update_target_ns={} p50_ns={} p95_ns={} max_ns={}",
            ACTIVE_JOBS,
            REPEATS,
            copied_string_bytes_before_per_snapshot.saturating_mul(REPEATS),
            ACTIVE_JOBS - 1,
            primary_candidates_scanned_before,
            primary_index_reads_after,
            primary_lookup_reduction_percent,
            update_elapsed.as_nanos(),
            MAX_NON_PRIMARY_UPDATE_LATENCY.as_nanos(),
            samples[REPEATS / 2],
            samples[REPEATS - 1],
            samples[REPEATS - 1],
        );
    }
}
