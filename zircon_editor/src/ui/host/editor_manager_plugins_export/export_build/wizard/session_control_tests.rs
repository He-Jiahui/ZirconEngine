use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use zircon_runtime_interface::{export::ExportStage, ui::dispatch::UiWindowId};

use crate::core::context::ToolSchedulerService;
use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::jobs::{test_job_system, JobError};
use crate::core::tools::ToolResourceKey;

use super::*;

struct FirstStageBlockingRunner {
    stage_started: Sender<ExportStage>,
    release_first_stage: Option<Receiver<()>>,
}

impl FirstStageBlockingRunner {
    fn new(stage_started: Sender<ExportStage>, release_first_stage: Receiver<()>) -> Self {
        Self {
            stage_started,
            release_first_stage: Some(release_first_stage),
        }
    }
}

impl ExportWizardCommandRunner for FirstStageBlockingRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        let _ = self.stage_started.send(command.stage);
        if let Some(release_first_stage) = self.release_first_stage.take() {
            let _ = release_first_stage.recv();
        }
        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines: vec![command.stdout_banner("windows-release")],
            stderr_lines: Vec::new(),
        })
    }
}

#[test]
fn export_wizard_panel_session_start_updates_controls_before_worker_poll() {
    let mut session = ready_session("export-panel-active-controls");
    let (_stage_started_sender, stage_started_receiver) = channel();
    let (release_stage_sender, release_stage_receiver) = channel();

    let update = session
        .handle_start_request_with_runner(FirstStageBlockingRunner::new(
            _stage_started_sender,
            release_stage_receiver,
        ))
        .expect("ready panel session should start");

    assert_eq!(update.snapshot.status, ExportWizardJobStatus::Running);
    assert_eq!(
        update.active_job_id.as_deref(),
        Some("export-panel-active-controls")
    );

    let controls = session.view_model().controls();
    assert!(!controls.can_start);
    assert!(controls.can_cancel);
    assert!(!controls.can_close);
    assert!(controls.show_progress);

    assert_eq!(
        stage_started_receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("first stage should still start"),
        ExportStage::Validate
    );
    release_stage_sender
        .send(())
        .expect("release first stage before joining");
    let _ = session
        .finish_job()
        .expect("blocking panel job should join cleanly");
}

#[test]
fn export_wizard_panel_session_cancel_disables_cancel_before_terminal_poll() {
    let mut session = ready_session("export-panel-cancelling-controls");
    let (_stage_started_sender, stage_started_receiver) = channel();
    let (release_stage_sender, release_stage_receiver) = channel();
    session
        .handle_start_request_with_runner(FirstStageBlockingRunner::new(
            _stage_started_sender,
            release_stage_receiver,
        ))
        .expect("ready panel session should start");
    stage_started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("first stage should start before cancellation");

    let update = session
        .handle_request(ExportWizardPanelRequest::Cancel)
        .expect("active panel session should accept cancel");

    assert_eq!(update.snapshot.status, ExportWizardJobStatus::Cancelling);
    assert!(update.snapshot.cancel_requested);
    let controls = session.view_model().controls();
    assert!(!controls.can_start);
    assert!(!controls.can_cancel);
    assert!(!controls.can_close);

    release_stage_sender
        .send(())
        .expect("release first stage after cancellation");
    assert_eq!(
        session.finish_job(),
        Err(ExportWizardPanelSessionError::Job(JobError::Cancelled))
    );
    assert_eq!(
        session.view_model().snapshot().status,
        ExportWizardJobStatus::Cancelled
    );
    assert!(session.view_model().controls().can_close);
}

#[test]
fn export_wizard_session_holds_modal_tool_until_terminal_completion() {
    let scheduler = ToolSchedulerService::new(SharedEditorMessageBus::default());
    let modal_resource = modal_resource("window.main");
    let mut session = ExportWizardPanelSession::new_with_tools(
        test_job_system(),
        "export-panel-modal-tool",
        export_wizard_pipeline_plan(ready_options()),
        scheduler.clone(),
        UiWindowId::new("window.main"),
    );
    let (_stage_started_sender, stage_started_receiver) = channel();
    let (release_stage_sender, release_stage_receiver) = channel();

    session
        .handle_start_request_with_runner(FirstStageBlockingRunner::new(
            _stage_started_sender,
            release_stage_receiver,
        ))
        .expect("ready panel session should start");
    stage_started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("first stage should start before checking the lease");
    let lease = scheduler
        .holder(&modal_resource)
        .expect("the active export session should hold the modal surface");
    assert!(lease
        .instance()
        .as_str()
        .starts_with("editor.export.wizard."));
    assert_eq!(lease.resources().as_slice(), [modal_resource.clone()]);

    release_stage_sender
        .send(())
        .expect("release first stage before joining");
    let _ = session.finish_job();
    assert_eq!(scheduler.holder(&modal_resource), None);
}

#[test]
fn replacing_export_plan_preserves_the_session_scoped_tool_identity() {
    let scheduler = ToolSchedulerService::new(SharedEditorMessageBus::default());
    let modal_resource = modal_resource("window.main");
    let mut session = ExportWizardPanelSession::new_with_tools(
        test_job_system(),
        "export-panel-old-job",
        export_wizard_pipeline_plan(ready_options()),
        scheduler.clone(),
        UiWindowId::new("window.main"),
    );
    let tool = session
        .tool_id_for_test()
        .expect("the export session should allocate a tool identity")
        .clone();
    session
        .regenerate_plan("export-panel-new-job", ready_options())
        .expect("inactive export plan should be replaceable");
    assert_eq!(session.tool_id_for_test(), Ok(&tool));

    let (_stage_started_sender, stage_started_receiver) = channel();
    let (release_stage_sender, release_stage_receiver) = channel();
    session
        .handle_start_request_with_runner(FirstStageBlockingRunner::new(
            _stage_started_sender,
            release_stage_receiver,
        ))
        .expect("regenerated export plan should start");
    stage_started_receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("first stage should start before checking the lease");

    assert_eq!(
        scheduler
            .holder(&modal_resource)
            .map(|lease| lease.instance().clone()),
        Some(tool)
    );
    release_stage_sender
        .send(())
        .expect("release first stage before joining");
    let _ = session.finish_job();
    assert_eq!(scheduler.holder(&modal_resource), None);
}

fn ready_session(job_id: &str) -> ExportWizardPanelSession {
    ExportWizardPanelSession::new(
        test_job_system(),
        job_id,
        export_wizard_pipeline_plan(ready_options()),
    )
}

fn ready_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\zircon-export\\assets\\assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\zircon_game.exe".to_string());
    options
}

fn modal_resource(window_id: &str) -> ToolResourceKey {
    ToolResourceKey::modal_surface(UiWindowId::new(window_id))
}
