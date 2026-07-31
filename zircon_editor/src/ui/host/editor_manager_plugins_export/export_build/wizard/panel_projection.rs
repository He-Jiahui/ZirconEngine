use zircon_runtime_interface::export::ExportStage;

use serde_json::Value;

use super::{
    ExportStageProgressKind, ExportWizardControlState, ExportWizardJobStatus,
    ExportWizardPanelViewModel, ExportWizardStageArtifactPath, ExportWizardStageViewRow,
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_START_BUTTON,
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
    pub stage: Option<ExportStage>,
    pub severity: ExportWizardPanelEntrySeverity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPanelSlotState {
    pub control_id: &'static str,
    pub kind: ExportWizardPanelSlotKind,
    pub entries: Vec<ExportWizardPanelSlotEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExportWizardReportPlanSummary {
    strategies: Vec<String>,
    required_stages: Vec<String>,
    completed_stages: Vec<String>,
    unsupported_strategies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExportWizardNativePluginsPayloadSummary {
    bundle_path: Option<String>,
    content_hash: Option<String>,
    file_count: Option<u64>,
    package_count: Option<u64>,
    package_ids: Option<Vec<String>>,
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
            detail: stage.cli_id().to_string(),
            stage: Some(stage),
            severity: ExportWizardPanelEntrySeverity::Info,
        });
    }

    if let Some(pipeline_report) = pipeline_report_body_entry(rows) {
        entries.push(pipeline_report);
    }

    let parsed_report = rows
        .iter()
        .find(|row| row.stage == ExportStage::Report)
        .and_then(|report| parsed_report_from_stdout(&report.stdout_lines));
    entries.extend(report_export_plan_body_entries(
        rows,
        parsed_report.as_ref(),
    ));
    entries.extend(report_native_plugins_payload_body_entries(
        rows,
        parsed_report.as_ref(),
    ));

    entries
}

fn pipeline_report_body_entry(
    rows: &[ExportWizardStageViewRow],
) -> Option<ExportWizardPanelSlotEntry> {
    let report = rows.iter().find(|row| row.stage == ExportStage::Report)?;
    let path = artifact_path_for_key(report, "pipeline_report")?;
    Some(ExportWizardPanelSlotEntry {
        key: "report.pipeline_report".to_string(),
        label: "Pipeline Report".to_string(),
        detail: path,
        stage: Some(ExportStage::Report),
        severity: severity_for_progress(report.progress_kind),
    })
}

fn report_export_plan_body_entries(
    rows: &[ExportWizardStageViewRow],
    parsed_report: Option<&Value>,
) -> Vec<ExportWizardPanelSlotEntry> {
    let Some(report) = rows.iter().find(|row| row.stage == ExportStage::Report) else {
        return Vec::new();
    };
    let Some(summary) = parsed_report.and_then(export_plan_summary_from_report) else {
        return Vec::new();
    };
    let unsupported_strategies_severity =
        unsupported_strategies_severity(&summary.unsupported_strategies);

    vec![
        export_plan_entry(
            "report.export_plan.strategies",
            "Export Strategies",
            summary.strategies,
            ExportWizardPanelEntrySeverity::Info,
        ),
        export_plan_entry(
            "report.export_plan.required_stages",
            "Required Stages",
            summary.required_stages,
            ExportWizardPanelEntrySeverity::Info,
        ),
        export_plan_entry(
            "report.export_plan.completed_stages",
            "Completed Stages",
            summary.completed_stages,
            severity_for_progress(report.progress_kind),
        ),
        export_plan_entry(
            "report.export_plan.unsupported_strategies",
            "Unsupported Strategies",
            summary.unsupported_strategies,
            unsupported_strategies_severity,
        ),
    ]
}

fn report_native_plugins_payload_body_entries(
    rows: &[ExportWizardStageViewRow],
    parsed_report: Option<&Value>,
) -> Vec<ExportWizardPanelSlotEntry> {
    let Some(report) = rows.iter().find(|row| row.stage == ExportStage::Report) else {
        return Vec::new();
    };
    let Some(summary) = parsed_report.and_then(native_plugins_payload_summary_from_report) else {
        return Vec::new();
    };
    let severity = severity_for_progress(report.progress_kind);
    let mut entries = Vec::new();
    push_optional_report_entry(
        &mut entries,
        "report.native_plugins_payload.bundle_path",
        "Native Plugins Bundle",
        summary.bundle_path,
        severity,
    );
    push_optional_report_entry(
        &mut entries,
        "report.native_plugins_payload.package_count",
        "Native Plugin Packages",
        summary.package_count.map(|value| value.to_string()),
        severity,
    );
    push_optional_report_entry(
        &mut entries,
        "report.native_plugins_payload.file_count",
        "Native Plugin Files",
        summary.file_count.map(|value| value.to_string()),
        severity,
    );
    push_optional_report_entry(
        &mut entries,
        "report.native_plugins_payload.content_hash",
        "Native Plugin Hash",
        summary.content_hash,
        severity,
    );
    if let Some(package_ids) = summary.package_ids {
        entries.push(export_plan_entry(
            "report.native_plugins_payload.package_ids",
            "Native Plugin Package Ids",
            package_ids,
            severity,
        ));
    }
    entries
}

fn push_optional_report_entry(
    entries: &mut Vec<ExportWizardPanelSlotEntry>,
    key: &str,
    label: &str,
    detail: Option<String>,
    severity: ExportWizardPanelEntrySeverity,
) {
    if let Some(detail) = detail {
        entries.push(ExportWizardPanelSlotEntry {
            key: key.to_string(),
            label: label.to_string(),
            detail,
            stage: Some(ExportStage::Report),
            severity,
        });
    }
}

fn export_plan_entry(
    key: &str,
    label: &str,
    values: Vec<String>,
    severity: ExportWizardPanelEntrySeverity,
) -> ExportWizardPanelSlotEntry {
    ExportWizardPanelSlotEntry {
        key: key.to_string(),
        label: label.to_string(),
        detail: value_list_detail(&values),
        stage: Some(ExportStage::Report),
        severity,
    }
}

fn unsupported_strategies_severity(values: &[String]) -> ExportWizardPanelEntrySeverity {
    if values.is_empty() {
        ExportWizardPanelEntrySeverity::Success
    } else {
        ExportWizardPanelEntrySeverity::Danger
    }
}

fn value_list_detail(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn parsed_report_from_stdout(stdout_lines: &[String]) -> Option<Value> {
    let report_json = report_json_from_stdout(stdout_lines)?;
    serde_json::from_str(&report_json).ok()
}

fn export_plan_summary_from_report(report: &Value) -> Option<ExportWizardReportPlanSummary> {
    let export_plan = report.get("export_plan")?;
    Some(ExportWizardReportPlanSummary {
        strategies: json_string_array(export_plan.get("strategies")),
        required_stages: json_string_array(export_plan.get("required_stages")),
        completed_stages: json_string_array(export_plan.get("completed_stages")),
        unsupported_strategies: json_string_array(export_plan.get("unsupported_strategies")),
    })
}

fn native_plugins_payload_summary_from_report(
    report: &Value,
) -> Option<ExportWizardNativePluginsPayloadSummary> {
    let payload = report.get("native_plugins_payload")?.as_object()?;
    Some(ExportWizardNativePluginsPayloadSummary {
        bundle_path: json_string(payload.get("bundle_path")),
        content_hash: json_string(payload.get("content_hash")),
        file_count: payload.get("file_count").and_then(Value::as_u64),
        package_count: payload.get("package_count").and_then(Value::as_u64),
        package_ids: native_plugin_package_ids(payload.get("materialized_packages")),
    })
}

fn report_json_from_stdout(stdout_lines: &[String]) -> Option<String> {
    let mut json = String::new();
    let mut collecting = false;
    let mut depth = 0usize;
    for line in stdout_lines {
        let trimmed = line.trim_start();
        if !collecting {
            if !trimmed.starts_with('{') {
                continue;
            }
            collecting = true;
        }
        depth = json_object_depth_after_line(line, depth);
        json.push_str(line);
        json.push('\n');
        if collecting && depth == 0 {
            return Some(json);
        }
    }
    None
}

fn json_object_depth_after_line(line: &str, current_depth: usize) -> usize {
    let mut depth = current_depth;
    let mut in_string = false;
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_string && character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match character {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

fn json_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn json_string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn native_plugin_package_ids(value: Option<&Value>) -> Option<Vec<String>> {
    value.and_then(Value::as_array).map(|packages| {
        packages
            .iter()
            .filter_map(|package| package.get("package_id"))
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect()
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
