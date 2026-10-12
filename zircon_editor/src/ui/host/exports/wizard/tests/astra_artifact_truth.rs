use super::super::{
    export_wizard_pipeline_plan, ExportWizardJobEvent, ExportWizardJobEventKind,
    ExportWizardJobState, ExportWizardPipelineOptions, ExportWizardPipelinePlan,
};
use super::*;

fn plan() -> ExportWizardPipelinePlan {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "artifact-truth",
        "zircon-project.toml",
        "planned-export",
    );
    options.source_asset_manifest = Some("source-assets.json".into());
    options.host_executable = Some("host.exe".into());
    export_wizard_pipeline_plan(options)
}

#[test]
fn astra_m23_planned_paths_never_become_reported_results() {
    let plan = plan();
    for (status, kind) in [
        (
            ExportWizardJobStatus::Pending,
            ExportWizardJobEventKind::Created,
        ),
        (
            ExportWizardJobStatus::Running,
            ExportWizardJobEventKind::Started,
        ),
        (
            ExportWizardJobStatus::Cancelling,
            ExportWizardJobEventKind::StageStarted,
        ),
        (
            ExportWizardJobStatus::Cancelled,
            ExportWizardJobEventKind::Cancelled,
        ),
        (
            ExportWizardJobStatus::Failed,
            ExportWizardJobEventKind::Failed,
        ),
        (
            ExportWizardJobStatus::Finished,
            ExportWizardJobEventKind::Finished,
        ),
    ] {
        let mut view = ExportWizardPanelViewModel::from_plan("artifact-truth", &plan);
        let mut snapshot = ExportWizardJobState::new("artifact-truth", &plan).into_snapshot();
        snapshot.status = status;
        snapshot.fatal = status == ExportWizardJobStatus::Failed;
        view.apply_event(ExportWizardJobEvent {
            kind,
            snapshot,
            output_delta: None,
            coalesced_output_events: 0,
        });
        let rows = view.stage_rows();
        assert!(rows.iter().any(|row| !row.planned_artifacts.is_empty()));
        assert!(rows
            .iter()
            .all(|row| row.artifact_paths.is_empty() && row.report_path.is_none()));
        let state = export_wizard_panel_template_state(&view);
        assert!(state
            .slot(ExportWizardPanelSlotKind::ArtifactPaths)
            .unwrap()
            .entries
            .is_empty());
        assert!(state
            .slot(ExportWizardPanelSlotKind::ReportBody)
            .unwrap()
            .entries
            .iter()
            .all(|entry| entry.key != "report.pipeline_report"));
    }
}

#[test]
fn astra_m23_latest_announced_path_is_preserved_even_on_failure() {
    let plan = plan();
    let mut view = ExportWizardPanelViewModel::from_plan("reported-artifact", &plan);
    let mut snapshot = ExportWizardJobState::new("reported-artifact", &plan).into_snapshot();
    snapshot
        .progress
        .push_stdout_line("zircon_export stage=Report profile=artifact-truth");
    snapshot
        .progress
        .push_stdout_line("pipeline_report=first-reported.json");
    snapshot
        .progress
        .push_stdout_line("pipeline_report=latest-reported.json");
    snapshot.status = ExportWizardJobStatus::Failed;
    snapshot.fatal = true;
    view.apply_event(ExportWizardJobEvent {
        kind: ExportWizardJobEventKind::Failed,
        snapshot,
        output_delta: None,
        coalesced_output_events: 0,
    });
    let rows = view.stage_rows();
    let report = rows
        .iter()
        .find(|row| row.stage == ExportStage::Report)
        .unwrap();
    assert_eq!(report.report_path.as_deref(), Some("latest-reported.json"));
    assert!(report
        .planned_artifacts
        .iter()
        .all(|artifact| artifact.path != "latest-reported.json"));
    let artifacts = artifact_path_entries(&rows);
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].key, "artifact.report.pipeline_report");
    assert_eq!(artifacts[0].detail, "latest-reported.json");
    assert_eq!(
        pipeline_report_body_entry(report).unwrap().detail,
        "latest-reported.json"
    );
}

#[test]
fn astra_m23_report_body_does_not_treat_stdout_json_as_report_artifact() {
    let plan = plan();
    let mut view = ExportWizardPanelViewModel::from_plan("stdout-only-report", &plan);
    let mut snapshot = ExportWizardJobState::new("stdout-only-report", &plan).into_snapshot();
    snapshot
        .progress
        .push_stdout_line("zircon_export stage=Report profile=artifact-truth");
    snapshot
        .progress
        .push_stdout_line(r#"{"export_plan":{"strategies":["Pack"]}}"#);
    snapshot.status = ExportWizardJobStatus::Finished;
    view.apply_event(ExportWizardJobEvent {
        kind: ExportWizardJobEventKind::Finished,
        snapshot,
        output_delta: None,
        coalesced_output_events: 0,
    });

    let panel = export_wizard_panel_template_state(&view);
    let report_entries = &panel
        .slot(ExportWizardPanelSlotKind::ReportBody)
        .unwrap()
        .entries;
    assert!(report_entries
        .iter()
        .all(|entry| !entry.key.starts_with("report.export_plan.")));
    assert!(report_entries
        .iter()
        .all(|entry| entry.key != "report.pipeline_report"));
}
