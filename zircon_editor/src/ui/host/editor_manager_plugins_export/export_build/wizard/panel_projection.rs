use zircon_runtime::plugin::ExportPipelineStage;

use super::{
    export_pipeline_stage_cli_id, ExportStageProgressKind, ExportWizardControlState,
    ExportWizardJobStatus, ExportWizardPanelViewModel, ExportWizardStageArtifactPath,
    ExportWizardStageViewRow, DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BUTTON,
    DESKTOP_EXPORT_START_BUTTON,
};

pub const DESKTOP_EXPORT_MISSING_INPUTS_SLOT: &str = "DesktopExportMissingInputs";
pub const DESKTOP_EXPORT_STAGE_ROWS_SLOT: &str = "DesktopExportStageRows";
pub const DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT: &str = "DesktopExportTerminalOutput";
pub const DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT: &str = "DesktopExportArtifactPaths";
pub const DESKTOP_EXPORT_REPORT_BODY_SLOT: &str = "DesktopExportReportBody";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWizardPanelSlotKind {
    MissingInputs,
    StageRows,
    TerminalOutput,
    ArtifactPaths,
    ReportBody,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWizardPanelEntrySeverity {
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPanelSlotEntry {
    pub key: String,
    pub label: String,
    pub detail: String,
    pub stage: Option<ExportPipelineStage>,
    pub severity: ExportWizardPanelEntrySeverity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPanelSlotState {
    pub control_id: &'static str,
    pub kind: ExportWizardPanelSlotKind,
    pub entries: Vec<ExportWizardPanelSlotEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPanelControlBindingState {
    pub control_id: &'static str,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPanelTemplateState {
    pub controls: ExportWizardControlState,
    pub control_bindings: Vec<ExportWizardPanelControlBindingState>,
    pub slots: Vec<ExportWizardPanelSlotState>,
}

impl ExportWizardPanelTemplateState {
    pub fn slot(&self, kind: ExportWizardPanelSlotKind) -> Option<&ExportWizardPanelSlotState> {
        self.slots.iter().find(|slot| slot.kind == kind)
    }

    pub fn control(&self, control_id: &str) -> Option<&ExportWizardPanelControlBindingState> {
        self.control_bindings
            .iter()
            .find(|control| control.control_id == control_id)
    }
}

pub fn export_wizard_panel_template_state(
    view_model: &ExportWizardPanelViewModel,
) -> ExportWizardPanelTemplateState {
    let controls = view_model.controls();
    let rows = view_model.stage_rows();
    let control_bindings = control_binding_states(&controls);
    ExportWizardPanelTemplateState {
        controls,
        control_bindings,
        slots: vec![
            slot_state(
                DESKTOP_EXPORT_MISSING_INPUTS_SLOT,
                ExportWizardPanelSlotKind::MissingInputs,
                missing_input_entries(&rows),
            ),
            slot_state(
                DESKTOP_EXPORT_STAGE_ROWS_SLOT,
                ExportWizardPanelSlotKind::StageRows,
                stage_row_entries(&rows),
            ),
            slot_state(
                DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT,
                ExportWizardPanelSlotKind::TerminalOutput,
                terminal_output_entries(view_model, &rows),
            ),
            slot_state(
                DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT,
                ExportWizardPanelSlotKind::ArtifactPaths,
                artifact_path_entries(&rows),
            ),
            slot_state(
                DESKTOP_EXPORT_REPORT_BODY_SLOT,
                ExportWizardPanelSlotKind::ReportBody,
                report_body_entries(view_model, &rows),
            ),
        ],
    }
}

fn control_binding_states(
    controls: &ExportWizardControlState,
) -> Vec<ExportWizardPanelControlBindingState> {
    vec![
        ExportWizardPanelControlBindingState {
            control_id: DESKTOP_EXPORT_GENERATE_PLAN_BUTTON,
            enabled: controls.can_close,
        },
        ExportWizardPanelControlBindingState {
            control_id: DESKTOP_EXPORT_START_BUTTON,
            enabled: controls.can_start,
        },
        ExportWizardPanelControlBindingState {
            control_id: DESKTOP_EXPORT_CANCEL_BUTTON,
            enabled: controls.can_cancel,
        },
    ]
}

fn slot_state(
    control_id: &'static str,
    kind: ExportWizardPanelSlotKind,
    entries: Vec<ExportWizardPanelSlotEntry>,
) -> ExportWizardPanelSlotState {
    ExportWizardPanelSlotState {
        control_id,
        kind,
        entries,
    }
}

fn missing_input_entries(rows: &[ExportWizardStageViewRow]) -> Vec<ExportWizardPanelSlotEntry> {
    rows.iter()
        .filter(|row| !row.missing_inputs.is_empty())
        .map(|row| ExportWizardPanelSlotEntry {
            key: format!("missing.{}", row.stage_id),
            label: row.label.to_string(),
            detail: row.missing_inputs.join(", "),
            stage: Some(row.stage),
            severity: ExportWizardPanelEntrySeverity::Warning,
        })
        .collect()
}

fn stage_row_entries(rows: &[ExportWizardStageViewRow]) -> Vec<ExportWizardPanelSlotEntry> {
    rows.iter()
        .map(|row| ExportWizardPanelSlotEntry {
            key: format!("stage.{}", row.stage_id),
            label: row.label.to_string(),
            detail: stage_row_detail(row),
            stage: Some(row.stage),
            severity: severity_for_progress(row.progress_kind),
        })
        .collect()
}

fn terminal_output_entries(
    view_model: &ExportWizardPanelViewModel,
    rows: &[ExportWizardStageViewRow],
) -> Vec<ExportWizardPanelSlotEntry> {
    let snapshot = view_model.snapshot();
    let mut entries = snapshot
        .diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| ExportWizardPanelSlotEntry {
            key: format!("diagnostic.{index}"),
            label: "Diagnostic".to_string(),
            detail: diagnostic.clone(),
            stage: None,
            severity: if snapshot.fatal {
                ExportWizardPanelEntrySeverity::Danger
            } else {
                ExportWizardPanelEntrySeverity::Info
            },
        })
        .collect::<Vec<_>>();

    for row in rows {
        for (index, line) in row.stdout_lines.iter().enumerate() {
            entries.push(ExportWizardPanelSlotEntry {
                key: format!("stage-output.{}.stdout.{index}", row.stage_id),
                label: format!("{} stdout", row.label),
                detail: line.clone(),
                stage: Some(row.stage),
                severity: ExportWizardPanelEntrySeverity::Info,
            });
        }
        for (index, line) in row.stderr_lines.iter().enumerate() {
            entries.push(ExportWizardPanelSlotEntry {
                key: format!("stage-output.{}.stderr.{index}", row.stage_id),
                label: format!("{} stderr", row.label),
                detail: line.clone(),
                stage: Some(row.stage),
                severity: if row.progress_kind == ExportStageProgressKind::Fatal {
                    ExportWizardPanelEntrySeverity::Danger
                } else {
                    ExportWizardPanelEntrySeverity::Warning
                },
            });
        }
        for (index, diagnostic) in row.diagnostics.iter().enumerate() {
            entries.push(ExportWizardPanelSlotEntry {
                key: format!("stage-diagnostic.{}.{}", row.stage_id, index),
                label: row.label.to_string(),
                detail: diagnostic.clone(),
                stage: Some(row.stage),
                severity: if row.progress_kind == ExportStageProgressKind::Fatal {
                    ExportWizardPanelEntrySeverity::Danger
                } else {
                    ExportWizardPanelEntrySeverity::Info
                },
            });
        }
    }

    entries
}

fn artifact_path_entries(rows: &[ExportWizardStageViewRow]) -> Vec<ExportWizardPanelSlotEntry> {
    let mut entries = Vec::new();
    for row in rows {
        for artifact in merged_artifacts(row) {
            entries.push(ExportWizardPanelSlotEntry {
                key: format!("artifact.{}.{}", row.stage_id, artifact.key),
                label: format!("{} {}", row.label, artifact.key),
                detail: artifact.path,
                stage: Some(row.stage),
                severity: ExportWizardPanelEntrySeverity::Neutral,
            });
        }
    }
    entries
}

fn report_body_entries(
    view_model: &ExportWizardPanelViewModel,
    rows: &[ExportWizardStageViewRow],
) -> Vec<ExportWizardPanelSlotEntry> {
    let snapshot = view_model.snapshot();
    let controls = view_model.controls();
    let mut entries = vec![ExportWizardPanelSlotEntry {
        key: "status".to_string(),
        label: controls.status_label.to_string(),
        detail: format!("{} | {}", snapshot.profile, snapshot.out),
        stage: snapshot.current_stage,
        severity: severity_for_status(snapshot.status, snapshot.fatal),
    }];

    if let Some(stage) = snapshot.current_stage {
        entries.push(ExportWizardPanelSlotEntry {
            key: "current_stage".to_string(),
            label: "Current Stage".to_string(),
            detail: export_pipeline_stage_cli_id(stage).to_string(),
            stage: Some(stage),
            severity: ExportWizardPanelEntrySeverity::Info,
        });
    }

    if let Some(pipeline_report) = pipeline_report_body_entry(rows) {
        entries.push(pipeline_report);
    }

    entries
}

fn pipeline_report_body_entry(
    rows: &[ExportWizardStageViewRow],
) -> Option<ExportWizardPanelSlotEntry> {
    let report = rows
        .iter()
        .find(|row| row.stage == ExportPipelineStage::Report)?;
    let path = artifact_path_for_key(report, "pipeline_report")?;
    Some(ExportWizardPanelSlotEntry {
        key: "report.pipeline_report".to_string(),
        label: "Pipeline Report".to_string(),
        detail: path,
        stage: Some(ExportPipelineStage::Report),
        severity: severity_for_progress(report.progress_kind),
    })
}

fn stage_row_detail(row: &ExportWizardStageViewRow) -> String {
    let mut parts = vec![progress_label(row.progress_kind).to_string()];
    if row.is_current {
        parts.push("current".to_string());
    }
    if !row.missing_inputs.is_empty() {
        parts.push(format!("missing {}", row.missing_inputs.join(", ")));
    }
    if let Some(report_path) = row.report_path.as_ref() {
        parts.push(format!("report {report_path}"));
    }
    parts.join(" | ")
}

fn merged_artifacts(row: &ExportWizardStageViewRow) -> Vec<ExportWizardStageArtifactPath> {
    let mut artifacts = row.planned_artifacts.clone();
    for artifact in &row.artifact_paths {
        if !artifacts
            .iter()
            .any(|existing| existing.key == artifact.key && existing.path == artifact.path)
        {
            artifacts.push(artifact.clone());
        }
    }
    artifacts
}

fn artifact_path_for_key(row: &ExportWizardStageViewRow, key: &str) -> Option<String> {
    row.artifact_paths
        .iter()
        .rev()
        .find(|artifact| artifact.key == key)
        .or_else(|| {
            row.planned_artifacts
                .iter()
                .find(|artifact| artifact.key == key)
        })
        .map(|artifact| artifact.path.clone())
}

fn progress_label(kind: ExportStageProgressKind) -> &'static str {
    match kind {
        ExportStageProgressKind::Pending => "Pending",
        ExportStageProgressKind::Running => "Running",
        ExportStageProgressKind::Passed => "Passed",
        ExportStageProgressKind::Fatal => "Fatal",
    }
}

fn severity_for_progress(kind: ExportStageProgressKind) -> ExportWizardPanelEntrySeverity {
    match kind {
        ExportStageProgressKind::Pending => ExportWizardPanelEntrySeverity::Neutral,
        ExportStageProgressKind::Running => ExportWizardPanelEntrySeverity::Info,
        ExportStageProgressKind::Passed => ExportWizardPanelEntrySeverity::Success,
        ExportStageProgressKind::Fatal => ExportWizardPanelEntrySeverity::Danger,
    }
}

fn severity_for_status(
    status: ExportWizardJobStatus,
    fatal: bool,
) -> ExportWizardPanelEntrySeverity {
    if fatal {
        return ExportWizardPanelEntrySeverity::Danger;
    }
    match status {
        ExportWizardJobStatus::Pending => ExportWizardPanelEntrySeverity::Neutral,
        ExportWizardJobStatus::Running | ExportWizardJobStatus::Cancelling => {
            ExportWizardPanelEntrySeverity::Info
        }
        ExportWizardJobStatus::Cancelled => ExportWizardPanelEntrySeverity::Warning,
        ExportWizardJobStatus::Finished => ExportWizardPanelEntrySeverity::Success,
        ExportWizardJobStatus::Failed => ExportWizardPanelEntrySeverity::Danger,
    }
}
