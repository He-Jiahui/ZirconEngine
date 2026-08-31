use std::{
    collections::HashSet,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use crate::core::jobs::JobError;
use zircon_runtime_interface::export::ExportStage;

use super::{
    ExportStageProgressKind, ExportWizardJobEvent, ExportWizardJobEventKind,
    ExportWizardJobSnapshot, ExportWizardJobState, ExportWizardJobStatus, ExportWizardPipelinePlan,
    ExportWizardStageArtifactPath, ExportWizardStageExecution, ExportWizardStageOutputBuffer,
};

const MAX_EVENTS_PER_DRAIN: usize = 64;
const MAX_EVENT_DRAIN_TIME: Duration = Duration::from_millis(2);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardControlState {
    pub status: ExportWizardJobStatus,
    pub status_label: &'static str,
    pub plan_ready: bool,
    pub missing_input_count: usize,
    pub can_start: bool,
    pub can_cancel: bool,
    pub can_close: bool,
    pub show_progress: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStageViewRow {
    pub stage: ExportStage,
    pub stage_id: &'static str,
    pub label: &'static str,
    pub progress_kind: ExportStageProgressKind,
    pub is_current: bool,
    pub report_path: Option<String>,
    pub artifact_paths: Vec<ExportWizardStageArtifactPath>,
    pub planned_artifacts: Vec<ExportWizardStageArtifactPath>,
    pub diagnostics: Vec<String>,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub missing_inputs: Vec<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStagePlannedArtifacts {
    pub stage: ExportStage,
    pub artifacts: Vec<ExportWizardStageArtifactPath>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStageMissingInputs {
    pub stage: ExportStage,
    pub inputs: Vec<&'static str>,
}

// Data-only bridge between the background job controller and retained UI binding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPanelViewModel {
    snapshot: ExportWizardJobSnapshot,
    plan_ready: bool,
    active_job: bool,
    planned_artifacts: Vec<ExportWizardStagePlannedArtifacts>,
    missing_inputs: Vec<ExportWizardStageMissingInputs>,
    latest_event_kind: Option<ExportWizardJobEventKind>,
    event_count: usize,
    coalesced_output_events: u64,
}

impl ExportWizardPanelViewModel {
    pub fn from_plan(job_id: impl Into<String>, plan: &ExportWizardPipelinePlan) -> Self {
        let snapshot = ExportWizardJobState::new(job_id, plan).into_snapshot();
        let planned_artifacts = plan
            .stages
            .iter()
            .map(|command| ExportWizardStagePlannedArtifacts {
                stage: command.stage,
                artifacts: command.produced_artifacts.clone(),
            })
            .collect();
        let missing_inputs = plan
            .stages
            .iter()
            .filter(|command| !command.missing_inputs.is_empty())
            .map(|command| ExportWizardStageMissingInputs {
                stage: command.stage,
                inputs: command.missing_inputs.clone(),
            })
            .collect();
        Self {
            snapshot,
            plan_ready: plan.is_ready(),
            active_job: false,
            planned_artifacts,
            missing_inputs,
            latest_event_kind: None,
            event_count: 0,
            coalesced_output_events: 0,
        }
    }

    pub fn apply_event(&mut self, event: ExportWizardJobEvent) {
        self.latest_event_kind = Some(event.kind);
        self.coalesced_output_events = self
            .coalesced_output_events
            .max(event.coalesced_output_events);
        let mut snapshot = match event.output_delta {
            Some(output_delta) => {
                let mut snapshot = self.snapshot.clone();
                snapshot.job_id = event.snapshot.job_id;
                snapshot.profile = event.snapshot.profile;
                snapshot.out = event.snapshot.out;
                snapshot.status = event.snapshot.status;
                snapshot.fatal = event.snapshot.fatal;
                snapshot.cancel_requested = event.snapshot.cancel_requested;
                snapshot.apply_stage_output(
                    output_delta.stage,
                    output_delta.output,
                    output_delta.progress,
                );
                snapshot
            }
            None => event.snapshot,
        };
        if self.snapshot.cancel_requested && !snapshot.cancel_requested && !snapshot.is_terminal() {
            snapshot.cancel_requested = true;
            if matches!(
                snapshot.status,
                ExportWizardJobStatus::Pending | ExportWizardJobStatus::Running
            ) {
                snapshot.status = ExportWizardJobStatus::Cancelling;
            }
        }
        self.active_job = !snapshot.is_terminal();
        self.snapshot = snapshot;
        self.event_count += 1;
    }

    pub fn mark_job_started(&mut self) {
        self.active_job = true;
        if matches!(self.snapshot.status, ExportWizardJobStatus::Pending) {
            self.snapshot.status = ExportWizardJobStatus::Running;
        }
    }

    pub fn mark_cancel_requested(&mut self) {
        if self.snapshot.is_terminal() {
            return;
        }
        self.active_job = true;
        self.snapshot.cancel_requested = true;
        if matches!(
            self.snapshot.status,
            ExportWizardJobStatus::Pending | ExportWizardJobStatus::Running
        ) {
            self.snapshot.status = ExportWizardJobStatus::Cancelling;
        }
    }

    pub fn mark_job_finished(&mut self, snapshot: &ExportWizardJobSnapshot) {
        self.active_job = false;
        if self.snapshot == *snapshot {
            return;
        }
        if let Some(kind) = terminal_event_kind(snapshot.status) {
            self.apply_event(ExportWizardJobEvent {
                kind,
                snapshot: snapshot.clone(),
                output_delta: None,
                coalesced_output_events: self.coalesced_output_events,
            });
        }
    }

    /// Converts infrastructure terminal errors into the wizard's single snapshot state source.
    pub fn mark_job_error(&mut self, error: &JobError) {
        let mut snapshot = self.snapshot.clone();
        match error {
            JobError::Cancelled => {
                snapshot.status = ExportWizardJobStatus::Cancelled;
                snapshot.cancel_requested = true;
                snapshot.fatal = false;
            }
            JobError::Failed(_) | JobError::Panicked(_) | JobError::ResultChannelClosed => {
                snapshot.status = ExportWizardJobStatus::Failed;
                snapshot.fatal = true;
                let diagnostic = error.to_string();
                if !snapshot.diagnostics.contains(&diagnostic) {
                    snapshot.diagnostics.push(diagnostic);
                }
            }
        }
        self.mark_job_finished(&snapshot);
    }

    pub fn drain_events(&mut self, events: &Receiver<ExportWizardJobEvent>) -> usize {
        let started_at = Instant::now();
        let mut drained = 0;
        while drained < MAX_EVENTS_PER_DRAIN && started_at.elapsed() < MAX_EVENT_DRAIN_TIME {
            let Ok(event) = events.try_recv() else {
                break;
            };
            self.apply_event(event);
            drained += 1;
        }
        drained
    }

    pub fn snapshot(&self) -> &ExportWizardJobSnapshot {
        &self.snapshot
    }

    pub fn latest_event_kind(&self) -> Option<ExportWizardJobEventKind> {
        self.latest_event_kind
    }

    pub fn event_count(&self) -> usize {
        self.event_count
    }

    pub fn coalesced_output_events(&self) -> u64 {
        self.coalesced_output_events
    }

    pub fn plan_ready(&self) -> bool {
        self.plan_ready
    }

    pub fn missing_input_count(&self) -> usize {
        self.missing_inputs
            .iter()
            .map(|missing| missing.inputs.len())
            .sum()
    }

    pub fn controls(&self) -> ExportWizardControlState {
        ExportWizardControlState {
            status: self.snapshot.status,
            status_label: status_label(self.snapshot.status),
            plan_ready: self.plan_ready,
            missing_input_count: self.missing_input_count(),
            can_start: !self.active_job
                && self.plan_ready
                && !self.snapshot.fatal
                && !self.snapshot.cancel_requested
                && self.snapshot.status == ExportWizardJobStatus::Pending,
            can_cancel: self.active_job && !self.snapshot.cancel_requested,
            can_close: !self.active_job || self.snapshot.is_terminal(),
            show_progress: self.active_job
                || self.event_count > 0
                || !self.snapshot.stages.is_empty(),
        }
    }

    pub fn stage_rows(&self) -> Vec<ExportWizardStageViewRow> {
        self.snapshot
            .progress
            .snapshots()
            .iter()
            .map(|progress| {
                let stage_execution = self.stage_execution(progress.stage);
                let stage_output = self.stage_output(progress.stage);
                let planned_artifacts = self.planned_artifacts(progress.stage);
                let report_path = progress
                    .report_path
                    .clone()
                    .or_else(|| report_path_from_artifacts(&planned_artifacts));
                ExportWizardStageViewRow {
                    stage: progress.stage,
                    stage_id: progress.stage.cli_id(),
                    label: progress.stage.report_name(),
                    progress_kind: self.row_progress_kind(progress.stage, progress.kind),
                    is_current: self.snapshot.current_stage == Some(progress.stage),
                    report_path,
                    artifact_paths: progress.artifact_paths.clone(),
                    planned_artifacts,
                    diagnostics: stage_diagnostics(progress.diagnostics.clone(), stage_execution),
                    stdout_lines: stage_stdout_lines(stage_execution, stage_output),
                    stderr_lines: stage_stderr_lines(stage_execution, stage_output),
                    missing_inputs: self.missing_inputs(progress.stage),
                }
            })
            .collect()
    }

    fn row_progress_kind(
        &self,
        stage: ExportStage,
        progress_kind: ExportStageProgressKind,
    ) -> ExportStageProgressKind {
        let Some(execution) = self.stage_execution(stage) else {
            if self.snapshot.current_stage == Some(stage)
                && matches!(
                    self.snapshot.status,
                    ExportWizardJobStatus::Running | ExportWizardJobStatus::Cancelling
                )
            {
                return ExportStageProgressKind::Running;
            }
            return progress_kind;
        };
        if execution.cancelled {
            progress_kind
        } else if execution.fatal {
            ExportStageProgressKind::Fatal
        } else if matches!(
            progress_kind,
            ExportStageProgressKind::Pending | ExportStageProgressKind::Running
        ) {
            ExportStageProgressKind::Passed
        } else {
            progress_kind
        }
    }

    fn planned_artifacts(&self, stage: ExportStage) -> Vec<ExportWizardStageArtifactPath> {
        self.planned_artifacts
            .iter()
            .find(|artifacts| artifacts.stage == stage)
            .map(|artifacts| artifacts.artifacts.clone())
            .unwrap_or_default()
    }

    fn missing_inputs(&self, stage: ExportStage) -> Vec<&'static str> {
        self.missing_inputs
            .iter()
            .find(|missing| missing.stage == stage)
            .map(|missing| missing.inputs.clone())
            .unwrap_or_default()
    }

    fn stage_execution(&self, stage: ExportStage) -> Option<&ExportWizardStageExecution> {
        self.snapshot
            .stages
            .iter()
            .find(|entry| entry.stage == stage)
    }

    fn stage_output(&self, stage: ExportStage) -> Option<&ExportWizardStageOutputBuffer> {
        self.snapshot
            .live_stage_outputs
            .iter()
            .find(|entry| entry.stage == stage)
    }
}

fn status_label(status: ExportWizardJobStatus) -> &'static str {
    match status {
        ExportWizardJobStatus::Pending => "Pending",
        ExportWizardJobStatus::Running => "Running",
        ExportWizardJobStatus::Cancelling => "Cancelling",
        ExportWizardJobStatus::Cancelled => "Cancelled",
        ExportWizardJobStatus::Finished => "Finished",
        ExportWizardJobStatus::Failed => "Failed",
    }
}

fn terminal_event_kind(status: ExportWizardJobStatus) -> Option<ExportWizardJobEventKind> {
    match status {
        ExportWizardJobStatus::Cancelled => Some(ExportWizardJobEventKind::Cancelled),
        ExportWizardJobStatus::Failed => Some(ExportWizardJobEventKind::Failed),
        ExportWizardJobStatus::Finished => Some(ExportWizardJobEventKind::Finished),
        ExportWizardJobStatus::Pending
        | ExportWizardJobStatus::Running
        | ExportWizardJobStatus::Cancelling => None,
    }
}

fn report_path_from_artifacts(artifacts: &[ExportWizardStageArtifactPath]) -> Option<String> {
    artifacts
        .iter()
        .find(|artifact| artifact.key == "report" || artifact.key == "pipeline_report")
        .map(|artifact| artifact.path.clone())
}

fn stage_diagnostics(
    diagnostics: Vec<String>,
    execution: Option<&ExportWizardStageExecution>,
) -> Vec<String> {
    if let Some(execution) = execution {
        return merge_unique_diagnostics(diagnostics, &execution.diagnostics);
    }
    diagnostics
}

fn merge_unique_diagnostics(mut diagnostics: Vec<String>, additions: &[String]) -> Vec<String> {
    let mut seen = diagnostics
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let additions = additions
        .iter()
        .filter(|diagnostic| seen.insert(diagnostic.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    diagnostics.extend(additions);
    diagnostics
}

fn stage_stdout_lines(
    execution: Option<&ExportWizardStageExecution>,
    output: Option<&ExportWizardStageOutputBuffer>,
) -> Vec<String> {
    if let Some(execution) = execution {
        return execution.stdout_lines.clone();
    }
    output
        .map(|output| output.stdout_lines.iter().cloned().collect())
        .unwrap_or_default()
}

fn stage_stderr_lines(
    execution: Option<&ExportWizardStageExecution>,
    output: Option<&ExportWizardStageOutputBuffer>,
) -> Vec<String> {
    if let Some(execution) = execution {
        return execution.stderr_lines.clone();
    }
    output
        .map(|output| output.stderr_lines.iter().cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "view_model/stage_diagnostics_tests.rs"]
mod stage_diagnostics_tests;
