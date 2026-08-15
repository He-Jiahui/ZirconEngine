use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::core::jobs::{EditorJobSystem, UnfinishedEditorJob};

use super::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDocumentId, AutosaveDocumentRequest,
    AutosaveDocumentState, AutosaveJobAdapter, AutosavePolicy, AutosaveScheduler, AutosaveStore,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorAutosavePoll {
    now: Duration,
    due: bool,
    completion: AutosaveCompletion,
}

impl EditorAutosavePoll {
    pub(crate) const fn now(self) -> Duration {
        self.now
    }

    pub(crate) const fn is_due(self) -> bool {
        self.due
    }

    pub(crate) const fn completion(self) -> AutosaveCompletion {
        self.completion
    }
}

/// EditorContext-owned autosave lifecycle. It reuses the context's one job
/// system and owns at most one active-project adapter generation.
pub(crate) struct EditorAutosaveService {
    jobs: EditorJobSystem,
    policy: AutosavePolicy,
    state: Mutex<EditorAutosaveState>,
}

#[derive(Default)]
struct EditorAutosaveState {
    project_root: Option<PathBuf>,
    project_started_at: Option<Instant>,
    adapter: Option<AutosaveJobAdapter>,
}

impl EditorAutosaveService {
    pub(crate) fn new(jobs: EditorJobSystem, policy: AutosavePolicy) -> Self {
        Self {
            jobs,
            policy,
            state: Mutex::new(EditorAutosaveState::default()),
        }
    }

    /// Synchronizes the active physical project generation and pumps terminal
    /// tickets. Dirty document projection is intentionally deferred until the
    /// returned poll says the interval is due.
    pub(crate) fn poll_project(&self, project_root: Option<&Path>) -> EditorAutosavePoll {
        let instant = Instant::now();
        let mut state = self.lock_state();
        state.synchronize_project(&self.jobs, self.policy, project_root, instant);
        let Some(project_started_at) = state.project_started_at else {
            return EditorAutosavePoll::default();
        };
        let now = instant.saturating_duration_since(project_started_at);
        let Some(adapter) = state.adapter.as_mut() else {
            return EditorAutosavePoll::default();
        };
        let completion = adapter.pump_completed(now);
        EditorAutosavePoll {
            now,
            due: adapter.is_due(now),
            completion,
        }
    }

    pub(crate) fn schedule(
        &self,
        now: Duration,
        documents: &[AutosaveDocumentState],
        estimated_bytes_for: impl FnMut(&AutosaveDocumentId) -> usize,
        request_for: impl FnMut(&AutosaveDocumentId) -> Option<AutosaveDocumentRequest>,
    ) -> Result<bool, AutosaveAdmissionError> {
        let mut state = self.lock_state();
        let Some(adapter) = state.adapter.as_mut() else {
            return Ok(false);
        };
        adapter.schedule(now, documents, estimated_bytes_for, request_for)
    }

    pub(crate) fn preflight_schedule(&self, now: Duration) -> Result<bool, AutosaveAdmissionError> {
        let state = self.lock_state();
        let Some(adapter) = state.adapter.as_ref() else {
            return Ok(false);
        };
        adapter.preflight_schedule(now)
    }

    pub(crate) fn begin_shutdown(&self) {
        let mut state = self.lock_state();
        if let Some(adapter) = state.adapter.as_mut() {
            adapter.begin_shutdown();
        }
    }

    pub(crate) fn shutdown(&self, deadline: Instant) -> Vec<UnfinishedEditorJob> {
        self.begin_shutdown();
        let unfinished = self.jobs.shutdown(deadline);
        let mut state = self.lock_state();
        state.adapter = None;
        state.project_root = None;
        state.project_started_at = None;
        unfinished
    }

    fn lock_state(&self) -> MutexGuard<'_, EditorAutosaveState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl EditorAutosaveState {
    fn synchronize_project(
        &mut self,
        jobs: &EditorJobSystem,
        policy: AutosavePolicy,
        project_root: Option<&Path>,
        now: Instant,
    ) {
        if self.project_root.as_deref() == project_root {
            return;
        }
        if let Some(adapter) = self.adapter.as_mut() {
            adapter.begin_shutdown();
        }
        self.adapter = None;
        self.project_root = project_root.map(Path::to_path_buf);
        self.project_started_at = project_root.map(|_| now);
        if let Some(project_root) = project_root {
            self.adapter = Some(AutosaveJobAdapter::new(
                jobs.clone(),
                AutosaveStore::new(project_root),
                AutosaveScheduler::new(policy),
            ));
        }
    }
}
