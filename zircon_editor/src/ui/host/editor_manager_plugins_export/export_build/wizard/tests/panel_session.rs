use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::core::jobs::{test_job_system_with_limits, EditorJobLimits, JobCategory, JobError};
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::template_runtime::EditorUiHostRuntime;
use zircon_runtime_interface::export::ExportStage;

use super::super::*;
use super::support::*;

#[test]
fn export_wizard_panel_bindings_project_template_button_events() {
    let mut runtime = EditorUiHostRuntime::default();
    register_export_wizard_panel_template(&mut runtime, desktop_export_panel_template_path())
        .expect("desktop export panel template and bindings should register");

    let projection =
        project_export_wizard_panel(&runtime).expect("desktop export panel should project");

    assert_eq!(projection.document_id, EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID);
    assert_eq!(
        projection.bindings.len(),
        export_wizard_panel_bindings().len()
    );
    for expected in export_wizard_panel_bindings() {
        let node = find_projection_node(&projection.root, expected.control_id)
            .unwrap_or_else(|| panic!("{} node should project", expected.control_id));
        assert!(
            node.binding_ids
                .iter()
                .any(|binding_id| binding_id == expected.binding_id),
            "{} should carry binding {}",
            expected.control_id,
            expected.binding_id
        );

        let projected = projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == expected.binding_id)
            .unwrap_or_else(|| panic!("{} should project", expected.binding_id));
        assert_eq!(projected.binding.path().view_id, EXPORT_WIZARD_VIEW_ID);
        assert_eq!(projected.binding.path().control_id, expected.control_id);
        assert_eq!(projected.binding.path().event_kind, expected.event_kind);
        assert_eq!(
            export_wizard_panel_action_for_control(expected.control_id, expected.event_kind),
            Some(expected.action)
        );

        let EditorUiBindingPayload::Custom(call) = projected.binding.payload() else {
            panic!(
                "{} should use custom export wizard call",
                expected.binding_id
            );
        };
        assert_eq!(call.symbol, EXPORT_WIZARD_BINDING_SYMBOL);
        assert_eq!(
            ExportWizardPanelAction::from_call(call),
            Some(expected.action)
        );
        assert_eq!(
            call.argument(1).and_then(|value| value.as_str()),
            Some(expected.control_id)
        );
    }
}

#[test]
fn export_wizard_panel_session_rejects_unready_start_until_plan_regenerates() {
    let missing_plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    ));
    let mut session =
        ExportWizardPanelSession::new(editor_jobs(), "export-panel-missing", missing_plan);

    assert!(!session.view_model().controls().can_start);
    assert_eq!(
        session.start_with_runner(StubRunner::default()),
        Err(ExportWizardPanelSessionError::ActionDisabled {
            action: ExportWizardPanelAction::Start,
            reason: "plan is not ready",
        })
    );

    session
        .regenerate_plan("export-panel-ready", ready_export_options())
        .expect("ready options should replace inactive plan");

    assert!(session.plan().is_ready());
    assert!(session.view_model().controls().can_start);
}

#[test]
fn export_wizard_panel_session_dispatches_generate_plan_request() {
    let missing_plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    ));
    let mut session =
        ExportWizardPanelSession::new(editor_jobs(), "export-panel-missing", missing_plan);

    let update = session
        .handle_request(ExportWizardPanelRequest::generate_plan(
            "export-panel-ready",
            ready_export_options(),
        ))
        .expect("generate plan request should replace inactive plan");

    assert_eq!(update.action, ExportWizardPanelAction::GeneratePlan);
    assert_eq!(update.events_drained, 0);
    assert_eq!(update.active_job_id, None);
    assert_eq!(update.snapshot.job_id, "export-panel-ready");
    assert_eq!(update.snapshot.status, ExportWizardJobStatus::Pending);
    assert!(session.plan().is_ready());
    assert!(session.view_model().controls().can_start);
}

#[test]
fn export_wizard_panel_session_rejects_generate_plan_call_without_options() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let mut session = ExportWizardPanelSession::new(editor_jobs(), "export-panel-call", plan);
    let call = export_wizard_panel_action_call(
        ExportWizardPanelAction::GeneratePlan,
        DESKTOP_EXPORT_GENERATE_PLAN_BUTTON,
    );

    assert_eq!(
        session.handle_action_call(&call),
        Err(ExportWizardPanelSessionError::ActionDisabled {
            action: ExportWizardPanelAction::GeneratePlan,
            reason: "generate_plan requires explicit pipeline options",
        })
    );
}

#[test]
fn export_wizard_panel_session_starts_polls_and_cancels_job() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let (stage_started_sender, stage_started_receiver) = channel();
    let (release_stage_sender, release_stage_receiver) = channel();
    let runner = BlockingRunner::new(stage_started_sender, release_stage_receiver);
    let mut session = ExportWizardPanelSession::new(editor_jobs(), "export-panel-cancel", plan);

    let start_update = session
        .handle_start_request_with_runner(runner)
        .expect("ready panel session should start");
    assert_eq!(start_update.action, ExportWizardPanelAction::Start);
    assert_eq!(
        start_update.active_job_id.as_deref(),
        Some("export-panel-cancel")
    );
    assert_eq!(session.active_job_id(), Some("export-panel-cancel"));
    assert_eq!(
        stage_started_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("first stage should start before polling"),
        ExportStage::Validate
    );

    let poll_update = session
        .handle_request(ExportWizardPanelRequest::Poll)
        .expect("poll request should drain events");
    assert_eq!(poll_update.action, ExportWizardPanelAction::Poll);
    assert!(poll_update.events_drained >= 3);
    assert_eq!(
        session.view_model().latest_event_kind(),
        Some(ExportWizardJobEventKind::StageStarted)
    );
    assert!(session.view_model().controls().can_cancel);

    let cancel_update = session
        .handle_request(ExportWizardPanelRequest::Cancel)
        .expect("active panel session should accept cancel");
    assert_eq!(cancel_update.action, ExportWizardPanelAction::Cancel);
    release_stage_sender
        .send(())
        .expect("release first stage after cancel");

    assert_eq!(
        poll_until_error(&mut session),
        ExportWizardPanelSessionError::Job(JobError::Cancelled)
    );
    assert_eq!(
        session.view_model().snapshot().status,
        ExportWizardJobStatus::Cancelled
    );
    assert_eq!(session.view_model().snapshot().stages.len(), 1);
    assert!(!session.view_model().snapshot().stages[0]
        .stdout_lines
        .is_empty());
    assert!(session
        .view_model()
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("cancelled after Validate finished")));
    assert_eq!(session.active_job_id(), None);
    assert_eq!(
        session.view_model().latest_event_kind(),
        Some(ExportWizardJobEventKind::Cancelled)
    );
    assert!(session.view_model().controls().can_close);
    assert!(!session.view_model().controls().can_cancel);
}

#[test]
fn export_wizard_panel_session_poll_finishes_terminal_job() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let mut session = ExportWizardPanelSession::new(editor_jobs(), "export-panel-finished", plan);

    let start_update = session
        .handle_start_request_with_runner(StubRunner::default())
        .expect("ready panel session should start");
    assert_eq!(
        start_update.active_job_id.as_deref(),
        Some("export-panel-finished")
    );

    let mut terminal_update = None;
    for _ in 0..20 {
        let update = session
            .handle_request(ExportWizardPanelRequest::Poll)
            .expect("poll request should drain events and finish terminal jobs");
        if update.snapshot.is_terminal() && update.active_job_id.is_none() {
            terminal_update = Some(update);
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let terminal_update = terminal_update.expect("job should reach a terminal snapshot");

    assert_eq!(terminal_update.action, ExportWizardPanelAction::Poll);
    assert_eq!(
        terminal_update.snapshot.status,
        ExportWizardJobStatus::Finished
    );
    assert_eq!(terminal_update.active_job_id, None);
    assert_eq!(session.active_job_id(), None);
    assert_eq!(
        session.view_model().latest_event_kind(),
        Some(ExportWizardJobEventKind::Finished)
    );
    assert!(session.view_model().controls().can_close);
    assert!(!session.view_model().controls().can_cancel);

    let generate_update = session
        .handle_request(ExportWizardPanelRequest::generate_plan(
            "export-panel-next",
            ready_export_options(),
        ))
        .expect("terminal poll should clear the old controller");
    assert_eq!(generate_update.snapshot.job_id, "export-panel-next");
    assert!(session.view_model().controls().can_start);
}

#[test]
fn export_wizard_panel_session_poll_preserves_typed_job_failure_and_clears_active_job() {
    struct PanicRunner;

    impl ExportWizardCommandRunner for PanicRunner {
        fn run(
            &mut self,
            _command: &ExportWizardPipelineStageCommand,
        ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
            panic!("typed export runner failure")
        }
    }

    let plan = export_wizard_pipeline_plan(ready_export_options());
    let mut session = ExportWizardPanelSession::new(editor_jobs(), "export-panel-panic", plan);
    session
        .start_with_runner(PanicRunner)
        .expect("panic runner job should submit");

    let error = poll_until_error(&mut session);
    assert!(matches!(
        error,
        ExportWizardPanelSessionError::Job(JobError::Panicked(message))
            if message.contains("typed export runner failure")
    ));
    assert_eq!(session.active_job_id(), None);
    assert!(session.view_model().snapshot().is_terminal());
    assert_eq!(
        session.view_model().snapshot().status,
        ExportWizardJobStatus::Failed
    );
    assert!(session
        .view_model()
        .snapshot()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("typed export runner failure")));
    assert!(session.view_model().controls().can_close);
    assert!(!session.view_model().controls().can_cancel);
}

#[test]
fn export_wizard_panel_session_poll_retains_final_business_failure_event() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let runner = StubRunner::with_execution(ExportWizardCommandExecution {
        exit_code: Some(7),
        stdout_lines: Vec::new(),
        stderr_lines: vec!["poll business failure".to_string()],
    });
    let mut session = ExportWizardPanelSession::new(editor_jobs(), "export-poll-failed", plan);
    session
        .start_with_runner(runner)
        .expect("failed business runner should submit");

    let error = poll_until_error(&mut session);
    let ExportWizardPanelSessionError::Job(job_error) = &error else {
        panic!("panel returned the wrong failure type: {error:?}");
    };
    assert!(matches!(
        job_error.downcast_ref::<EditorExportBuildError>(),
        Some(EditorExportBuildError::WizardStageFailed {
            stage: ExportStage::Validate,
            exit_code: Some(7),
        })
    ));
    let snapshot = session.view_model().snapshot();
    assert_eq!(snapshot.status, ExportWizardJobStatus::Failed);
    assert_eq!(snapshot.stages.len(), 1);
    assert!(snapshot.stages[0]
        .stderr_lines
        .iter()
        .any(|line| line.contains("poll business failure")));
    assert!(snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("poll business failure")));
    assert_eq!(
        session.view_model().latest_event_kind(),
        Some(ExportWizardJobEventKind::Failed)
    );
    assert_eq!(session.active_job_id(), None);
}

#[test]
fn export_wizard_panel_session_direct_finish_projects_typed_job_failure() {
    struct PanicRunner;

    impl ExportWizardCommandRunner for PanicRunner {
        fn run(
            &mut self,
            _command: &ExportWizardPipelineStageCommand,
        ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
            panic!("direct finish export runner failure")
        }
    }

    let plan = export_wizard_pipeline_plan(ready_export_options());
    let mut session = ExportWizardPanelSession::new(editor_jobs(), "export-finish-panic", plan);
    session
        .start_with_runner(PanicRunner)
        .expect("panic runner job should submit");

    assert!(matches!(
        session.finish_job(),
        Err(ExportWizardPanelSessionError::Job(JobError::Panicked(message)))
            if message.contains("direct finish export runner failure")
    ));
    assert_eq!(session.active_job_id(), None);
    assert_eq!(
        session.view_model().snapshot().status,
        ExportWizardJobStatus::Failed
    );
    assert!(session.view_model().snapshot().is_terminal());
    assert!(session.view_model().controls().can_close);
    assert!(!session.view_model().controls().can_cancel);
}

#[test]
fn export_wizard_panel_session_poll_finishes_queued_prestart_cancellation() {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let (stage_started_sender, stage_started_receiver) = channel();
    let (release_stage_sender, release_stage_receiver) = channel();
    let first = ExportWizardJobController::submit(
        &jobs,
        "export-quota-gate",
        plan.clone(),
        BlockingRunner::new(stage_started_sender, release_stage_receiver),
    )
    .expect("quota gate should submit");
    stage_started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("quota gate should occupy the export category");

    let mut session = ExportWizardPanelSession::new(jobs.clone(), "export-queued-cancel", plan);
    session
        .start_with_runner(StubRunner::default())
        .expect("queued export should submit");
    let cancellation = match session.handle_request(ExportWizardPanelRequest::Cancel) {
        Ok(_) => poll_until_error(&mut session),
        Err(error) => error,
    };

    assert_eq!(
        cancellation,
        ExportWizardPanelSessionError::Job(JobError::Cancelled)
    );
    assert_eq!(session.active_job_id(), None);
    assert!(session.view_model().snapshot().is_terminal());
    assert_eq!(
        session.view_model().snapshot().status,
        ExportWizardJobStatus::Cancelled
    );
    assert!(session.view_model().snapshot().cancel_requested);
    assert!(session.view_model().controls().can_close);
    assert!(!session.view_model().controls().can_cancel);

    release_stage_sender
        .send(())
        .expect("quota gate should be released");
    // This fixture gates only the first stage. Disconnect the receiver so later
    // stages do not wait for release messages that the test never intends to send.
    drop(release_stage_sender);
    first.finish().result.expect("quota gate should finish");
}

fn poll_until_error(session: &mut ExportWizardPanelSession) -> ExportWizardPanelSessionError {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match session.handle_request(ExportWizardPanelRequest::Poll) {
            Ok(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(1)),
            Ok(_) => panic!("poll missed the five second terminal error deadline"),
            Err(error) => return error,
        }
    }
}
