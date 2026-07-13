use std::sync::mpsc::channel;

use zircon_runtime_interface::export::ExportStage;

use super::super::*;
use super::support::*;

#[test]
fn export_wizard_view_model_projects_plan_stage_rows_and_controls() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);

    let view_model = ExportWizardPanelViewModel::from_plan("export-view-model-ready", &plan);
    let controls = view_model.controls();
    assert!(controls.plan_ready);
    assert!(controls.can_start);
    assert!(!controls.can_cancel);
    assert!(controls.can_close);
    assert_eq!(controls.status, ExportWizardJobStatus::Pending);
    assert_eq!(controls.missing_input_count, 0);

    let rows = view_model.stage_rows();
    assert_eq!(rows.len(), ExportStage::ALL.len());
    let validate = rows
        .iter()
        .find(|row| row.stage == ExportStage::Validate)
        .expect("Validate row should exist");
    assert_eq!(validate.stage_id, "validate");
    assert_eq!(validate.label, "Validate");
    assert_eq!(validate.progress_kind, ExportStageProgressKind::Pending);
    assert_eq!(
        validate.report_path.as_deref(),
        Some("D:\\zircon-export\\stages\\validate\\report.json")
    );
    assert!(validate.missing_inputs.is_empty());
}

#[test]
fn export_wizard_view_model_reports_missing_inputs_before_start() {
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    ));

    let view_model = ExportWizardPanelViewModel::from_plan("export-view-model-missing", &plan);
    let controls = view_model.controls();
    assert!(!controls.plan_ready);
    assert!(!controls.can_start);
    assert_eq!(controls.missing_input_count, 2);

    let rows = view_model.stage_rows();
    let cook_assets = rows
        .iter()
        .find(|row| row.stage == ExportStage::CookAssets)
        .expect("CookAssets row should exist");
    assert_eq!(cook_assets.missing_inputs, vec!["source_asset_manifest"]);
    let platform_bundle = rows
        .iter()
        .find(|row| row.stage == ExportStage::PlatformBundle)
        .expect("PlatformBundle row should exist");
    assert_eq!(platform_bundle.missing_inputs, vec!["host_executable"]);
}

#[test]
fn export_wizard_view_model_drains_job_events_into_terminal_rows() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::default();
    let mut emitted_events = Vec::new();
    let mut view_model = ExportWizardPanelViewModel::from_plan("export-view-model-finished", &plan);

    let snapshot = run_export_wizard_job(
        "export-view-model-finished",
        &plan,
        &mut runner,
        &ExportWizardNeverCancel,
        &mut |event| emitted_events.push(event),
    );
    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);

    let expected_event_count = emitted_events.len();
    let (sender, receiver) = channel();
    for event in emitted_events {
        sender.send(event).expect("event should be queued");
    }
    drop(sender);

    assert_eq!(view_model.drain_events(&receiver), expected_event_count);
    assert_eq!(
        view_model.latest_event_kind(),
        Some(ExportWizardJobEventKind::Finished)
    );
    assert_eq!(view_model.event_count(), expected_event_count);

    let controls = view_model.controls();
    assert_eq!(controls.status, ExportWizardJobStatus::Finished);
    assert!(!controls.can_start);
    assert!(!controls.can_cancel);
    assert!(controls.can_close);

    let rows = view_model.stage_rows();
    assert!(rows
        .iter()
        .all(|row| row.progress_kind == ExportStageProgressKind::Passed));
    assert!(rows.iter().all(|row| {
        row.report_path
            .as_deref()
            .is_some_and(|path| path.ends_with("report.json"))
    }));
}

#[test]
fn export_wizard_panel_template_state_projects_template_slots() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let view_model = ExportWizardPanelViewModel::from_plan("export-panel-slots", &plan);

    let state = export_wizard_panel_template_state(&view_model);

    assert!(state.controls.can_start);
    assert_eq!(state.control_bindings.len(), 3);
    assert_eq!(
        state
            .control(DESKTOP_EXPORT_GENERATE_PLAN_BUTTON)
            .expect("generate plan button state should exist")
            .enabled,
        true
    );
    assert_eq!(
        state
            .control(DESKTOP_EXPORT_START_BUTTON)
            .expect("start button state should exist")
            .enabled,
        true
    );
    assert_eq!(
        state
            .control(DESKTOP_EXPORT_CANCEL_BUTTON)
            .expect("cancel button state should exist")
            .enabled,
        false
    );
    assert_eq!(state.slots.len(), 5);
    assert_eq!(
        state
            .slot(ExportWizardPanelSlotKind::StageRows)
            .expect("stage rows slot should exist")
            .control_id,
        DESKTOP_EXPORT_STAGE_ROWS_SLOT
    );
    assert_eq!(
        state
            .slot(ExportWizardPanelSlotKind::StageRows)
            .expect("stage rows slot should exist")
            .entries
            .len(),
        ExportStage::ALL.len()
    );
    assert!(state
        .slot(ExportWizardPanelSlotKind::MissingInputs)
        .expect("missing inputs slot should exist")
        .entries
        .is_empty());
    assert!(state
        .slot(ExportWizardPanelSlotKind::ArtifactPaths)
        .expect("artifact paths slot should exist")
        .entries
        .iter()
        .any(|entry| entry.key == "artifact.validate.report"
            && entry.detail.ends_with("stages\\validate\\report.json")));
    assert_eq!(
        state
            .slot(ExportWizardPanelSlotKind::ReportBody)
            .expect("report body slot should exist")
            .entries
            .first()
            .map(|entry| entry.label.as_str()),
        Some("Pending")
    );
}

#[test]
fn export_wizard_panel_template_state_reports_missing_inputs() {
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    ));
    let view_model = ExportWizardPanelViewModel::from_plan("export-panel-missing", &plan);

    let state = export_wizard_panel_template_state(&view_model);
    assert_eq!(
        state
            .control(DESKTOP_EXPORT_GENERATE_PLAN_BUTTON)
            .expect("generate plan button state should exist")
            .enabled,
        true
    );
    assert_eq!(
        state
            .control(DESKTOP_EXPORT_START_BUTTON)
            .expect("start button state should exist")
            .enabled,
        false
    );
    let missing_entries = &state
        .slot(ExportWizardPanelSlotKind::MissingInputs)
        .expect("missing inputs slot should exist")
        .entries;

    assert_eq!(missing_entries.len(), 2);
    assert!(missing_entries.iter().any(|entry| {
        entry.key == "missing.cook_assets"
            && entry.detail == "source_asset_manifest"
            && entry.severity == ExportWizardPanelEntrySeverity::Warning
    }));
    assert!(missing_entries.iter().any(|entry| {
        entry.key == "missing.platform_bundle"
            && entry.detail == "host_executable"
            && entry.severity == ExportWizardPanelEntrySeverity::Warning
    }));
}
