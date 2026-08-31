use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::core::jobs::{EditorJobSystem, UnfinishedEditorJob};

use super::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDiagnosticStore, AutosaveDocumentId,
    AutosaveDocumentOutcome, AutosaveDocumentRequest, AutosaveDocumentState, AutosaveJobAdapter,
    AutosavePolicy, AutosaveScheduler, AutosaveStore, DEFAULT_AUTOSAVE_COMPLETION_BUDGET,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AutosaveDiagnosticPersistenceIssue {
    project_root: PathBuf,
    document: AutosaveDocumentId,
    message: String,
}

impl AutosaveDiagnosticPersistenceIssue {
    fn new(project_root: PathBuf, outcome: &AutosaveDocumentOutcome, message: String) -> Self {
        Self {
            project_root,
            document: outcome.document().clone(),
            message,
        }
    }

    pub(crate) fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub(crate) fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorAutosavePoll {
    now: Duration,
    due: bool,
    completion: AutosaveCompletion,
    diagnostic_persistence_issues: Vec<AutosaveDiagnosticPersistenceIssue>,
}

impl EditorAutosavePoll {
    pub(crate) const fn now(&self) -> Duration {
        self.now
    }

    pub(crate) const fn is_due(&self) -> bool {
        self.due
    }

    pub(crate) fn completion(&self) -> &AutosaveCompletion {
        &self.completion
    }

    pub(crate) fn diagnostic_persistence_issues(&self) -> &[AutosaveDiagnosticPersistenceIssue] {
        &self.diagnostic_persistence_issues
    }
}

/// EditorContext-owned autosave lifecycle. It reuses the context's one job
/// system and retains draining adapters across project switches until their
/// terminal evidence is durable.
pub(crate) struct EditorAutosaveService {
    pub(super) jobs: EditorJobSystem,
    pub(super) state: Mutex<EditorAutosaveState>,
}

pub(super) struct EditorAutosaveState {
    policy: AutosavePolicy,
    pub(super) active: Option<ActiveAutosaveProject>,
    pub(super) retired: VecDeque<RetiredAutosaveProject>,
}

pub(super) struct ActiveAutosaveProject {
    pub(super) project_root: PathBuf,
    pub(super) project_started_at: Instant,
    pub(super) adapter: AutosaveJobAdapter,
}

pub(super) struct RetiredAutosaveProject {
    pub(super) project_root: PathBuf,
    pub(super) project_started_at: Instant,
    pub(super) adapter: AutosaveJobAdapter,
    pub(super) fallback_diagnostics: VecDeque<AutosaveDocumentOutcome>,
}

impl EditorAutosaveService {
    pub(crate) fn new(jobs: EditorJobSystem, policy: AutosavePolicy) -> Self {
        Self {
            jobs,
            state: Mutex::new(EditorAutosaveState {
                policy,
                active: None,
                retired: VecDeque::new(),
            }),
        }
    }

    /// Hot-applies a settings-validated cadence to the active project scheduler.
    pub(crate) fn update_policy(&self, policy: AutosavePolicy) {
        let mut state = self.lock_state();
        if state.policy == policy {
            return;
        }
        state.policy = policy;
        if let Some(active) = state.active.as_mut() {
            active.adapter.update_policy(policy);
        }
    }

    #[cfg(test)]
    pub(crate) fn policy(&self) -> AutosavePolicy {
        self.lock_state().policy
    }

    /// Synchronizes the active project and pumps both active and retired
    /// ticket generations. Dirty projection is deferred until the active
    /// adapter reports its interval due.
    pub(crate) fn poll_project(&self, project_root: Option<&Path>) -> EditorAutosavePoll {
        let instant = Instant::now();
        let mut state = self.lock_state();
        state.synchronize_project(&self.jobs, project_root, instant);
        let mut diagnostic_persistence_issues = state.pump_retired(instant);
        let Some(active) = state.active.as_mut() else {
            return EditorAutosavePoll {
                diagnostic_persistence_issues,
                ..EditorAutosavePoll::default()
            };
        };
        let now = instant.saturating_duration_since(active.project_started_at);
        let mut completion = active.adapter.pump_completed(now);
        diagnostic_persistence_issues.extend(persist_fallback_diagnostics(
            &active.project_root,
            completion.outcomes_mut(),
        ));
        EditorAutosavePoll {
            now,
            due: active.adapter.is_due(now),
            completion,
            diagnostic_persistence_issues,
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
        let Some(active) = state.active.as_mut() else {
            return Ok(false);
        };
        active
            .adapter
            .schedule(now, documents, estimated_bytes_for, request_for)
    }

    pub(crate) fn preflight_schedule(&self, now: Duration) -> Result<bool, AutosaveAdmissionError> {
        let state = self.lock_state();
        let Some(active) = state.active.as_ref() else {
            return Ok(false);
        };
        active.adapter.preflight_schedule(now)
    }

    pub(crate) fn begin_shutdown(&self) {
        let mut state = self.lock_state();
        if let Some(active) = state.active.as_mut() {
            active.adapter.begin_shutdown();
        }
        for retired in &mut state.retired {
            retired.adapter.begin_shutdown();
        }
    }

    pub(crate) fn shutdown(&self, deadline: Instant) -> Vec<UnfinishedEditorJob> {
        self.begin_shutdown();
        let unfinished = self.jobs.shutdown(deadline);
        let mut state = self.lock_state();
        state.active = None;
        state.retired.clear();
        unfinished
    }

    pub(super) fn lock_state(&self) -> MutexGuard<'_, EditorAutosaveState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl EditorAutosaveState {
    fn synchronize_project(
        &mut self,
        jobs: &EditorJobSystem,
        project_root: Option<&Path>,
        now: Instant,
    ) {
        if matches!(
            (self.active.as_ref(), project_root),
            (Some(active), Some(project_root)) if active.project_root.as_path() == project_root
        ) {
            return;
        }
        if self.active.is_none() && project_root.is_none() {
            return;
        }
        if let Some(mut active) = self.active.take() {
            active.adapter.begin_shutdown();
            self.retired.push_back(RetiredAutosaveProject {
                project_root: active.project_root,
                project_started_at: active.project_started_at,
                adapter: active.adapter,
                fallback_diagnostics: VecDeque::new(),
            });
        }
        if let Some(project_root) = project_root {
            let project_root = project_root.to_path_buf();
            self.active = Some(ActiveAutosaveProject {
                adapter: AutosaveJobAdapter::new(
                    jobs.clone(),
                    AutosaveStore::new(&project_root),
                    AutosaveScheduler::new(self.policy),
                ),
                project_root,
                project_started_at: now,
            });
        }
    }

    fn pump_retired(&mut self, instant: Instant) -> Vec<AutosaveDiagnosticPersistenceIssue> {
        let mut completion_budget = DEFAULT_AUTOSAVE_COMPLETION_BUDGET;
        let mut issues = Vec::new();
        let retired_count = self.retired.len();
        for _ in 0..retired_count {
            let Some(mut retired) = self.retired.pop_front() else {
                break;
            };
            let now = instant.saturating_duration_since(retired.project_started_at);
            let completion = retired
                .adapter
                .pump_completed_with_budget(now, completion_budget);
            completion_budget = completion_budget.saturating_sub(completion.inspected_tickets());
            retired.fallback_diagnostics.extend(
                completion
                    .outcomes()
                    .iter()
                    .filter(|outcome| !outcome.diagnostic_persisted())
                    .cloned(),
            );
            issues.extend(retired.persist_fallback_diagnostics());
            if !retired.is_finished() {
                self.retired.push_back(retired);
            }
        }
        issues
    }
}

impl RetiredAutosaveProject {
    fn is_finished(&self) -> bool {
        self.adapter.is_drained() && self.fallback_diagnostics.is_empty()
    }

    pub(super) fn persist_fallback_diagnostics(
        &mut self,
    ) -> Vec<AutosaveDiagnosticPersistenceIssue> {
        let mut issues = Vec::new();
        while let Some(mut outcome) = self.fallback_diagnostics.pop_front() {
            match AutosaveDiagnosticStore::new(&self.project_root).append(&outcome) {
                Ok(_) => outcome.mark_diagnostic_persisted(),
                Err(error) => {
                    issues.push(AutosaveDiagnosticPersistenceIssue::new(
                        self.project_root.clone(),
                        &outcome,
                        error.to_string(),
                    ));
                    self.fallback_diagnostics.push_front(outcome);
                    break;
                }
            }
        }
        issues
    }
}

/// Worker jobs persist normal terminal results themselves. This fallback only
/// handles cancellation synthesized before `AutosaveWriteJob::run` starts, at
/// the lifecycle boundary that must preserve retired-project evidence.
pub(super) fn persist_fallback_diagnostics(
    project_root: &Path,
    outcomes: &mut [AutosaveDocumentOutcome],
) -> Vec<AutosaveDiagnosticPersistenceIssue> {
    let store = AutosaveDiagnosticStore::new(project_root);
    let mut issues = Vec::new();
    for outcome in outcomes
        .iter_mut()
        .filter(|outcome| !outcome.diagnostic_persisted())
    {
        match store.append(outcome) {
            Ok(_) => outcome.mark_diagnostic_persisted(),
            Err(error) => issues.push(AutosaveDiagnosticPersistenceIssue::new(
                project_root.to_path_buf(),
                outcome,
                error.to_string(),
            )),
        }
    }
    issues
}
