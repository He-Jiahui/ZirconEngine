use std::sync::mpsc::channel;
use std::time::Duration;

use zircon_runtime::plugin::ExportPipelineStage;

use super::super::*;
use super::support::*;

#[test]
fn export_wizard_job_state_finishes_from_successful_pipeline_execution() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::default();
    let execution = execute_export_wizard_pipeline(&plan, &mut runner);
    let mut job = ExportWizardJobState::new("export-1", &plan);

    job.begin();
    job.finish_from_pipeline(execution);

    let snapshot = job.snapshot();
    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);
    assert!(snapshot.is_terminal());
    assert!(!snapshot.fatal);
    assert_eq!(snapshot.current_stage, Some(ExportPipelineStage::Report));
    assert_eq!(snapshot.stages.len(), export_pipeline_stages().len());
}

#[test]
fn export_wizard_job_state_exposes_plan_diagnostic_failure_without_starting() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.previous_pack = Some("D:\\old\\game.zrpack".to_string());
    let plan = export_wizard_pipeline_plan(options);

    let job = ExportWizardJobState::new("export-invalid", &plan);

    let snapshot = job.snapshot();
    assert_eq!(snapshot.status, ExportWizardJobStatus::Failed);
    assert!(snapshot.fatal);
    assert!(snapshot.stages.is_empty());
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("previous_pack and delta_pack")));
}

#[test]
fn export_wizard_job_state_tracks_cancel_request_and_cancelled_terminal_state() {
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    ));
    let mut job = ExportWizardJobState::new("export-cancel", &plan);

    job.begin();
    job.request_cancel();
    assert_eq!(job.snapshot().status, ExportWizardJobStatus::Cancelling);
    assert!(job.snapshot().cancel_requested);

    job.mark_cancelled("cancelled by user");

    let snapshot = job.snapshot();
    assert_eq!(snapshot.status, ExportWizardJobStatus::Cancelled);
    assert!(snapshot.is_terminal());
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic == "cancelled by user"));
}

#[test]
fn export_wizard_job_runner_emits_successful_snapshot_events() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::default();
    let mut events = Vec::new();

    let snapshot = run_export_wizard_job(
        "export-runner-success",
        &plan,
        &mut runner,
        &ExportWizardNeverCancel,
        &mut |event| events.push(event),
    );

    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);
    assert_eq!(snapshot.current_stage, Some(ExportPipelineStage::Report));
    assert_eq!(snapshot.stages.len(), export_pipeline_stages().len());
    assert_eq!(
        runner.seen_stages,
        export_pipeline_stages().to_vec(),
        "runner should execute every planned stage"
    );
    assert_eq!(
        events.first().map(|event| event.kind),
        Some(ExportWizardJobEventKind::Created)
    );
    assert_eq!(
        events.last().map(|event| event.kind),
        Some(ExportWizardJobEventKind::Finished)
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == ExportWizardJobEventKind::StageStarted)
            .count(),
        export_pipeline_stages().len()
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == ExportWizardJobEventKind::StageFinished)
            .count(),
        export_pipeline_stages().len()
    );
}

#[test]
fn export_wizard_job_runner_stops_after_fatal_stage_event() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::with_execution(ExportWizardCommandExecution {
        exit_code: Some(2),
        stdout_lines: vec!["zircon_export stage=Validate profile=windows-release".to_string()],
        stderr_lines: vec!["validate failed".to_string()],
    });
    let mut events = Vec::new();

    let snapshot = run_export_wizard_job(
        "export-runner-failure",
        &plan,
        &mut runner,
        &ExportWizardNeverCancel,
        &mut |event| events.push(event),
    );

    assert_eq!(snapshot.status, ExportWizardJobStatus::Failed);
    assert_eq!(snapshot.current_stage, Some(ExportPipelineStage::Validate));
    assert_eq!(snapshot.stages.len(), 1);
    assert_eq!(runner.seen_stages, vec![ExportPipelineStage::Validate]);
    assert_eq!(
        events.last().map(|event| event.kind),
        Some(ExportWizardJobEventKind::Failed)
    );
}

#[test]
fn export_wizard_job_runner_cancels_after_stage_boundary() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let cancel_signal = CancelAfterRuns::new(1);
    let mut runner = ObservingRunner {
        inner: StubRunner::default(),
        observed_runs: cancel_signal.observer(),
    };
    let mut events = Vec::new();

    let snapshot = run_export_wizard_job(
        "export-runner-cancel",
        &plan,
        &mut runner,
        &cancel_signal,
        &mut |event| events.push(event),
    );

    assert_eq!(snapshot.status, ExportWizardJobStatus::Cancelled);
    assert!(snapshot.cancel_requested);
    assert_eq!(snapshot.current_stage, Some(ExportPipelineStage::Validate));
    assert_eq!(snapshot.stages.len(), 1);
    assert_eq!(
        events.last().map(|event| event.kind),
        Some(ExportWizardJobEventKind::Cancelled)
    );
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("cancelled after Validate finished")));
}

#[test]
fn export_wizard_job_controller_streams_events_and_finishes_worker() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);

    let controller =
        ExportWizardJobController::spawn("export-controller-success", plan, StubRunner::default());
    assert_eq!(controller.handle().job_id, "export-controller-success");

    let mut event_kinds = Vec::new();
    loop {
        let event = controller
            .events()
            .recv_timeout(Duration::from_secs(1))
            .expect("controller should stream job events");
        event_kinds.push(event.kind);
        if event.kind == ExportWizardJobEventKind::Finished {
            break;
        }
    }
    let snapshot = controller.finish().expect("worker should finish");
    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);
    assert_eq!(snapshot.stages.len(), export_pipeline_stages().len());
    assert_eq!(
        event_kinds.first().copied(),
        Some(ExportWizardJobEventKind::Created)
    );
    assert_eq!(
        event_kinds.last().copied(),
        Some(ExportWizardJobEventKind::Finished)
    );
}

#[test]
fn export_wizard_job_controller_handle_requests_stage_boundary_cancel() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let (stage_started_sender, stage_started_receiver) = channel();
    let (release_stage_sender, release_stage_receiver) = channel();
    let runner = BlockingRunner::new(stage_started_sender, release_stage_receiver);
    let controller = ExportWizardJobController::spawn("export-controller-cancel", plan, runner);

    assert_eq!(
        stage_started_receiver
            .recv()
            .expect("stage should start before cancel"),
        ExportPipelineStage::Validate
    );
    controller.request_cancel();
    assert!(controller.handle().is_cancel_requested());
    release_stage_sender
        .send(())
        .expect("release first stage after cancel");

    let snapshot = controller.finish().expect("worker should finish");
    assert_eq!(snapshot.status, ExportWizardJobStatus::Cancelled);
    assert_eq!(snapshot.stages.len(), 1);
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("cancelled after Validate finished")));
}
