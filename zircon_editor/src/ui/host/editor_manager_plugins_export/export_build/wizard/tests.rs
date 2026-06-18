use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiNodeProjection};
use zircon_runtime::plugin::{ExportPackagingStrategy, ExportPipelineStage};

use super::*;

#[derive(Default)]
struct StubRunner {
    executions: Vec<ExportWizardCommandExecution>,
    seen_stages: Vec<ExportPipelineStage>,
}

impl StubRunner {
    fn with_execution(execution: ExportWizardCommandExecution) -> Self {
        Self {
            executions: vec![execution],
            seen_stages: Vec::new(),
        }
    }
}

impl ExportWizardCommandRunner for StubRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, String> {
        self.seen_stages.push(command.stage);
        if self.executions.is_empty() {
            return Ok(ExportWizardCommandExecution {
                exit_code: Some(0),
                stdout_lines: vec![command.stdout_banner("windows-release")],
                stderr_lines: Vec::new(),
            });
        }
        Ok(self.executions.remove(0))
    }
}

struct CancelAfterRuns {
    requested_after_runs: usize,
    observed_runs: Rc<Cell<usize>>,
}

impl CancelAfterRuns {
    fn new(requested_after_runs: usize) -> Self {
        Self {
            requested_after_runs,
            observed_runs: Rc::new(Cell::new(0)),
        }
    }

    fn observer(&self) -> Rc<Cell<usize>> {
        Rc::clone(&self.observed_runs)
    }
}

impl ExportWizardCancelSignal for CancelAfterRuns {
    fn is_cancel_requested(&self) -> bool {
        self.observed_runs.get() >= self.requested_after_runs
    }
}

struct ObservingRunner {
    inner: StubRunner,
    observed_runs: Rc<Cell<usize>>,
}

impl ExportWizardCommandRunner for ObservingRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, String> {
        let execution = self.inner.run(command);
        self.observed_runs.set(self.observed_runs.get() + 1);
        execution
    }
}

struct BlockingRunner {
    stage_started: Sender<ExportPipelineStage>,
    release_stage: Receiver<()>,
    seen_stages: Vec<ExportPipelineStage>,
}

impl BlockingRunner {
    fn new(stage_started: Sender<ExportPipelineStage>, release_stage: Receiver<()>) -> Self {
        Self {
            stage_started,
            release_stage,
            seen_stages: Vec::new(),
        }
    }
}

impl ExportWizardCommandRunner for BlockingRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, String> {
        self.seen_stages.push(command.stage);
        let _ = self.stage_started.send(command.stage);
        let _ = self.release_stage.recv();
        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines: vec![command.stdout_banner("windows-release")],
            stderr_lines: Vec::new(),
        })
    }
}

#[test]
fn export_wizard_pipeline_plan_builds_stage_commands_in_cli_order() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.repo_root = Some("E:\\Git\\ZirconEngine".to_string());
    options.source_asset_manifest = Some("D:\\assets\\cooked-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options.offline = true;
    options.dry_run = true;

    let plan = export_wizard_pipeline_plan(options);

    assert!(plan.is_ready(), "{:?}", plan);
    assert_eq!(
        plan.stages
            .iter()
            .map(|command| command.stage)
            .collect::<Vec<_>>(),
        export_pipeline_stages().to_vec()
    );

    let validate = plan
        .command(ExportPipelineStage::Validate)
        .expect("validate command");
    assert_eq!(validate.program, "python");
    assert_eq!(validate.argument_value("--stage"), Some("validate"));
    assert_eq!(
        validate.argument_value("--profile"),
        Some("windows-release")
    );
    assert_eq!(
        validate.argument_value("--project"),
        Some("zircon-project.toml")
    );
    assert!(validate.contains_flag("--offline"));
    assert!(validate.contains_flag("--dry-run"));
    assert_eq!(
        parse_export_pipeline_stage(
            validate
                .argument_value("--stage")
                .expect("stage argument should exist")
        ),
        Some(ExportPipelineStage::Validate)
    );

    let native_dynamic = plan
        .command(ExportPipelineStage::NativeDynamic)
        .expect("native dynamic command");
    assert_eq!(
        native_dynamic.argument_value("--stage"),
        Some("native_dynamic")
    );
    assert_eq!(
        native_dynamic.argument_value("--validate-report"),
        Some("D:\\zircon-export\\stages\\validate\\report.json")
    );
    assert_eq!(
        parse_export_pipeline_stage("native_dynamic"),
        Some(ExportPipelineStage::NativeDynamic)
    );
    assert_eq!(
        parse_export_pipeline_stage("NativeDynamic"),
        Some(ExportPipelineStage::NativeDynamic)
    );
}

#[test]
fn export_wizard_pipeline_plan_selects_stages_from_packaging_strategies() {
    let library_embed = export_wizard_pipeline_plan(
        ready_export_options().with_strategies([ExportPackagingStrategy::LibraryEmbed]),
    );
    assert_eq!(
        stage_sequence(&library_embed),
        vec![
            ExportPipelineStage::Validate,
            ExportPipelineStage::CompileHost,
            ExportPipelineStage::CookAssets,
            ExportPipelineStage::Pack,
            ExportPipelineStage::PlatformBundle,
            ExportPipelineStage::Report,
        ]
    );
    assert!(library_embed
        .command(ExportPipelineStage::SourceTemplate)
        .is_none());
    assert!(library_embed
        .command(ExportPipelineStage::NativeDynamic)
        .is_none());

    let source_template = export_wizard_pipeline_plan(
        ready_export_options().with_strategies([ExportPackagingStrategy::SourceTemplate]),
    );
    assert_eq!(
        stage_sequence(&source_template),
        vec![
            ExportPipelineStage::Validate,
            ExportPipelineStage::SourceTemplate,
            ExportPipelineStage::Report,
        ]
    );

    let native_dynamic = export_wizard_pipeline_plan(
        ready_export_options().with_strategies([ExportPackagingStrategy::NativeDynamic]),
    );
    assert_eq!(
        stage_sequence(&native_dynamic),
        vec![
            ExportPipelineStage::Validate,
            ExportPipelineStage::NativeDynamic,
            ExportPipelineStage::CompileHost,
            ExportPipelineStage::CookAssets,
            ExportPipelineStage::Pack,
            ExportPipelineStage::PlatformBundle,
            ExportPipelineStage::Report,
        ]
    );

    let combined = export_wizard_pipeline_plan(ready_export_options().with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::NativeDynamic,
        ExportPackagingStrategy::LibraryEmbed,
    ]));
    assert_eq!(stage_sequence(&combined), export_pipeline_stages().to_vec());
}

#[test]
fn export_wizard_pipeline_plan_threads_stage_artifact_inputs() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.pack_file = Some("D:\\zircon-export\\custom\\game.zrpack".to_string());
    options.previous_pack = Some("D:\\old\\game.zrpack".to_string());
    options.delta_pack = Some("D:\\zircon-export\\custom\\game.zrpd".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options.template_dir = Some("export-templates\\windows-x86_64-library_embed-debug".to_string());
    options.determinism_check = true;

    let plan = export_wizard_pipeline_plan(options);

    let native_dynamic = plan
        .command(ExportPipelineStage::NativeDynamic)
        .expect("native dynamic command");
    assert_eq!(
        native_dynamic.argument_value("--validate-report"),
        Some("D:\\zircon-export\\stages\\validate\\report.json")
    );
    assert!(native_dynamic.produced_artifacts.iter().any(|artifact| {
        artifact.key == "plugins_dir" && artifact.path.ends_with("stages\\native_dynamic\\plugins")
    }));
    assert!(native_dynamic.produced_artifacts.iter().any(|artifact| {
        artifact.key == "loader_manifest"
            && artifact
                .path
                .ends_with("stages\\native_dynamic\\plugins\\native_plugins.toml")
    }));

    let cook_assets = plan
        .command(ExportPipelineStage::CookAssets)
        .expect("cook assets command");
    assert_eq!(
        cook_assets.argument_value("--asset-manifest"),
        Some("D:\\assets\\source-assets.json")
    );
    assert!(cook_assets.produced_artifacts.iter().any(|artifact| {
        artifact.key == "cooked_asset_manifest"
            && artifact.path.ends_with("stages\\cook_assets\\assets.json")
    }));

    let pack = plan
        .command(ExportPipelineStage::Pack)
        .expect("pack command");
    assert_eq!(
        pack.argument_value("--asset-manifest"),
        Some("D:\\zircon-export\\stages\\cook_assets\\assets.json")
    );
    assert_eq!(
        pack.argument_value("--pack-file"),
        Some("D:\\zircon-export\\custom\\game.zrpack")
    );
    assert_eq!(
        pack.argument_value("--previous-pack"),
        Some("D:\\old\\game.zrpack")
    );
    assert_eq!(
        pack.argument_value("--delta-pack"),
        Some("D:\\zircon-export\\custom\\game.zrpd")
    );
    assert!(pack.contains_flag("--determinism-check"));

    let platform_bundle = plan
        .command(ExportPipelineStage::PlatformBundle)
        .expect("platform bundle command");
    assert_eq!(
        platform_bundle.argument_value("--pack-file"),
        Some("D:\\zircon-export\\custom\\game.zrpack")
    );
    assert_eq!(
        platform_bundle.argument_value("--host-executable"),
        Some("D:\\zircon-export\\host\\ZirconRuntime.exe")
    );
    assert_eq!(
        platform_bundle.argument_value("--template-dir"),
        Some("export-templates\\windows-x86_64-library_embed-debug")
    );
}

#[test]
fn export_wizard_pipeline_plan_reports_missing_execution_inputs() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.previous_pack = Some("D:\\old\\game.zrpack".to_string());

    let plan = export_wizard_pipeline_plan(options);

    assert!(!plan.is_ready());
    assert!(plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("previous_pack and delta_pack")));
    assert!(plan
        .command(ExportPipelineStage::CookAssets)
        .expect("cook assets command")
        .missing_inputs
        .contains(&"source_asset_manifest"));
    assert!(plan
        .command(ExportPipelineStage::PlatformBundle)
        .expect("platform bundle command")
        .missing_inputs
        .contains(&"host_executable"));
    assert!(plan
        .command(ExportPipelineStage::Pack)
        .expect("pack command")
        .missing_inputs
        .contains(&"previous_pack+delta_pack"));
}

#[test]
fn export_wizard_pipeline_banners_drive_progress_parser() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut progress = ExportWizardProgressState::new();

    for command in &plan.stages {
        progress.push_stdout_line(&command.stdout_banner(&plan.profile));
    }

    assert_eq!(progress.current_stage(), Some(ExportPipelineStage::Report));
    assert_eq!(
        progress
            .snapshot(ExportPipelineStage::Report)
            .expect("report snapshot")
            .profile
            .as_deref(),
        Some("windows-release")
    );
}

#[test]
fn export_wizard_stage_execution_feeds_stdout_into_progress() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let command = plan
        .command(ExportPipelineStage::Pack)
        .expect("pack command");
    let mut runner = StubRunner::with_execution(ExportWizardCommandExecution {
        exit_code: Some(0),
        stdout_lines: vec![
            command.stdout_banner("windows-release"),
            "pack=D:\\zircon-export\\stages\\pack\\assets.zrpack".to_string(),
            "report=D:\\zircon-export\\stages\\pack\\report.json".to_string(),
            r#""fatal": false,"#.to_string(),
        ],
        stderr_lines: Vec::new(),
    });
    let mut progress = ExportWizardProgressState::new();

    let execution = execute_export_wizard_stage(command, &mut runner, &mut progress);

    assert_eq!(execution.exit_code, Some(0));
    assert!(!execution.fatal);
    assert!(execution.diagnostics.is_empty());
    assert_eq!(
        execution
            .progress
            .snapshot(ExportPipelineStage::Pack)
            .expect("pack progress")
            .report_path
            .as_deref(),
        Some("D:\\zircon-export\\stages\\pack\\report.json")
    );
    assert_eq!(runner.seen_stages, vec![ExportPipelineStage::Pack]);
}

#[test]
fn export_wizard_stage_execution_preserves_report_json_diagnostics() {
    let plan = export_wizard_pipeline_plan(ready_export_options());
    let command = plan
        .command(ExportPipelineStage::Report)
        .expect("report command");
    let mut runner = StubRunner::with_execution(ExportWizardCommandExecution {
        exit_code: Some(0),
        stdout_lines: vec![
            command.stdout_banner("windows-release"),
            r#""diagnostics": ["#.to_string(),
            r#"  "validate failed","#.to_string(),
            r#"],"#.to_string(),
            r#""fatal": true,"#.to_string(),
        ],
        stderr_lines: Vec::new(),
    });
    let mut progress = ExportWizardProgressState::new();

    let execution = execute_export_wizard_stage(command, &mut runner, &mut progress);

    assert!(execution.fatal);
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic == "validate failed"));
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("reported fatal status")));
}

#[test]
fn export_wizard_pipeline_execution_stops_on_missing_inputs_before_process_run() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::default();

    let execution = execute_export_wizard_pipeline(&plan, &mut runner);

    assert_eq!(
        execution.stages.last().expect("stopped stage").stage,
        ExportPipelineStage::CookAssets
    );
    assert!(execution.fatal);
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("source_asset_manifest")));
    assert_eq!(
        runner.seen_stages,
        vec![
            ExportPipelineStage::Validate,
            ExportPipelineStage::SourceTemplate,
            ExportPipelineStage::NativeDynamic,
            ExportPipelineStage::CompileHost,
        ]
    );
}

#[test]
fn export_wizard_pipeline_execution_stops_on_plan_diagnostics_before_process_run() {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options.previous_pack = Some("D:\\old\\game.zrpack".to_string());
    let plan = export_wizard_pipeline_plan(options);
    let mut runner = StubRunner::default();

    let execution = execute_export_wizard_pipeline(&plan, &mut runner);

    assert!(execution.fatal);
    assert!(execution.stages.is_empty());
    assert!(runner.seen_stages.is_empty());
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("previous_pack and delta_pack")));
}

#[test]
fn export_wizard_pipeline_execution_stops_on_process_failure() {
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

    let execution = execute_export_wizard_pipeline(&plan, &mut runner);

    assert_eq!(execution.stages.len(), 1);
    assert!(execution.fatal);
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("Validate stderr: validate failed")));
    assert!(execution
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("exited with code 2")));
}

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

#[test]
fn export_wizard_view_model_projects_plan_stage_rows_and_controls() {
    let mut options = ExportWizardPipelineOptions::new(
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
    assert_eq!(rows.len(), export_pipeline_stages().len());
    let validate = rows
        .iter()
        .find(|row| row.stage == ExportPipelineStage::Validate)
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
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::new(
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
        .find(|row| row.stage == ExportPipelineStage::CookAssets)
        .expect("CookAssets row should exist");
    assert_eq!(cook_assets.missing_inputs, vec!["source_asset_manifest"]);
    let platform_bundle = rows
        .iter()
        .find(|row| row.stage == ExportPipelineStage::PlatformBundle)
        .expect("PlatformBundle row should exist");
    assert_eq!(platform_bundle.missing_inputs, vec!["host_executable"]);
}

#[test]
fn export_wizard_view_model_drains_job_events_into_terminal_rows() {
    let mut options = ExportWizardPipelineOptions::new(
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
    assert!(rows.iter().all(|row| row
        .report_path
        .as_deref()
        .is_some_and(|path| path.ends_with("report.json"))));
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
        export_pipeline_stages().len()
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
    let plan = export_wizard_pipeline_plan(ExportWizardPipelineOptions::new(
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

fn ready_export_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::new(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options
}

fn stage_sequence(plan: &ExportWizardPipelinePlan) -> Vec<ExportPipelineStage> {
    plan.stages
        .iter()
        .map(|command| command.stage)
        .collect::<Vec<_>>()
}

fn desktop_export_panel_template_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should be inside repo root")
        .join("zircon_plugins")
        .join("editor_build_export_desktop")
        .join("editor")
        .join("panel.v2.ui.toml")
}

fn find_projection_node<'a>(
    node: &'a RetainedUiNodeProjection,
    control_id: &str,
) -> Option<&'a RetainedUiNodeProjection> {
    if node.control_id.as_deref() == Some(control_id) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_projection_node(child, control_id))
}
