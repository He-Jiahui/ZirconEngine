use std::error::Error as _;
use std::io;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::core::jobs::{JobError, JobSubmitError};
use crate::ui::host::export_process_support::ExportProcessError;
use zircon_runtime_interface::export::ExportStage;

use super::super::*;
use super::support::*;

#[test]
fn export_wizard_job_state_finishes_from_successful_pipeline_execution() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
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
    assert_eq!(snapshot.current_stage, Some(ExportStage::Report));
    assert_eq!(snapshot.stages.len(), ExportStage::ALL.len());
}

#[test]
fn export_wizard_job_state_exposes_plan_diagnostic_failure_without_starting() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
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
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::for_test_profile(
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
    let mut options = ExportWizardPipelineOptions::for_test_profile(
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
    assert_eq!(snapshot.current_stage, Some(ExportStage::Report));
    assert_eq!(snapshot.stages.len(), ExportStage::ALL.len());
    assert_eq!(
        runner.seen_stages,
        ExportStage::ALL.to_vec(),
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
        ExportStage::ALL.len()
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == ExportWizardJobEventKind::StageFinished)
            .count(),
        ExportStage::ALL.len()
    );
}

#[test]
fn export_wizard_job_runner_stops_after_fatal_stage_event() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
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
    assert_eq!(snapshot.current_stage, Some(ExportStage::Validate));
    assert_eq!(snapshot.stages.len(), 1);
    assert_eq!(runner.seen_stages, vec![ExportStage::Validate]);
    assert_eq!(
        events.last().map(|event| event.kind),
        Some(ExportWizardJobEventKind::Failed)
    );
}

#[test]
fn export_wizard_job_runner_cancels_after_stage_boundary() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
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
    assert_eq!(snapshot.current_stage, Some(ExportStage::Validate));
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
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);

    let jobs = editor_jobs();
    let controller = ExportWizardJobController::submit(
        &jobs,
        "export-controller-success",
        plan,
        StubRunner::default(),
    )
    .expect("controller should submit to editor jobs");
    assert_eq!(controller.job_id(), "export-controller-success");

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
    let snapshot = controller.finish().result.expect("worker should finish");
    assert_eq!(snapshot.status, ExportWizardJobStatus::Finished);
    assert_eq!(snapshot.stages.len(), ExportStage::ALL.len());
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
fn export_wizard_job_controller_preserves_typed_submit_error() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let jobs = editor_jobs();

    let result = ExportWizardJobController::submit(&jobs, "", plan, StubRunner::default());

    assert!(matches!(result, Err(JobSubmitError::EmptyLabel)));
}

#[test]
fn export_wizard_job_controller_handle_requests_stage_boundary_cancel() {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
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
    let jobs = editor_jobs();
    let controller =
        ExportWizardJobController::submit(&jobs, "export-controller-cancel", plan, runner)
            .expect("controller should submit to editor jobs");

    assert_eq!(
        stage_started_receiver
            .recv()
            .expect("stage should start before cancel"),
        ExportStage::Validate
    );
    controller.request_cancel();
    assert!(controller.is_cancel_requested());
    release_stage_sender
        .send(())
        .expect("release first stage after cancel");

    let completion = controller.finish();
    assert_eq!(completion.result, Err(JobError::Cancelled));
    let snapshot = &completion
        .events
        .last()
        .expect("direct finish should retain the cancelled business event")
        .snapshot;
    assert_eq!(snapshot.status, ExportWizardJobStatus::Cancelled);
    assert_eq!(snapshot.stages.len(), 1);
    assert!(!snapshot.stages[0].stdout_lines.is_empty());
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("cancelled after Validate finished")));
}

#[test]
fn export_wizard_job_controller_maps_business_failure_to_typed_ticket_error() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let runner = StubRunner::with_execution(ExportWizardCommandExecution {
        exit_code: Some(7),
        stdout_lines: Vec::new(),
        stderr_lines: vec!["validate business failure".to_string()],
    });
    let jobs = editor_jobs();
    let controller =
        ExportWizardJobController::submit(&jobs, "export-controller-failed", plan, runner)
            .expect("failed business job should submit");

    let completion = controller.finish();
    let error = completion
        .result
        .expect_err("non-zero export stage must fail the editor job");
    assert!(matches!(
        error.downcast_ref::<EditorExportBuildError>(),
        Some(EditorExportBuildError::WizardStageFailed {
            stage: ExportStage::Validate,
            exit_code: Some(7),
        })
    ));
    let snapshot = &completion
        .events
        .last()
        .expect("direct finish should retain the failed business event")
        .snapshot;
    assert_eq!(snapshot.status, ExportWizardJobStatus::Failed);
    assert_eq!(snapshot.stages.len(), 1);
    assert!(snapshot.stages[0]
        .stderr_lines
        .iter()
        .any(|line| line.contains("validate business failure")));
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("validate business failure")));
}

#[test]
fn export_wizard_job_controller_preserves_runner_io_source_through_ticket() {
    struct IoFailureRunner;

    impl ExportWizardCommandRunner for IoFailureRunner {
        fn run(
            &mut self,
            _command: &ExportWizardPipelineStageCommand,
        ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
            Err(EditorExportBuildError::Process(ExportProcessError::io(
                "failed to start test export process",
                "typed wizard runner",
                None,
                None,
                io::Error::new(io::ErrorKind::PermissionDenied, "source marker"),
            )))
        }
    }

    let plan = export_wizard_pipeline_plan(ready_export_options());
    let controller = ExportWizardJobController::submit(
        &editor_jobs(),
        "export-controller-typed-source",
        plan,
        IoFailureRunner,
    )
    .expect("typed failure job should submit");

    let error = controller
        .finish()
        .result
        .expect_err("runner IO failure must reach the ticket");
    let export_error = error
        .downcast_ref::<EditorExportBuildError>()
        .expect("ticket must retain the editor export error");
    let EditorExportBuildError::Process(process_error) = export_error else {
        panic!("ticket retained the wrong export error variant: {export_error}");
    };
    let io_error = process_error
        .source()
        .and_then(|source| source.downcast_ref::<io::Error>())
        .expect("export process error must retain its IO source");
    assert_eq!(io_error.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(io_error.to_string(), "source marker");
}

#[test]
fn export_wizard_job_controller_preserves_failure_observed_during_cancellation() {
    struct CancelThenFailRunner {
        started: std::sync::mpsc::Sender<()>,
        release: std::sync::mpsc::Receiver<()>,
    }

    impl ExportWizardCommandRunner for CancelThenFailRunner {
        fn run(
            &mut self,
            _command: &ExportWizardPipelineStageCommand,
        ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
            unreachable!("cancel-aware entry should be used")
        }

        fn run_with_output_and_cancel(
            &mut self,
            _command: &ExportWizardPipelineStageCommand,
            _emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
            should_cancel: &mut (dyn FnMut() -> bool + Send),
        ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
            self.started
                .send(())
                .expect("test should observe the runner after scheduling");
            self.release
                .recv_timeout(Duration::from_secs(5))
                .expect("test should release the active runner");
            assert!(should_cancel(), "active runner should observe cancellation");
            Err(EditorExportBuildError::Process(ExportProcessError::io(
                "failed while cancelling test export process",
                "typed cancellation race",
                None,
                None,
                io::Error::new(io::ErrorKind::BrokenPipe, "cancellation source marker"),
            )))
        }
    }

    let plan = export_wizard_pipeline_plan(ready_export_options());
    let (started_sender, started_receiver) = channel();
    let (release_sender, release_receiver) = channel();
    let controller = ExportWizardJobController::submit(
        &editor_jobs(),
        "export-controller-cancel-failure",
        plan,
        CancelThenFailRunner {
            started: started_sender,
            release: release_receiver,
        },
    )
    .expect("cancellation failure job should submit");
    started_receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("runner should start before cancellation is requested");
    controller.request_cancel();
    release_sender
        .send(())
        .expect("active runner should be released after cancellation");

    let error = controller
        .finish()
        .result
        .expect_err("sourceful cancellation race must fail instead of becoming cancellation");
    assert!(matches!(
        error.downcast_ref::<EditorExportBuildError>(),
        Some(EditorExportBuildError::Process(_))
    ));
}
