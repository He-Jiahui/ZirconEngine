use std::sync::mpsc::channel;
use std::time::Duration;

use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::template_runtime::EditorUiHostRuntime;
use zircon_runtime::plugin::ExportPipelineStage;

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
    let missing_plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    ));
    let mut session = ExportWizardPanelSession::new("export-panel-missing", missing_plan);

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
    let missing_plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    ));
    let mut session = ExportWizardPanelSession::new("export-panel-missing", missing_plan);

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
    let mut session = ExportWizardPanelSession::new("export-panel-call", plan);
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
    let mut session = ExportWizardPanelSession::new("export-panel-cancel", plan);

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
        ExportPipelineStage::Validate
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

    let snapshot = session
        .finish_job()
        .expect("panel job should join")
        .expect("panel job should have been active");
    assert_eq!(snapshot.status, ExportWizardJobStatus::Cancelled);
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
    let mut session = ExportWizardPanelSession::new("export-panel-finished", plan);

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
        if update.snapshot.is_terminal() {
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
