use std::collections::BTreeMap;
use std::thread;
use std::time::Instant;

use crate::core::jobs::UnfinishedEditorJob;

use super::autosave_service::{
    ActiveAutosaveProject, AutosaveDiagnosticPersistenceIssue, EditorAutosaveService,
    RetiredAutosaveProject, persist_fallback_diagnostics,
};
use super::{
    AutosaveCompletion, AutosaveDocumentId, AutosaveDocumentOutcome, AutosaveDocumentRequest,
    AutosaveDocumentState,
};

/// Terminal evidence gathered while the editor drains autosave before shared
/// jobs stop. Every outcome is document-bound, including deadline exhaustion.
#[derive(Clone, Debug, Default)]
pub(crate) struct AutosaveShutdownReport {
    outcomes: Vec<AutosaveDocumentOutcome>,
    diagnostic_persistence_issues: Vec<AutosaveDiagnosticPersistenceIssue>,
    unfinished_jobs: Vec<UnfinishedEditorJob>,
}

impl AutosaveShutdownReport {
    pub(crate) fn outcomes(&self) -> &[AutosaveDocumentOutcome] {
        &self.outcomes
    }

    pub(crate) fn diagnostic_persistence_issues(&self) -> &[AutosaveDiagnosticPersistenceIssue] {
        &self.diagnostic_persistence_issues
    }

    pub(crate) fn unfinished_jobs(&self) -> &[UnfinishedEditorJob] {
        &self.unfinished_jobs
    }
}

impl EditorAutosaveService {
    /// Fences interval autosave, drains final snapshots while the shared job
    /// system remains live, then performs the global deadline shutdown.
    pub(crate) fn shutdown_with_final_autosave(
        &self,
        requests: Vec<AutosaveDocumentRequest>,
        deadline: Instant,
    ) -> AutosaveShutdownReport {
        let mut report = AutosaveShutdownReport::default();
        let mut remaining = requests
            .into_iter()
            .map(|request| (request.document().clone(), request))
            .collect::<BTreeMap<_, _>>();
        let mut active_project_root = None;
        let mut final_batch_submitted = false;

        {
            let mut state = self.lock_state();
            if let Some(active) = state.active.as_mut() {
                active_project_root = Some(active.project_root.clone());
                active.adapter.fence_regular_admission();
                final_batch_submitted = drain_final_autosave_until_deadline(
                    active,
                    &mut remaining,
                    deadline,
                    &mut report,
                );
            } else if !remaining.is_empty() {
                let unbound_outcomes = remaining
                    .values()
                    .map(|request| {
                        AutosaveDocumentOutcome::shutdown_unavailable(
                            request.document().clone(),
                            request.source_path().clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                remaining.clear();
                report.outcomes.extend(unbound_outcomes);
            }
            if let Some(active) = state.active.as_mut() {
                active.adapter.begin_shutdown();
            }
            for retired in &mut state.retired {
                retired.adapter.begin_shutdown();
            }
        }

        report.unfinished_jobs = self.jobs.shutdown(deadline);

        let mut state = self.lock_state();
        let instant = Instant::now();
        if let Some(active) = state.active.as_mut() {
            let now = instant.saturating_duration_since(active.project_started_at);
            let mut completion = active.adapter.pump_completed(now);
            collect_active_completion(
                &mut report,
                &active.project_root,
                &mut remaining,
                &mut completion,
                final_batch_submitted,
                final_batch_submitted,
            );
        }
        for retired in &mut state.retired {
            collect_retired_completion(&mut report, retired, instant);
        }

        if let Some(project_root) = active_project_root {
            let mut deadline_outcomes = remaining
                .into_values()
                .map(|request| {
                    AutosaveDocumentOutcome::shutdown_deadline(
                        request.document().clone(),
                        request.source_path().clone(),
                    )
                })
                .collect::<Vec<_>>();
            report
                .diagnostic_persistence_issues
                .extend(persist_fallback_diagnostics(
                    &project_root,
                    &mut deadline_outcomes,
                ));
            report.outcomes.extend(deadline_outcomes);
        }
        state.active = None;
        state.retired.clear();
        report
    }
}

fn drain_final_autosave_until_deadline(
    active: &mut ActiveAutosaveProject,
    remaining: &mut BTreeMap<AutosaveDocumentId, AutosaveDocumentRequest>,
    deadline: Instant,
    report: &mut AutosaveShutdownReport,
) -> bool {
    let mut final_batch_submitted = false;
    loop {
        let instant = Instant::now();
        let now = instant.saturating_duration_since(active.project_started_at);
        let mut completion = active.adapter.pump_completed(now);
        collect_active_completion(
            report,
            &active.project_root,
            remaining,
            &mut completion,
            final_batch_submitted,
            final_batch_submitted,
        );

        if active.adapter.is_drained() && !remaining.is_empty() {
            let documents = remaining
                .keys()
                .cloned()
                .map(|document| AutosaveDocumentState::from_dirty_projection(document, true))
                .collect::<Vec<_>>();
            match active.adapter.schedule_final(
                now,
                &documents,
                |document| {
                    remaining
                        .get(document)
                        .map_or(1, AutosaveDocumentRequest::estimated_pending_bytes)
                },
                |document| remaining.get(document).cloned(),
            ) {
                Ok(true) => {
                    final_batch_submitted = true;
                    continue;
                }
                Ok(false) | Err(_) => {}
            }
        }

        if remaining.is_empty() && active.adapter.is_drained() {
            return final_batch_submitted;
        }
        if instant >= deadline {
            return final_batch_submitted;
        }
        thread::yield_now();
    }
}

fn collect_active_completion(
    report: &mut AutosaveShutdownReport,
    project_root: &std::path::Path,
    remaining: &mut BTreeMap<AutosaveDocumentId, AutosaveDocumentRequest>,
    completion: &mut AutosaveCompletion,
    consume_requests: bool,
    include_in_report: bool,
) {
    report
        .diagnostic_persistence_issues
        .extend(persist_fallback_diagnostics(
            project_root,
            completion.outcomes_mut(),
        ));
    if consume_requests {
        for outcome in completion.outcomes() {
            remaining.remove(outcome.document());
        }
    }
    if include_in_report {
        report
            .outcomes
            .extend(completion.outcomes().iter().cloned());
    }
}

fn collect_retired_completion(
    report: &mut AutosaveShutdownReport,
    retired: &mut RetiredAutosaveProject,
    instant: Instant,
) {
    let now = instant.saturating_duration_since(retired.project_started_at);
    let completion = retired.adapter.pump_completed(now);
    retired.fallback_diagnostics.extend(
        completion
            .outcomes()
            .iter()
            .filter(|outcome| !outcome.diagnostic_persisted())
            .cloned(),
    );
    report
        .outcomes
        .extend(completion.outcomes().iter().cloned());
    report
        .diagnostic_persistence_issues
        .extend(retired.persist_fallback_diagnostics());
}
